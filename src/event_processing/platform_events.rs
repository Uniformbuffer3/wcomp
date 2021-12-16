use crate::geometry_manager::{
    CursorRequest, KeyboardRequest, OutputRequest, SeatRequest, WCompRequest,
};
use crate::wcomp::WComp;
use pal::PlatformBackend;

impl WComp {
    pub(crate) fn process_platform_requests(
        &mut self,
        requests: impl Iterator<Item = pal::Event>,
    ) -> impl Iterator<Item = WCompRequest> {
        requests
            .flat_map(|request| {
                match request {
                    pal::Event::Output { time: _, id, event } => match &event {
                        pal::OutputEvent::Added(_) => {
                            if self.platform.platform_type() == pal::PlatformType::Direct {
                                self.platform
                                    .request(vec![pal::definitions::Request::Surface {
                                        request: pal::definitions::SurfaceRequest::Create(Some(id)),
                                    }]);
                            }
                            Vec::new()
                        }
                        pal::OutputEvent::Removed => Vec::new(),
                        _ => Vec::new(),
                    },
                    pal::Event::Surface { time: _, id, event } => {
                        match &event {
                            pal::SurfaceEvent::Added(surface_info) => {
                                if let pal::definitions::Surface::WGpu(surface) =
                                    &surface_info.surface
                                {
                                    let id = id.into();
                                    let size = surface_info.size.clone();
                                    let handle = surface.clone();
                                    let request = WCompRequest::Output {
                                        request: OutputRequest::Added { id, handle, size },
                                    };
                                    vec![request]
                                } else {
                                    panic!("It is not of WGpu type");
                                }
                            }
                            pal::SurfaceEvent::Resized(size) => {
                                let id = id.into();
                                let size = size.clone();
                                let request = WCompRequest::Output {
                                    request: OutputRequest::Resized { id, size },
                                };
                                vec![request]
                                //self.messages.borrow_mut().push(WCompMessage::from(event));
                            }
                            pal::SurfaceEvent::Removed => {
                                let id = id.into();
                                let request = WCompRequest::Output {
                                    request: OutputRequest::Removed { id },
                                };
                                vec![request]
                                //self.messages.borrow_mut().push(WCompMessage::from(event));
                            }
                            _ => Vec::new(),
                        }
                    }
                    pal::Event::Seat { time, id, event } => {
                        match event {
                            pal::SeatEvent::Added { name } => {
                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Added { id, name },
                                };
                                vec![request]
                                //self.messages.borrow_mut().push(WCompMessage::from(event));
                            }
                            pal::SeatEvent::Removed => {
                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Removed { id },
                                };
                                vec![request]
                                //self.messages.borrow_mut().push(WCompMessage::from(event));
                            }
                            pal::SeatEvent::Keyboard(pal::KeyboardEvent::Added(_keyboard_info)) => {
                                //TODO Rimuovere valori hardcoded per il key rate and delay
                                let id = id.into();
                                let rate = 200;
                                let delay = 25;
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Keyboard(KeyboardRequest::Added {
                                        id,
                                        rate,
                                        delay,
                                    }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Keyboard(pal::KeyboardEvent::Removed) => {
                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Keyboard(KeyboardRequest::Removed { id }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Keyboard(pal::KeyboardEvent::Key {
                                code,
                                key,
                                state,
                                serial: _,
                                time,
                            }) => {
                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Keyboard(KeyboardRequest::Key {
                                        id,
                                        time,
                                        code,
                                        key,
                                        state,
                                    }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Keyboard(pal::KeyboardEvent::AutoRepeat {
                                rate: _,
                                delay: _,
                            }) => Vec::new(),
                            pal::SeatEvent::Keyboard(pal::KeyboardEvent::LayoutModified {
                                layout: _,
                            }) => Vec::new(),
                            pal::SeatEvent::Cursor(pal::CursorEvent::Added(_info)) => {
                                let size = pal::Size2D {
                                    width: 24,
                                    height: 24,
                                };
                                let (position, _depth) =
                                    self.geometry_manager.get_surface_optimal_position(&size);
                                let image = None;

                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Cursor(CursorRequest::Added {
                                        id,
                                        position,
                                        image,
                                    }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Cursor(pal::CursorEvent::Removed) => {
                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Cursor(CursorRequest::Removed { id }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Cursor(pal::CursorEvent::Button {
                                code,
                                key,
                                state,
                            }) => {
                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Cursor(CursorRequest::Button {
                                        id,
                                        time,
                                        code,
                                        key,
                                        state,
                                    }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Cursor(pal::CursorEvent::Axis {
                                source,
                                direction,
                                value,
                            }) => {
                                let id = id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Cursor(CursorRequest::Axis {
                                        id,
                                        time,
                                        source,
                                        direction,
                                        value,
                                    }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Cursor(pal::CursorEvent::Entered {
                                surface_id,
                                position: _,
                            }) => {
                                /*
                                self.platform.request(vec![pal::Request::Seat {
                                    request: pal::SeatRequest::Cursor(pal::CursorRequest::ChangeImage(pal::CursorImage::Hidden))
                                }]);
                                */
                                let id = id.into();
                                let output_id = surface_id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Cursor(CursorRequest::Entered {
                                        id,
                                        output_id,
                                    }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Cursor(pal::CursorEvent::Left { surface_id }) => {
                                /*
                                self.platform.request(vec![pal::Request::Seat {
                                    request: pal::SeatRequest::Cursor(pal::CursorRequest::ChangeImage(pal::CursorImage::Default))
                                }]);
                                */
                                let id = id.into();
                                let output_id = surface_id.into();
                                let request = WCompRequest::Seat {
                                    request: SeatRequest::Cursor(CursorRequest::Left {
                                        id,
                                        output_id,
                                    }),
                                };
                                vec![request]
                            }
                            pal::SeatEvent::Cursor(pal::CursorEvent::AbsoluteMovement {
                                position,
                            }) => self
                                .geometry_manager
                                .relative_move_cursor(id.into(), position)
                                .map(|position| {
                                    let id = id.into();
                                    let request = WCompRequest::Seat {
                                        request: SeatRequest::Cursor(CursorRequest::Moved {
                                            id,
                                            position,
                                        }),
                                    };
                                    vec![request]
                                })
                                .into_iter()
                                .flatten()
                                .collect::<Vec<_>>(),
                            _ => Vec::new(),
                        }
                    }
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
