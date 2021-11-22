use crate::event_processing::WCompMessage;
use crate::geometry_manager::{SurfaceKind, SurfaceRequest, WCompRequest};
use crate::wcomp::WComp;
use ews::Buffer;
use screen_task::ScreenTask;

impl WComp {
    pub(crate) fn process_wayland_requests(
        &mut self,
        requests: impl Iterator<Item = ews::WaylandRequest>,
    ) -> impl Iterator<Item = WCompRequest> {
        requests.flat_map(|request|{
            match request {
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::NewToplevel{surface}}=>{
                    let size = self.geometry_manager.get_surface_optimal_size();
                    surface.with_pending_state(|state|{
                        state.size = Some((size.width as i32,size.height as i32).into());
                    }).unwrap();
                    surface.send_configure();

                    surface.get_surface().map(|raw_surface|{
                        ews::with_states(&raw_surface,|surface_data|{
                            let id = ews::surface_id(&surface_data).expect(&format!("{:#?} not found",surface));
                            let kind = SurfaceKind::from(surface.clone());
                            WCompRequest::Surface{request: SurfaceRequest::Added{id,kind}}
                        }).ok()
                    }).flatten().into_iter().collect::<Vec<_>>()
                }

                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::AckConfigure{surface,configure}}=>{
                    /*
                    match configure {
                        ews::Configure::Toplevel(toplevel)=>{println!("Ack: {}",u32::from(toplevel.serial));}
                        ews::Configure::Popup(popup)=>{println!("Ack: {}",u32::from(popup.serial));}
                    }
                    */
                    ews::with_states(&surface, |surface_data| {
                        let id = ews::surface_id(&surface_data).expect(&format!("{:#?} not found",surface));
                        let surface_state = surface_data.cached_state.current::<ews::SurfaceCachedState>();
                        let geometry = surface_state.geometry.map(|geometry|{
                            let position = pal::Position2D::from((geometry.loc.x as i32,geometry.loc.y as i32));
                            let size = pal::Size2D::from((geometry.size.w as u32,geometry.size.h as u32));
                            pal::Rectangle::from((position,size))
                        });
                        let min_size = pal::Size2D::from((surface_state.min_size.w as u32,surface_state.min_size.h as u32));
                        let max_size = pal::Size2D::from((surface_state.max_size.w as u32,surface_state.max_size.h as u32));
                        vec![
                            WCompRequest::Surface{request: SurfaceRequest::Configuration {
                                id,
                                geometry,
                                min_size,
                                max_size,
                            }},

                        ]
                    }).ok().into_iter().flatten().collect::<Vec<_>>()

                    //Vec::new()
                },
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Move{surface,seat,serial}}=>{
                    if let Some(seat_id) = ews::seat_id(&seat){
                        if let Some(cursor) = self.ews.get_cursor(seat_id){
                            cursor.grab_start_data().map(|mut start_data|{
                                if let Some((surface,position)) = start_data.focus.as_mut() {
                                    let id = ews::with_states(&surface,|surface_data|ews::surface_id(&surface_data)).ok().flatten().unwrap();
                                    if let Some(surface) = self.geometry_manager.surface_ref(id) {
                                        position.x = surface.position.x;
                                        position.y = surface.position.y;
                                    }
                                    else{log::error!(target: "WComp","Moving surface: surface {:#?} not found",surface);};
                                }

                                let move_logic = crate::move_logic::MoveLogic::new(start_data,self.async_requests.clone());
                                cursor.set_grab(move_logic, serial);
                            });
                        }
                        else{log::error!(target: "WComp","Moving surface: cursor {:#?} not found",seat_id);};
                    }else{log::error!(target: "WComp","Moving surface: cannot get id from cursor");};
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Resize{surface,seat,serial,edges}}=>{
                    println!("Resize event detected!");
                    if let Some(seat_id) = ews::seat_id(&seat){
                        if let Some(cursor) = self.ews.get_cursor(seat_id){
                            cursor.grab_start_data().map(|mut start_data|{
                                if let Some((surface,position)) = start_data.focus.as_mut() {
                                    let id = ews::with_states(&surface,|surface_data|ews::surface_id(&surface_data)).ok().flatten().unwrap();
                                    if let Some(surface) = self.geometry_manager.surface_ref(id) {
                                        position.x = surface.position.x;
                                        position.y = surface.position.y;
                                        surface.inner_geometry().map(|geometry|{
                                            let move_logic = crate::resize_logic::ResizeLogic::new(start_data,self.async_requests.clone(),id,serial.into(),geometry.clone(),edges);
                                            cursor.set_grab(move_logic, serial);
                                        });
                                    }
                                    else{log::error!(target: "WComp","Resizing surface: surface {:#?} not found",surface);};
                                }
                            });
                        }
                        else{log::error!(target: "WComp","Resizing surface: cursor {:#?} not found",seat_id);};
                    }else{log::error!(target: "WComp","Resizing surface: cannot get id from cursor");};
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Maximize{surface}}=>{
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::UnMaximize{surface}}=>{
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Fullscreen{surface,output}}=>{
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::UnFullscreen{surface}}=>{
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Minimize{surface}}=>{
                    println!("Minimize event detected!");
                    Vec::new()
                }
                ews::WaylandRequest::Commit {surface}=>{
                    ews::with_states(&surface,|surface_data|{
                        let id = ews::surface_id(&surface_data).expect(&format!("{:#?} not found",surface));
                        let mut events = Vec::new();
                        let mut attributes = surface_data.cached_state.current::<ews::SurfaceAttributes>();
                        match attributes.buffer.as_ref() {
                            Some(ews::BufferAssignment::NewBuffer{buffer,delta: _})=>{
                                match ews::buffer_type(&buffer) {
                                    Some(ews::BufferType::Shm)=>{
                                        ews::with_buffer_contents(&buffer,|_data,info|{
                                            let size = pal::Size2D::from((info.width as u32,info.height as u32));
                                            let (position,_depth) = self.geometry_manager.get_surface_optimal_position(&size);
                                            let geometry = pal::Rectangle::from((position,size.clone()));
                                            let inner_geometry = surface_data.cached_state.current::<ews::SurfaceCachedState>().geometry.map(|geometry|{
                                                let size = pal::Size2D::from((geometry.size.w as u32,geometry.size.h as u32));
                                                let position = pal::Position2D::from((geometry.loc.x as i32, geometry.loc.y as i32));
                                                pal::Rectangle::from((position,size))
                                            }).unwrap_or(geometry);

                                            let handle = buffer.clone();
                                            events.push(WCompRequest::Surface{request: SurfaceRequest::BufferAttached{id,handle,inner_geometry,size}});
                                        }).unwrap();
                                    }
                                    Some(ews::BufferType::Dma)=>{
                                        buffer.as_ref().user_data().get::<ews::Dmabuf>().map(|dmabuf|{
                                            let size = pal::Size2D::from((dmabuf.width() as u32,dmabuf.height() as u32));
                                            let (position,_depth) = self.geometry_manager.get_surface_optimal_position(&size);
                                            let geometry = pal::Rectangle::from((position,size.clone()));
                                            let inner_geometry = surface_data.cached_state.current::<ews::SurfaceCachedState>().geometry.map(|geometry|{
                                                let size = pal::Size2D::from((geometry.size.w as u32,geometry.size.h as u32));
                                                let position = pal::Position2D::from((geometry.loc.x as i32, geometry.loc.y as i32));
                                                pal::Rectangle::from((position,size))
                                            }).unwrap_or(geometry);
                                            let handle = buffer.clone();
                                            events.push(WCompRequest::Surface{request: SurfaceRequest::BufferAttached{id,handle,inner_geometry,size}});
                                        });
                                    }
                                    _=>unreachable!()
                                }
                            }
                            Some(ews::BufferAssignment::Removed)=>{
                                events.push(WCompRequest::Surface{request: SurfaceRequest::BufferDetached{id}});
                            }
                            None=>()
                        }

                        attributes.buffer = None;
                        events.push(WCompRequest::Surface{request: SurfaceRequest::Committed{id}});
                        events
                    }).unwrap_or(Vec::new())
                },
                ews::WaylandRequest::Seat{seat,request: ews::SeatRequest::CursorImage(image_status)}=>{
                    match image_status {
                        ews::CursorImageStatus::Image(surface)=>{
                            //println!("Cursor surface: {:#?}",surface);
                        }
                        ews::CursorImageStatus::Default=>{

                        }
                        ews::CursorImageStatus::Hidden=>{

                        }
                    }
                    //println!("Seat request: {:#?}",request);
                    Vec::new()
                }
                ews::WaylandRequest::Dmabuf{buffer}=>{
                    Vec::new()
                }
                ews::WaylandRequest::Dnd{dnd}=>{
                    Vec::new()
                }
                ews::WaylandRequest::SurfaceRemoved{id}=>{
                    vec![WCompRequest::Surface{request: SurfaceRequest::Removed{id}}]
                }
                other_request=>{
                    //println!("Other request: {:#?}",other_request);
                    Vec::new()
                }

            }
        }).collect::<Vec<_>>().into_iter()
    }
}
