//! Module containing wayland events processing functions.

use crate::geometry_manager::{PopupState, SurfaceKind, SurfaceRequest, WCompRequest};
use crate::wcomp::WComp;
use ews::Buffer;

impl WComp {
    /// Process the [Wayland requests][ews::WaylandRequest].
    pub(crate) fn process_wayland_requests(
        &mut self,
        requests: impl Iterator<Item = ews::WaylandRequest>,
    ) -> impl Iterator<Item = WCompRequest> {
        requests.flat_map(|request|{
            match request {
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::NewToplevel{surface}}=>{
                    let size = self.geometry_manager.get_surface_optimal_size();
                    let (position,_depth) = self.geometry_manager.get_surface_optimal_position(&size);

                    surface.with_pending_state(|state|{
                        state.size = Some((size.width as i32,size.height as i32).into());
                    }).unwrap();
                    surface.send_configure();

                    surface.get_surface().map(|raw_surface|{
                        ews::with_states(&raw_surface,|surface_data|{
                            let id = ews::surface_id(&surface_data).expect(&format!("{:#?} not found",surface));
                            let kind = SurfaceKind::from(surface.clone());
                            WCompRequest::Surface{request: SurfaceRequest::Add{id,kind,position}}
                        }).ok()
                    }).flatten().into_iter().collect::<Vec<_>>()
                },
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::NewPopup{surface,positioner}}=>{
                    println!("Positioner: {:#?}",positioner);
                    surface.with_pending_state(|state|{
                        state.positioner = positioner;
                        state.geometry = ews::Rectangle::from_loc_and_size((0,0),positioner.rect_size);
                    }).unwrap();
                    surface.send_configure().unwrap();

                    surface.get_surface().map(|raw_surface|{
                        ews::with_states(&raw_surface,|surface_data|{
                            let id = ews::surface_id(&surface_data).expect(&format!("{:#?} not found",surface));
                            let position = pal::Position2D::from((0i32,0i32));
                            let anchor = pal::Rectangle::from((
                                pal::Position2D::from((positioner.anchor_rect.loc.x,positioner.anchor_rect.loc.y)),
                                pal::Size2D::from((positioner.anchor_rect.size.w as u32,positioner.anchor_rect.size.h as u32)),
                            ));
                            let offset = pal::Offset2D::from((positioner.offset.x,positioner.offset.y));

                            let state = PopupState {
                                anchor,
                                anchor_edges: positioner.anchor_edges,
                                gravity: positioner.gravity,
                                constraint_adjustment: positioner.constraint_adjustment,
                                offset,
                                reactive: positioner.reactive,
                            };

                            let kind = SurfaceKind::Popup{
                                handle: surface.clone(),
                                state
                            };
                            WCompRequest::Surface{request: SurfaceRequest::Add{id,kind,position}}
                        }).ok()
                    }).flatten().into_iter().collect::<Vec<_>>()
                },
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::AckConfigure{surface,configure: _}}=>{
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
                },
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Move{surface: _,seat,serial}}=>{
                    if let Some(seat_id) = ews::seat_id(&seat){
                        if let Some(cursor) = self.ews.get_cursor(seat_id){
                            cursor.grab_start_data().map(|mut start_data|{
                                if let Some((surface,position)) = start_data.focus.as_mut() {
                                    let id = ews::with_states(&surface,|surface_data|ews::surface_id(&surface_data)).ok().flatten().unwrap();
                                    if let Some(surface) = self.geometry_manager.surface_ref(id) {
                                        position.x = surface.position().x;
                                        position.y = surface.position().y;
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
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Resize{surface: _,seat,serial,edges}}=>{
                    if let Some(seat_id) = ews::seat_id(&seat){
                        if let Some(cursor) = self.ews.get_cursor(seat_id){
                            cursor.grab_start_data().map(|mut start_data|{
                                if let Some((surface,position)) = start_data.focus.as_mut() {
                                    let id = ews::with_states(&surface,|surface_data|ews::surface_id(&surface_data)).ok().flatten().unwrap();
                                    if let Some(surface) = self.geometry_manager.surface_ref(id) {
                                        position.x = surface.position().x;
                                        position.y = surface.position().y;
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
                    surface.get_surface().map(|raw_surface|{
                        ews::with_states(&raw_surface, |surface_data| {
                            let id = ews::surface_id(&surface_data).expect(&format!("{:#?} not found",surface));
                            WCompRequest::Surface{request: SurfaceRequest::Maximize {id}}
                        }).ok()
                    }).flatten().into_iter().collect::<Vec<_>>()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::UnMaximize{surface}}=>{
                    surface.get_surface().map(|raw_surface|{
                        ews::with_states(&raw_surface, |surface_data| {
                            let id = ews::surface_id(&surface_data).expect(&format!("{:#?} not found",surface));
                            WCompRequest::Surface{request: SurfaceRequest::Unmaximize {id}}
                        }).ok()
                    }).flatten().into_iter().collect::<Vec<_>>()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Fullscreen{surface: _,output: _}}=>{
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::UnFullscreen{surface: _}}=>{
                    Vec::new()
                }
                ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Minimize{surface: _}}=>{
                    println!("Minimize event detected!");
                    Vec::new()
                }
                ews::WaylandRequest::Commit {surface}=>{
                    ews::with_states(&surface,|surface_data|{
                        let id = ews::surface_id(&surface_data).expect(&format!("Id on {:#?} not found, it is likely a ews bug missing to track such surface",surface));
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
                                            events.push(WCompRequest::Surface{request: SurfaceRequest::AttachBuffer{id,handle,inner_geometry,size}});
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
                                            events.push(WCompRequest::Surface{request: SurfaceRequest::AttachBuffer{id,handle,inner_geometry,size}});
                                        });
                                    }
                                    _=>unreachable!()
                                }
                            }
                            Some(ews::BufferAssignment::Removed)=>{
                                events.push(WCompRequest::Surface{request: SurfaceRequest::DetachBuffer{id}});
                            }
                            None=>()
                        }

                        attributes.buffer = None;
                        events.push(WCompRequest::Surface{request: SurfaceRequest::Commit{id}});
                        events
                    }).unwrap_or(Vec::new())
                },
                ews::WaylandRequest::Seat{seat: _,request: ews::SeatRequest::CursorImage(image_status)}=>{
                    match image_status {
                        ews::CursorImageStatus::Image(_surface)=>{
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
                ews::WaylandRequest::Dmabuf{buffer: _}=>{
                    Vec::new()
                }
                ews::WaylandRequest::Dnd{dnd: _}=>{
                    Vec::new()
                }
                ews::WaylandRequest::SurfaceRemoved{id}=>{
                    vec![WCompRequest::Surface{request: SurfaceRequest::Remove{id}}]
                }
                _other_request=>{
                    //println!("Other request: {:#?}",other_request);
                    Vec::new()
                }

            }
        }).collect::<Vec<_>>().into_iter()
    }
}
