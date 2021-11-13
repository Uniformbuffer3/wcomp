use crate::wcomp::WComp;
use ews::Buffer;
use screen_task::ScreenTask;
use crate::geometry_manager::{GeometryEvent,SurfaceEvent};
use crate::event_processing::WCompMessage;

impl WComp {
    fn process_new_surface(&mut self, surface: &ews::WlSurface)->bool{
        let role = ews::get_role(surface);
        let result = ews::with_states(&surface,|surface_data|{
            let mut redraw = false;
            if let Some(true) = ews::surface_id(&surface_data).map(|id|self.geometry_manager.surface_ref(id).is_some()) {return redraw;}
            if let Some(ews::BufferAssignment::NewBuffer{buffer,delta}) = &surface_data.cached_state.current::<ews::SurfaceAttributes>().buffer {
                let id = ews::surface_id(&surface_data).unwrap();

                let size = self.geometry_manager.get_surface_optimal_size();
                let position = self.geometry_manager.get_surface_optimal_position(&size);
                let geometry = pal::Rectangle::from((pal::Position2D::from(position),size));

                let inner_geometry = surface_data.cached_state.current::<ews::SurfaceCachedState>().geometry.map(|geometry|{
                    println!("Geometry: {:#?}",geometry);
                    let size = pal::Size2D::from((geometry.size.w as u32,geometry.size.h as u32));
                    let position = pal::Position2D::from((geometry.loc.x as i32, geometry.loc.y as i32));
                    pal::Rectangle::from((position,size))
                }).unwrap_or(geometry);

                let serial = ews::SERIAL_COUNTER.next_serial().into();
                let handle = surface.clone();

                match ews::buffer_type(&buffer) {
                    Some(ews::BufferType::Shm)=>{
                        ews::with_buffer_contents(&buffer,|data,info|{
                            let has_been_added = match role {
                                Some("xdg_toplevel")=>{
                                    let event = GeometryEvent::Surface{serial,event: SurfaceEvent::Added{id,handle,inner_geometry,geometry}};
                                    self.messages.borrow_mut().push(WCompMessage::from(event));
                                    true
                                }
                                _=>false
                            };
                            if has_been_added {
                                let source = crate::utils::shm_convert_format(data,info);
                                self.wgpu_engine.task_handle_cast_mut(&self.screen_task, |screen_task: &mut ScreenTask|{
                                    screen_task.create_surface(id, "", source, position.into(), [info.width as u32,info.height as u32]);
                                });
                                redraw = true;
                            }

                        }).unwrap();
                    }
                    Some(ews::BufferType::Dma)=>{
                        buffer.as_ref().user_data().get::<ews::Dmabuf>().map(|dmabuf|{
                            let has_been_added = match role {
                                Some("xdg_toplevel")=>{
                                    let event = GeometryEvent::Surface{serial,event: SurfaceEvent::Added{id,handle,inner_geometry,geometry}};
                                    self.messages.borrow_mut().push(WCompMessage::from(event));
                                    true
                                }
                                _=>false
                            };
                            if has_been_added {
                                let size = pal::Size2D::from([dmabuf.width() as u32,dmabuf.height() as u32]);
                                let fd = dmabuf.handles().next().unwrap();
                                let plane_offset = dmabuf.offsets().next().unwrap() as u64;
                                let plane_stride = dmabuf.strides().next().unwrap() as u32;
                                let modifier = dmabuf.format().modifier;
                                let info = screen_task::DmabufInfo {
                                    fd,
                                    size: size.into(),
                                    modifier,
                                    plane_offset,
                                    plane_stride
                                };
                                let source = screen_task::SurfaceSource::Dmabuf{info};
                                self.wgpu_engine.task_handle_cast_mut(&self.screen_task, |screen_task: &mut ScreenTask|{
                                    screen_task.create_surface(id, "", source, position.into(), size.into());
                                });
                                redraw = true;
                            }
                        });
                    }
                    _=>()
                }
            }
            else{println!("No buffer attached");}
            redraw
        });
        match result {
            Ok(redraw)=>redraw,
            Err(err)=>{println!("{:#?}",err);false}
        }
    }
    pub(crate) fn process_wayland_request(&mut self, message: ews::WaylandRequest)->bool{
        let mut redraw = false;
        match message {
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::NewToplevel{surface}}=>{
                let size = self.geometry_manager.get_surface_optimal_size();
                surface.with_pending_state(|state|{
                    state.size = Some((size.width as i32,size.height as i32).into());
                    println!("Size: {:#?}",state.size);
                }).unwrap();
                surface.send_configure();
            }

            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::AckConfigure{surface,configure}}=>{
                println!("New Surface!");

            },
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Move{surface,seat,serial}}=>{
                log::info!(target: "WComp","Moving surface: {:#?}",surface);
                if let Some(seat_id) = ews::seat_id(&seat){
                    if let Some(cursor) = self.ews.get_cursor(seat_id){
                        cursor.grab_start_data().map(|mut start_data|{
                            if let Some((surface,position)) = start_data.focus.as_mut() {
                                let id = ews::with_states(&surface,|surface_data|ews::surface_id(&surface_data)).ok().flatten().unwrap();
                                if let Some(surface) = self.geometry_manager.surface_ref(id) {
                                    position.x = surface.geometry.position.x;
                                    position.y = surface.geometry.position.y;
                                }
                                else{log::error!(target: "WComp","Moving surface: surface {:#?} not found",surface);};
                            }

                            let move_logic = crate::move_logic::MoveLogic::new(start_data,self.messages.clone());
                            cursor.set_grab(move_logic, serial);
                        });
                    }
                    else{log::error!(target: "WComp","Moving surface: cursor {:#?} not found",seat_id);};
                }else{log::error!(target: "WComp","Moving surface: cannot get id from cursor");};
            }
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Resize{surface,seat,serial,edges}}=>{
                println!("Resize event detected!");
            }
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Maximize{surface}}=>{
                println!("Maximize event detected!");
            }
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::UnMaximize{surface}}=>{
                println!("Unmaximize event detected!");
            }
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Fullscreen{surface,output}}=>{
                println!("Fullscreen event detected!");
            }
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::UnFullscreen{surface}}=>{
                println!("Unfullscreen event detected!");
            }
            ews::WaylandRequest::XdgRequest{request: ews::XdgRequest::Minimize{surface}}=>{
                println!("Minimize event detected!");
            }
            ews::WaylandRequest::Commit {surface}=>{
                log::info!(target: "WComp","Committing surface: {:#?}",surface);
                redraw |= self.process_new_surface(&surface);
                let result = ews::with_states(&surface,|surface_data|{
                    let attributes = surface_data.cached_state.current::<ews::SurfaceAttributes>();
                    if let Some(ews::BufferAssignment::NewBuffer{buffer,delta}) = &attributes.buffer {
                        println!("Buffer: {:#?}",buffer);
                        let position_offset = surface_data.cached_state.current::<ews::SurfaceCachedState>().geometry.map(|geometry|{
                            let point: (i32,i32) = geometry.loc.into();
                            pal::Offset2D::from(point)
                        }).unwrap_or(pal::Offset2D::from((0,0)));

                        let size_offset = pal::Size2D{width: position_offset.x as u32,height: position_offset.y as u32};
                        let buffer_type = ews::buffer_type(&buffer);
                        match buffer_type {
                            Some(ews::BufferType::Shm)=>{
                                ews::with_buffer_contents(&buffer,|data,info|{
                                    log::info!(target: "WComp","Committing surface {:#?}",surface);
                                    let id = ews::surface_id(&surface_data).unwrap();
                                    //let source = crate::utils::shm_convert_format(data,info);
                                    self.wgpu_engine.task_handle_cast_mut(&self.screen_task, |screen_task: &mut ScreenTask|{
                                        screen_task.update_data(id, data.to_vec());
                                    });
                                }).unwrap();
                                redraw = true;
                            }
                            Some(ews::BufferType::Dma)=>{}
                            _=>()
                        }
                    }
                });
                redraw=true;
            },
            ews::WaylandRequest::Seat{seat,request: ews::SeatRequest::CursorImage(image_status)}=>{
                match image_status {
                    ews::CursorImageStatus::Image(surface)=>{
                        println!("Cursor surface: {:#?}",surface);
                    }
                    ews::CursorImageStatus::Default=>{

                    }
                    ews::CursorImageStatus::Hidden=>{

                    }
                }
                //println!("Seat request: {:#?}",request);
            }
            ews::WaylandRequest::Dmabuf{buffer}=>{
                println!("Dmabuf request");
            }
            ews::WaylandRequest::Dnd{dnd}=>{
                println!("Dnd request");
            }

            other_request=>{
                //println!("Other request: {:#?}",other_request);
            }

        }
        redraw
    }
}
