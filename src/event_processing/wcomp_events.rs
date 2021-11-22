use crate::geometry_manager::{
    CursorEvent, KeyboardEvent, OutputEvent, SeatEvent, SurfaceEvent, SurfaceKind, WCompEvent,
};
use crate::wcomp::WComp;
use ews::Buffer;
use screen_task::ScreenTask;

impl WComp {
    pub fn process_events(&mut self, events: impl Iterator<Item = WCompEvent>) -> bool {
        let mut redraw = false;
        events.for_each(|event| {
            match event {
                WCompEvent::Output {
                    serial,
                    event: OutputEvent::Added { id, handle, size },
                } => {
                    log::info!(target: "WCompEvent","Output added");
                    self.wgpu_engine.create_surface(
                        id.into(),
                        String::from("MainSurface"),
                        handle,
                        size.width,
                        size.height,
                    );
                    redraw = true;
                }
                WCompEvent::Output {
                    serial,
                    event: OutputEvent::Removed { id },
                } => {
                    log::info!(target: "WCompEvent","Output removed");
                    self.wgpu_engine.destroy_surface(id.into());
                    redraw = true;
                }
                WCompEvent::Output {
                    serial,
                    event: OutputEvent::Resized { id, size },
                } => {
                    log::info!(target: "WCompEvent","Output resized");
                    self.wgpu_engine.resize_surface(id, size.width, size.height);
                    redraw = true;
                }
                WCompEvent::Output {
                    serial,
                    event: OutputEvent::Moved { old, new_position },
                } => {
                    log::info!(target: "WCompEvent","Output moved");
                    //TODO Muovere output nel surface_manager
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Added { id, name },
                } => {
                    log::info!(target: "WCompEvent","Seat {} added",id);
                    self.ews.create_seat(id, name);
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Removed { id },
                } => {
                    log::info!(target: "WCompEvent","Seat {} removed",id);
                    self.ews.destroy_seat(id);
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Keyboard(KeyboardEvent::Added { id, rate, delay }),
                } => {
                    log::info!(target: "WCompEvent","Keyboard {} added",id);
                    self.ews.add_keyboard(id, rate, delay);
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Keyboard(KeyboardEvent::Removed { id }),
                } => {
                    log::info!(target: "WCompEvent","Keyboard {} removed",id);
                    self.ews.del_keyboard(id.into());
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Keyboard(KeyboardEvent::Focus { id, surface }),
                } => {
                    log::info!(target: "WCompEvent","Keyboard {} focus {:#?}",id,surface);
                    self.ews.get_keyboard(id).map(|keyboard| {
                        let focus = surface
                            .map(|surface_id| {
                                self.geometry_manager
                                    .surface_ref(surface_id)
                                    .map(|surface| surface.handle())
                            })
                            .flatten()
                            .flatten();
                        keyboard.set_focus(focus, serial.into());
                    });
                }
                WCompEvent::Seat {
                    serial,
                    event:
                        SeatEvent::Keyboard(KeyboardEvent::Key {
                            id,
                            time,
                            code,
                            key,
                            state,
                        }),
                } => {
                    log::info!(target: "WCompEvent","Keyboard {} key {:#?}",id,key);
                    self.ews.get_keyboard(id).map(|keyboard| {
                        let keystate = match state {
                            pal::State::Down => ews::KeyState::Pressed,
                            pal::State::Up => ews::KeyState::Released,
                        };
                        keyboard.input::<(), _>(
                            code,
                            keystate,
                            serial.into(),
                            time,
                            |_modifier, _keysim| ews::FilterResult::Forward,
                        );
                    });
                }
                WCompEvent::Seat {
                    serial,
                    event:
                        SeatEvent::Cursor(CursorEvent::Added {
                            id,
                            position,
                            image,
                        }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} added",id);
                    self.ews.add_cursor(id);
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Cursor(CursorEvent::Removed { id }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} removed",id);
                    self.ews.del_cursor(id);
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Cursor(CursorEvent::Moved { id, position }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} moved to {}",id,position);
                    if let Some(cursor_handle) = self.ews.get_cursor(id) {
                        let focus = self
                            .geometry_manager
                            .get_surface_at(&position)
                            .map(|surface| {
                                surface.handle().map(|handle| {
                                    let position = surface.position.clone();
                                    let position: (i32, i32) = position.into();
                                    (handle.clone(), position.into())
                                })
                            })
                            .flatten();
                        let position = (position.x as f64, position.y as f64).into();
                        let serial = serial.into();
                        let time = self.timer.elapsed().as_millis() as u32;
                        cursor_handle.motion(position, focus, serial, time)
                    } else {
                        log::error!(target: "WCompEvent","Seat {} not found to get cursor",id);
                    }
                    redraw = true;
                }
                WCompEvent::Seat {
                    serial,
                    event:
                        SeatEvent::Cursor(CursorEvent::Button {
                            id,
                            time,
                            code,
                            key,
                            state,
                        }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} button: {:#?}",id,key);
                    self.ews.get_cursor(id).map(|cursor| {
                        let state = match state {
                            pal::State::Down => ews::ButtonState::Pressed,
                            pal::State::Up => ews::ButtonState::Released,
                        };
                        cursor.button(code, state, serial.into(), time);
                    });
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Cursor(CursorEvent::Focus { id, surface }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} focus {:#?}",id,surface);
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Cursor(CursorEvent::Entered { id, output_id }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} entered",id);
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Cursor(CursorEvent::Left { id, output_id }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} left",id);
                }
                WCompEvent::Surface {
                    serial,
                    event: SurfaceEvent::Added { id, kind },
                } => {
                    log::info!(target: "WCompEvent","Surface {} added",id);
                }
                WCompEvent::Surface {
                    serial,
                    event: SurfaceEvent::Removed { id },
                } => {
                    log::info!(target: "WCompEvent","Surface removed: {:#?}",id);
                }
                WCompEvent::Surface {
                    serial,
                    event:
                        SurfaceEvent::Moved {
                            id,
                            position,
                            depth,
                        },
                } => {
                    log::info!(target: "WCompEvent","Surface {} moved to {}:{}",id,position,depth);
                    self.wgpu_engine.task_handle_cast_mut(
                        &self.screen_task,
                        |screen_task: &mut ScreenTask| {
                            screen_task.move_surface(id, [position.x, position.y, depth as i32]);
                        },
                    );
                    redraw = true;
                }
                WCompEvent::Surface {
                    serial,
                    event: SurfaceEvent::Resized { id, size },
                } => {
                    log::info!(target: "WCompEvent","Resizing surface {:#?} to {}",id,size);
                    self.wgpu_engine.task_handle_cast_mut(
                        &self.screen_task,
                        |screen_task: &mut ScreenTask| {
                            screen_task.resize_surface(id, size.into());
                        },
                    );
                    redraw = true;
                }
                WCompEvent::Surface {
                    serial,
                    event: SurfaceEvent::Configuration { id, size },
                } => {
                    log::info!(target: "WCompEvent","Configuration");
                    self.geometry_manager
                        .surface_ref(id)
                        .map(|surface| match &surface.kind {
                            SurfaceKind::Toplevel { handle, .. } => {
                                handle
                                    .with_pending_state(|state| {
                                        let inner_size: (i32, i32) =
                                            (size.width as i32, size.height as i32);
                                        state.size = Some(inner_size.into());
                                    })
                                    .ok()
                                    .map(|_| {
                                        handle.send_configure();
                                    });
                            }
                            _ => (),
                        });
                }
                WCompEvent::Surface {
                    serial,
                    event:
                        SurfaceEvent::BufferAttached {
                            id,
                            handle,
                            inner_geometry,
                            geometry,
                        },
                } => {
                    log::info!(target: "WCompEvent","Attaching buffer");
                    match ews::buffer_type(&handle) {
                        Some(ews::BufferType::Shm) => {
                            ews::with_buffer_contents(&handle, |data, info| {
                                let source = crate::utils::shm_convert_format(data, info);
                                self.wgpu_engine.task_handle_cast_mut(
                                    &self.screen_task,
                                    |screen_task: &mut ScreenTask| {
                                        screen_task.create_surface(
                                            id,
                                            "",
                                            source,
                                            [geometry.position.x, geometry.position.y, 0],
                                            geometry.size.into(),
                                        );
                                    },
                                );
                            })
                            .unwrap();
                        }
                        Some(ews::BufferType::Dma) => {
                            handle
                                .as_ref()
                                .user_data()
                                .get::<ews::Dmabuf>()
                                .map(|dmabuf| {
                                    let fd = dmabuf.handles().next().unwrap();
                                    let plane_offset = dmabuf.offsets().next().unwrap() as u64;
                                    let plane_stride = dmabuf.strides().next().unwrap() as u32;
                                    let modifier = dmabuf.format().modifier;
                                    let info = screen_task::DmabufInfo {
                                        fd,
                                        size: [dmabuf.width() as u32, dmabuf.height() as u32],
                                        modifier,
                                        plane_offset,
                                        plane_stride,
                                    };
                                    let source = screen_task::SurfaceSource::Dmabuf { info };
                                    self.wgpu_engine.task_handle_cast_mut(
                                        &self.screen_task,
                                        |screen_task: &mut ScreenTask| {
                                            screen_task.create_surface(
                                                id,
                                                "",
                                                source,
                                                [geometry.position.x, geometry.position.y, 0],
                                                geometry.size.into(),
                                            );
                                        },
                                    );
                                });
                        }
                        _ => (),
                    }
                }
                WCompEvent::Surface {
                    serial,
                    event:
                        SurfaceEvent::BufferReplaced {
                            id,
                            handle,
                            inner_geometry,
                            geometry,
                        },
                } => {
                    log::info!(target: "WCompEvent","Replacing buffer");
                    let source = match ews::buffer_type(&handle) {
                        Some(ews::BufferType::Shm) => {
                            ews::with_buffer_contents(&handle, |data, info| {
                                crate::utils::shm_convert_format(data, info)
                            })
                            .ok()
                        }
                        Some(ews::BufferType::Dma) => {
                            //Workaround to fix driver bug that is unable to im
                            /*
                            handle.as_ref().user_data().get::<ews::Dmabuf>().map(|dmabuf|{
                                let fd = dmabuf.handles().next().unwrap();
                                let plane_offset = dmabuf.offsets().next().unwrap() as u64;
                                let plane_stride = dmabuf.strides().next().unwrap() as u32;
                                let modifier = dmabuf.format().modifier;
                                let info = screen_task::DmabufInfo {
                                    fd,
                                    size: [dmabuf.width() as u32,dmabuf.height() as u32],
                                    modifier,
                                    plane_offset,
                                    plane_stride
                                };
                                screen_task::SurfaceSource::Dmabuf{info}
                            })
                            */
                            None
                        }
                        _ => None,
                    };

                    source.map(|source| {
                        self.wgpu_engine.task_handle_cast_mut(
                            &self.screen_task,
                            |screen_task: &mut ScreenTask| {
                                screen_task.update_source(id, source);
                            },
                        );
                    });
                }
                WCompEvent::Surface {
                    serial,
                    event: SurfaceEvent::BufferDetached { id },
                } => {
                    log::info!(target: "WCompEvent","Detaching buffer");
                    self.wgpu_engine.task_handle_cast_mut(
                        &self.screen_task,
                        |screen_task: &mut ScreenTask| {
                            screen_task.remove_surface(id);
                        },
                    );
                    redraw = true;
                }
                WCompEvent::Surface {
                    serial,
                    event: SurfaceEvent::Committed { id },
                } => {
                    log::info!(target: "WCompEvent","Committed");
                    self.geometry_manager
                        .surface_ref(id)
                        .map(|surface| (surface.handle().cloned(), surface.buffer().cloned()))
                        .map(|(surface, buffer)| {
                            surface.as_ref().map(|surface| {
                                let result = ews::with_states(surface, |surface_data| {
                                    let mut attributes = surface_data
                                        .cached_state
                                        .current::<ews::SurfaceAttributes>();
                                    let damages = attributes.damage.drain(..).collect::<Vec<_>>();
                                    if !damages.is_empty() {
                                        redraw = true;
                                        buffer.as_ref().map(|buffer| {
                                            let buffer_type = ews::buffer_type(&buffer);
                                            match buffer_type {
                                                Some(ews::BufferType::Shm) => {
                                                    ews::with_buffer_contents(
                                                        &buffer,
                                                        |data, _info| {
                                                            self.wgpu_engine.task_handle_cast_mut(
                                                                &self.screen_task,
                                                                |screen_task: &mut ScreenTask| {
                                                                    screen_task.update_data(
                                                                        id,
                                                                        data.to_vec(),
                                                                    );
                                                                },
                                                            );
                                                        },
                                                    )
                                                    .unwrap();
                                                }
                                                Some(ews::BufferType::Dma) => (),
                                                _ => (),
                                            }
                                            buffer.release();
                                        });
                                    }
                                });
                                match result {
                                    Ok(_) => (),
                                    Err(_) => (),
                                }
                            })
                        });
                }
                _ => (),
            }
        });
        redraw
    }

    fn process_new_surface(&mut self, surface: &ews::WlSurface) -> bool {
        let role = ews::get_role(surface);
        let result = ews::with_states(&surface, |surface_data| {
            let mut redraw = false;
            if let Some(true) = ews::surface_id(&surface_data)
                .map(|id| self.geometry_manager.surface_ref(id).is_some())
            {
                return redraw;
            }
            if let Some(ews::BufferAssignment::NewBuffer { buffer, delta }) = &surface_data
                .cached_state
                .current::<ews::SurfaceAttributes>()
                .buffer
            {
                let id = ews::surface_id(&surface_data).unwrap();
                let handle = surface.clone();
                match ews::buffer_type(&buffer) {
                    Some(ews::BufferType::Shm) => {
                        ews::with_buffer_contents(&buffer, |data, info| {
                            let size = pal::Size2D::from((info.width as u32, info.height as u32));
                            let (position, depth) =
                                self.geometry_manager.get_surface_optimal_position(&size);
                            let geometry = pal::Rectangle::from((position, size));
                            let inner_geometry = surface_data
                                .cached_state
                                .current::<ews::SurfaceCachedState>()
                                .geometry
                                .map(|geometry| {
                                    let size = pal::Size2D::from((
                                        geometry.size.w as u32,
                                        geometry.size.h as u32,
                                    ));
                                    let position = pal::Position2D::from((
                                        geometry.loc.x as i32,
                                        geometry.loc.y as i32,
                                    ));
                                    pal::Rectangle::from((position, size))
                                })
                                .unwrap_or(geometry.clone());

                            let has_been_added = match role {
                                Some("xdg_toplevel") => {
                                    //let event = WCompRequest::Surface{serial,event: SurfaceRequest::Added{id,handle,inner_geometry,geometry}};
                                    //self.messages.borrow_mut().push(WCompMessage::from(event));
                                    true
                                }
                                _ => false,
                            };
                            if has_been_added {
                                let source = crate::utils::shm_convert_format(data, info);
                                self.wgpu_engine.task_handle_cast_mut(
                                    &self.screen_task,
                                    |screen_task: &mut ScreenTask| {
                                        screen_task.create_surface(
                                            id,
                                            "",
                                            source,
                                            [geometry.position.x, geometry.position.y, 0],
                                            geometry.size.into(),
                                        );
                                    },
                                );
                                redraw = true;
                            }
                        })
                        .unwrap();
                    }
                    Some(ews::BufferType::Dma) => {
                        buffer
                            .as_ref()
                            .user_data()
                            .get::<ews::Dmabuf>()
                            .map(|dmabuf| {
                                let size = pal::Size2D::from((
                                    dmabuf.width() as u32,
                                    dmabuf.height() as u32,
                                ));
                                let (position, depth) =
                                    self.geometry_manager.get_surface_optimal_position(&size);
                                let geometry = pal::Rectangle::from((position, size.clone()));
                                let inner_geometry = surface_data
                                    .cached_state
                                    .current::<ews::SurfaceCachedState>()
                                    .geometry
                                    .map(|geometry| {
                                        let size = pal::Size2D::from((
                                            geometry.size.w as u32,
                                            geometry.size.h as u32,
                                        ));
                                        let position = pal::Position2D::from((
                                            geometry.loc.x as i32,
                                            geometry.loc.y as i32,
                                        ));
                                        pal::Rectangle::from((position, size))
                                    })
                                    .unwrap_or(geometry.clone());

                                let has_been_added = match role {
                                    Some("xdg_toplevel") => {
                                        //let event = WCompRequest::Surface{serial,event: SurfaceRequest::Added{id,handle,inner_geometry,geometry}};
                                        //self.messages.borrow_mut().push(WCompMessage::from(event));
                                        true
                                    }
                                    _ => false,
                                };
                                if has_been_added {
                                    let fd = dmabuf.handles().next().unwrap();
                                    let plane_offset = dmabuf.offsets().next().unwrap() as u64;
                                    let plane_stride = dmabuf.strides().next().unwrap() as u32;
                                    let modifier = dmabuf.format().modifier;
                                    let info = screen_task::DmabufInfo {
                                        fd,
                                        size: size.clone().into(),
                                        modifier,
                                        plane_offset,
                                        plane_stride,
                                    };
                                    let source = screen_task::SurfaceSource::Dmabuf { info };
                                    self.wgpu_engine.task_handle_cast_mut(
                                        &self.screen_task,
                                        |screen_task: &mut ScreenTask| {
                                            screen_task.create_surface(
                                                id,
                                                "",
                                                source,
                                                [geometry.position.x, geometry.position.y, 0],
                                                geometry.size.clone().into(),
                                            );
                                        },
                                    );
                                    redraw = true;
                                }
                            });
                    }
                    _ => (),
                }
            }
            redraw
        });
        match result {
            Ok(redraw) => redraw,
            Err(err) => {
                println!("{:#?}", err);
                false
            }
        }
    }
}
