use crate::geometry_manager::{SurfaceRequest, WCompRequest};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ResizeLogic {
    start_data: ews::GrabStartData,
    requests: Rc<RefCell<Vec<WCompRequest>>>,
    id: usize,
    serial: u32,
    inner_geometry: pal::Rectangle<i32,u32>,
    edge: ews::ResizeEdge,
}
impl ResizeLogic {
    pub fn new(
        start_data: ews::GrabStartData,
        requests: Rc<RefCell<Vec<WCompRequest>>>,
        id: usize,
        serial: u32,
        inner_geometry: pal::Rectangle<i32,u32>,
        edge: ews::ResizeEdge,
    ) -> Self {
        requests.borrow_mut().push(WCompRequest::Surface {
            request: SurfaceRequest::InteractiveResizeStart { id,serial, edge },
        });
        Self {
            start_data,
            requests,
            id,
            serial,
            inner_geometry,
            edge,
        }
    }
}
impl ews::PointerGrab for ResizeLogic {
    fn motion(
        &mut self,
        handle: &mut ews::PointerInnerHandle<'_>,
        location: ews::Point<f64, ews::Logical>,
        focus: Option<(ews::WlSurface, ews::Point<i32, ews::Logical>)>,
        serial: ews::Serial,
        time: u32,
    ) {
        self.start_data.focus.as_ref().map(|(focus, position)| {
            let id = ews::with_states(&focus, |surface_data| ews::surface_id(&surface_data))
                .ok()
                .flatten()
                .unwrap();
            let position = pal::Position2D::from((position.x,position.y));
            let cursor_position = pal::Position2D {
                x: location.x as i32,
                y: location.y as i32,
            };
            let relative_cursor_position = cursor_position.clone() - position.clone();

            let mut events = match self.edge {
                ews::ResizeEdge::Right => {
                    let inner_size = pal::Size2D {
                        width: (relative_cursor_position.x - self.inner_geometry.position.x) as u32,
                        height: self.inner_geometry.size.height,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }
                ews::ResizeEdge::Bottom => {
                    let inner_size = pal::Size2D {
                        width: self.inner_geometry.size.height,
                        height: (relative_cursor_position.y - self.inner_geometry.position.y) as u32,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }
                ews::ResizeEdge::BottomRight => {
                    let inner_size = pal::Size2D {
                        width: (relative_cursor_position.x - self.inner_geometry.position.x) as u32,
                        height: (relative_cursor_position.y - self.inner_geometry.position.y) as u32,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }
                ews::ResizeEdge::Left => {
                    let inner_size = pal::Size2D {
                        width: (self.inner_geometry.size.width as i32 - relative_cursor_position.x + self.inner_geometry.position.x) as u32,
                        height: self.inner_geometry.size.height,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }
                ews::ResizeEdge::Top => {
                    let inner_size = pal::Size2D {
                        width: self.inner_geometry.size.width,
                        height: (self.inner_geometry.size.height as i32 - relative_cursor_position.y + self.inner_geometry.position.y) as u32,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }
                ews::ResizeEdge::TopLeft => {
                    let inner_size = pal::Size2D {
                        width: (self.inner_geometry.size.width as i32 - relative_cursor_position.x + self.inner_geometry.position.x) as u32,
                        height: (self.inner_geometry.size.height as i32 - relative_cursor_position.y + self.inner_geometry.position.y) as u32,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }
                ews::ResizeEdge::TopRight => {
                    let inner_size = pal::Size2D {
                        width: (relative_cursor_position.x - self.inner_geometry.position.x) as u32,
                        height: (self.inner_geometry.size.height as i32 - relative_cursor_position.y + self.inner_geometry.position.y) as u32,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }
                ews::ResizeEdge::BottomLeft => {
                    let inner_size = pal::Size2D {
                        width: (self.inner_geometry.size.width as i32 - relative_cursor_position.x + self.inner_geometry.position.x) as u32,
                        height: (relative_cursor_position.y - self.inner_geometry.position.y) as u32,
                    };
                    vec![WCompRequest::Surface {
                        request: SurfaceRequest::InteractiveResize { id, serial: self.serial, inner_size },
                    }]
                }

                _ => Vec::new(),
            };
            self.requests.borrow_mut().append(&mut events);
        });

    }
    fn button(
        &mut self,
        handle: &mut ews::PointerInnerHandle<'_>,
        button: u32,
        state: ews::ButtonState,
        serial: ews::Serial,
        time: u32,
    ) {
        if button == self.start_data().button && state == ews::ButtonState::Released {
            handle.unset_grab(serial, time);
            self.requests.borrow_mut().push(WCompRequest::Surface {
                request: SurfaceRequest::InteractiveResizeStop { id: self.id,serial: self.serial },
            });
        }
    }
    fn axis(&mut self, handle: &mut ews::PointerInnerHandle<'_>, details: ews::AxisFrame) {
        //println!("Axis event");
    }
    fn start_data(&self) -> &ews::GrabStartData {
        &self.start_data
    }
}
