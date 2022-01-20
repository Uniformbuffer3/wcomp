//! Module containing wcomp events processing functions.

use crate::geometry_manager::{
    CursorEvent, KeyboardEvent, OutputEvent, SeatEvent, SurfaceEvent, SurfaceKind, WCompEvent,
};
use crate::wcomp::WComp;
use ews::Buffer;
use screen_task::ScreenTask;

impl WComp {
    /// Process [wcomp events][WCompEvent].
    pub fn process_events(&mut self, events: impl Iterator<Item = WCompEvent>) -> bool {
        let mut redraw = false;
        events.for_each(|event| {
            match event {
                WCompEvent::Output {
                    serial: _,
                    event: OutputEvent::Added { id, handle, size },
                } => {
                    log::info!(target: "WCompEvent","Output {} added of {:?}",id,size);
                    self.wgpu_engine.create_surface(
                        id.into(),
                        String::from(format!("Output {}", id)),
                        handle,
                        size.width,
                        size.height,
                    );
                    redraw = true;
                }
                WCompEvent::Output {
                    serial: _,
                    event: OutputEvent::Removed { id },
                } => {
                    log::info!(target: "WCompEvent","Output {} removed",id);
                    self.wgpu_engine.destroy_surface(id.into());
                    redraw = true;
                }
                WCompEvent::Output {
                    serial: _,
                    event: OutputEvent::Resized { id, size },
                } => {
                    log::info!(target: "WCompEvent","Output {} resized to {:?}",id,size);
                    self.wgpu_engine.resize_surface(id, size.width, size.height);
                    redraw = true;
                }
                WCompEvent::Output {
                    serial: _,
                    event: OutputEvent::Moved { id, position },
                } => {
                    log::info!(target: "WCompEvent","Output {} moved to {:?}",id,position);
                    self.wgpu_engine.task_handle_cast_mut(
                        &self.screen_task,
                        |screen_task: &mut ScreenTask| {
                            screen_task.move_output(id, [position.x, position.y]);
                        },
                    );
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Added { id, name },
                } => {
                    log::info!(target: "WCompEvent","Seat {} added",id);
                    self.ews.create_seat(id, name);
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Removed { id },
                } => {
                    log::info!(target: "WCompEvent","Seat {} removed",id);
                    self.ews.destroy_seat(id);
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Keyboard(KeyboardEvent::Added { id, rate, delay }),
                } => {
                    log::info!(target: "WCompEvent","Keyboard {} added",id);
                    self.ews.add_keyboard(id, rate, delay);
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Keyboard(KeyboardEvent::Removed { id }),
                } => {
                    log::info!(target: "WCompEvent","Keyboard {} removed",id);
                    self.ews.del_keyboard(id.into());
                }
                WCompEvent::Seat {
                    serial,
                    event: SeatEvent::Keyboard(KeyboardEvent::Focus { id, surface }),
                } => {
                    log::info!(target: "WCompEvent","Keyboard {} focus {:?}",id,surface);
                    self.ews.get_keyboard(id).map(|keyboard| {
                        let focus = surface
                            .map(|surface_id| {
                                self.geometry_manager
                                    .surface_ref(surface_id)
                                    .map(|surface| surface.handle())
                                    .flatten()
                            })
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
                    log::info!(target: "WCompEvent","Keyboard {} key {:?}",id,key);
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
                    serial: _,
                    event:
                        SeatEvent::Cursor(CursorEvent::Added {
                            id,
                            position: _,
                            image: _,
                        }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} added",id);
                    self.ews.add_cursor(id);
                }
                WCompEvent::Seat {
                    serial: _,
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
                            .cursor_ref(id)
                            .map(|cursor| cursor.focus())
                            .cloned()
                            .flatten()
                            .map(|surface_id| self.geometry_manager.surface_ref(surface_id))
                            .flatten()
                            .map(|surface| {
                                surface.handle().map(|handle| {
                                    let position = surface.position().clone();
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
                    log::info!(target: "WCompEvent","Cursor {} button: {:?}",id,key);
                    self.ews.get_cursor(id).map(|cursor| {
                        let state = match state {
                            pal::State::Down => ews::ButtonState::Pressed,
                            pal::State::Up => ews::ButtonState::Released,
                        };
                        cursor.button(code, state, serial.into(), time);
                    });
                }
                WCompEvent::Seat {
                    serial: _,
                    event:
                        SeatEvent::Cursor(CursorEvent::Axis {
                            id,
                            time,
                            source,
                            direction,
                            value,
                        }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} axis",id);
                    self.ews.get_cursor(id).map(|cursor| {
                        let source = match source {
                            pal::AxisSource::Wheel => ews::AxisSource::Wheel,
                        };

                        let mut axis_frame = ews::AxisFrame::new(time).source(source);

                        let direction = match direction {
                            pal::AxisDirection::Horizontal => ews::Axis::HorizontalScroll,
                            pal::AxisDirection::Vertical => ews::Axis::VerticalScroll,
                        };

                        match value {
                            pal::AxisValue::Discrete(value) => {
                                axis_frame = axis_frame.discrete(direction, value);
                            }
                            _ => (),
                        }

                        cursor.axis(axis_frame);
                    });
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Cursor(CursorEvent::Focus { id, surface }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} focused surface {:?}",id,surface);
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Cursor(CursorEvent::Entered { id, output_id: _ }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} entered",id);
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Cursor(CursorEvent::Left { id, output_id: _ }),
                } => {
                    log::info!(target: "WCompEvent","Cursor {} left",id);
                }
                WCompEvent::Surface {
                    serial: _,
                    event: SurfaceEvent::Added { id, kind: _ },
                } => {
                    log::info!(target: "WCompEvent","Surface {} added",id);
                }
                WCompEvent::Surface {
                    serial: _,
                    event: SurfaceEvent::Removed { id },
                } => {
                    log::info!(target: "WCompEvent","Surface removed: {}",id);
                }
                WCompEvent::Surface {
                    serial: _,
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
                    serial: _,
                    event: SurfaceEvent::InteractiveResizeStarted {id,serial:_,edge}
                } => {
                    log::info!(target: "WCompEvent","Interactive resize started on surface {} with edge {:?}",id,edge);
                    self.geometry_manager
                        .surface_ref(id)
                        .map(|surface| match surface.kind() {
                            SurfaceKind::Toplevel { handle, .. } => {
                                handle
                                    .with_pending_state(|top_level_state| {
                                        top_level_state.states.set(ews::SurfaceState::Resizing);
                                    })
                                    .unwrap();
                                handle.send_configure();
                            }
                            _ => (),
                        });
                },
                WCompEvent::Surface {
                    serial: _,
                    event: SurfaceEvent::InteractiveResizeStopped {id,serial: _}
                } => {
                    log::info!(target: "WCompEvent","Interactive resize stopped on surface {}",id);
                    self.geometry_manager
                        .surface_ref(id)
                        .map(|surface| match surface.kind() {
                            SurfaceKind::Toplevel { handle, .. } => {
                                handle
                                    .with_pending_state(|top_level_state| {
                                        top_level_state.states.unset(ews::SurfaceState::Resizing);
                                    })
                                    .unwrap();
                                handle.send_configure();
                            }
                            _ => (),
                        });
                },
                WCompEvent::Surface {
                    serial: _,
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
                    serial: _,
                    event: SurfaceEvent::Configuration { id, size },
                } => {
                    log::info!(target: "WCompEvent","Configuration");
                    self.geometry_manager
                        .surface_ref(id)
                        .map(|surface| match surface.kind() {
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
                    serial: _,
                    event:
                        SurfaceEvent::BufferAttached {
                            id,
                            handle,
                            inner_geometry: _,
                            geometry,
                        },
                } => {
                    log::info!(target: "WCompEvent","Attaching buffer to surface {}",id);
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
                    serial: _,
                    event:
                        SurfaceEvent::BufferReplaced {
                            id,
                            handle,
                            inner_geometry: _,
                            geometry: _,
                        },
                } => {
                    log::info!(target: "WCompEvent","Replacing buffer of surface {}",id);
                    let source = match ews::buffer_type(&handle) {
                        Some(ews::BufferType::Shm) => {
                            ews::with_buffer_contents(&handle, |data, info| {
                                crate::utils::shm_convert_format(data, info)
                            })
                            .ok()
                        }
                        Some(ews::BufferType::Dma) => {
                            //Workaround to fix driver bug that is unable to import a second time the same dma buffer imported in the previous cycle.
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
                    serial: _,
                    event: SurfaceEvent::BufferDetached { id },
                } => {
                    log::info!(target: "WCompEvent","Detaching buffer from surface {}",id);
                    self.wgpu_engine.task_handle_cast_mut(
                        &self.screen_task,
                        |screen_task: &mut ScreenTask| {
                            screen_task.remove_surface(id);
                        },
                    );
                    redraw = true;
                }
                WCompEvent::Surface {
                    serial: _,
                    event: SurfaceEvent::Activated { id },
                } => {
                    log::info!(target: "WCompEvent","Surface {} activated",id);
                    self.geometry_manager
                        .surface_ref(id)
                        .map(|surface| match surface.kind() {
                            SurfaceKind::Toplevel { handle, .. } => {
                                handle
                                    .with_pending_state(|top_level_state| {
                                        top_level_state.states.set(ews::SurfaceState::Activated);
                                    })
                                    .unwrap();
                                handle.send_configure();
                            }
                            _ => (),
                        });
                }
                WCompEvent::Surface {
                    serial: _,
                    event: SurfaceEvent::Deactivated { id },
                } => {
                    log::info!(target: "WCompEvent","Surface {} deactivated",id);
                    self.geometry_manager
                        .surface_ref(id)
                        .map(|surface| match surface.kind() {
                            SurfaceKind::Toplevel { handle, .. } => {
                                handle
                                    .with_pending_state(|top_level_state| {
                                        top_level_state.states.unset(ews::SurfaceState::Activated);
                                    })
                                    .unwrap();
                                handle.send_configure();
                            }
                            _ => (),
                        });
                }
                WCompEvent::Surface {
                    serial: _,
                    event: SurfaceEvent::Committed { id },
                } => {
                    log::info!(target: "WCompEvent","Surface {} committed",id);
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
}
