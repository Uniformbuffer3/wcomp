use crate::geometry_manager::{SurfaceRequest, WCompRequest};
use std::cell::RefCell;
use std::rc::Rc;

pub struct MoveLogic {
    start_data: ews::GrabStartData,
    requests: Rc<RefCell<Vec<WCompRequest>>>,
}
impl MoveLogic {
    pub fn new(start_data: ews::GrabStartData, requests: Rc<RefCell<Vec<WCompRequest>>>) -> Self {
        Self {
            start_data,
            requests,
        }
    }
}
impl ews::PointerGrab for MoveLogic {
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

            let offset = pal::Offset2D {
                x: self.start_data.location.x as i32 - position.x,
                y: self.start_data.location.y as i32 - position.y,
            };
            let cursor_position = pal::Position2D {
                x: location.x as i32,
                y: location.y as i32,
            };
            let position = cursor_position - offset;
            let event = WCompRequest::Surface {
                request: SurfaceRequest::Moved {
                    id,
                    position,
                    depth: 0,
                },
            };
            self.requests.borrow_mut().push(event.into());
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
        }
    }
    fn axis(&mut self, handle: &mut ews::PointerInnerHandle<'_>, details: ews::AxisFrame) {
        //println!("Axis event");
    }
    fn start_data(&self) -> &ews::GrabStartData {
        &self.start_data
    }
}
