mod surface_manager;
pub use surface_manager::{
    PopupState, Surface, SurfaceEvent, SurfaceKind, SurfaceManager, SurfaceRequest,
};

mod output_manager;
pub use output_manager::{OutputEvent, OutputManager, OutputRequest};

mod seat_manager;
pub use seat_manager::{
    Cursor, CursorEvent, CursorRequest, KeyboardEvent, KeyboardRequest, SeatEvent, SeatManager,
    SeatRequest,
};

use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum WCompEvent {
    Seat { serial: u32, event: SeatEvent },
    Surface { serial: u32, event: SurfaceEvent },
    Output { serial: u32, event: OutputEvent },
}
impl WCompEvent {
    pub fn serial(&self) -> u32 {
        match self {
            Self::Seat { serial, event: _ } => *serial,
            Self::Surface { serial, event: _ } => *serial,
            Self::Output { serial, event: _ } => *serial,
        }
    }
}
impl From<(u32, SeatEvent)> for WCompEvent {
    fn from(tuple: (u32, SeatEvent)) -> Self {
        Self::Seat {
            serial: tuple.0,
            event: tuple.1,
        }
    }
}
impl From<(u32, SurfaceEvent)> for WCompEvent {
    fn from(tuple: (u32, SurfaceEvent)) -> Self {
        Self::Surface {
            serial: tuple.0,
            event: tuple.1,
        }
    }
}
impl From<(u32, OutputEvent)> for WCompEvent {
    fn from(tuple: (u32, OutputEvent)) -> Self {
        Self::Output {
            serial: tuple.0,
            event: tuple.1,
        }
    }
}

#[derive(Debug)]
pub enum WCompRequest {
    Seat { request: SeatRequest },
    Surface { request: SurfaceRequest },
    Output { request: OutputRequest },
}

#[derive(Debug)]
pub struct GeometryManager {
    seat_manager: SeatManager,
    surface_manager: SurfaceManager,
    output_manager: OutputManager,
    events: Vec<WCompRequest>,
}

impl GeometryManager {
    pub fn new() -> Self {
        let seat_manager = SeatManager::new();
        let surface_manager = SurfaceManager::new();
        let output_manager = OutputManager::new();
        let events = Vec::new();
        Self {
            seat_manager,
            surface_manager,
            output_manager,
            events,
        }
    }

    pub fn events(&mut self) -> impl Iterator<Item = WCompRequest> + '_ {
        self.events.drain(..)
    }

    pub fn get_surface_at(&mut self, position: &pal::Position2D<i32>) -> Option<&Surface> {
        self.surface_manager.get_surface_at(&position)
    }

    pub fn add_seat(
        &mut self,
        id: usize,
        name: String,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Seat entered");
        let events = self
            .seat_manager
            .add_seat(id, name)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }
    pub fn del_seat(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Seat removed");
        let events = self
            .seat_manager
            .del_seat(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn add_keyboard(
        &mut self,
        id: usize,
        rate: i32,
        delay: i32,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Keyboard added");
        let events = self
            .seat_manager
            .add_keyboard(id, rate, delay)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn del_keyboard(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Keyboard removed");
        let events = self
            .seat_manager
            .del_keyboard(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn keyboard_key(
        &mut self,
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Key>,
        state: pal::State,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Keyboard key {:#?} {:#?}",key,state);
        let events = self
            .seat_manager
            .keyboard_key(id, time, code, key, state)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn cursor_ref(&self, id: usize) -> Option<&Cursor> {
        self.seat_manager.cursor_ref(id)
    }

    pub fn add_cursor(
        &mut self,
        id: usize,
        position: pal::Position2D<i32>,
        image: Option<usize>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Cursor added");
        let events = self
            .seat_manager
            .add_cursor(id, position, image)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn del_cursor(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Cursor removed");
        let events = self
            .seat_manager
            .del_cursor(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn enter_cursor(
        &mut self,
        id: usize,
        output_id: usize,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        /*
            self.seat_manager.cursor_ref(id).map(|cursor|{
                cursor.
            })
            self.surface_manager.add_cursor_surface(id, handle, offset, space);
        */
        log::info!(target:"WComp","Geometry manager | Cursor entered");
        let events = self
            .seat_manager
            .enter_cursor(id, output_id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }
    pub fn left_cursor(
        &mut self,
        id: usize,
        output_id: usize,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Cursor left");
        let events = self
            .seat_manager
            .left_cursor(id, output_id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    /*
        pub fn relative_move_cursor(&mut self, id: usize, position: pal::Position2D<i32>){
            let events = self.seat_manager.cursor_ref(id).map(|cursor|cursor.output()).flatten().map(|output_id|{
                self.output_manager.relative_to_absolute(output_id,position).map(|absolute_position|{
                    self.seat_manager.move_cursor(id, absolute_position)
                })
            }).flatten().into_iter().flatten().map(WCompEvent::from);

            self.postprocess_events(events);
        }
    */
    pub fn relative_to_absolute(
        &self,
        output_id: usize,
        position: pal::Position2D<i32>,
    ) -> Option<pal::Position2D<i32>> {
        self.output_manager
            .relative_to_absolute(output_id, position)
    }

    pub fn relative_move_cursor(
        &mut self,
        id: usize,
        position: pal::Position2D<i32>,
    ) -> Option<pal::Position2D<i32>> {
        self.seat_manager
            .cursor_ref(id)
            .map(|cursor| cursor.output().clone())
            .flatten()
            .map(|output_id| {
                self.output_manager
                    .relative_to_absolute(output_id, position)
            })
            .flatten()
    }

    pub fn move_cursor(
        &mut self,
        id: usize,
        position: pal::Position2D<i32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Cursor moved");
        let events = self
            .seat_manager
            .move_cursor(id, position)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }
    pub fn cursor_button(
        &mut self,
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Button>,
        state: pal::State,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Cursor button");
        let focus = self
            .seat_manager
            .cursor_ref(id)
            .map(|cursor| cursor.position().clone())
            .map(|position| self.get_surface_at(&position))
            .flatten()
            .map(|surface| surface.id());

        let events = self
            .seat_manager
            .cursor_button(id, time, code, key, state)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from)
            .chain(
                self.seat_manager
                    .keyboard_focus(id, focus)
                    .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
                    .map(WCompEvent::from),
            );

        self.postprocess_events(events)
    }
    pub fn cursor_axis(
        &mut self,
        id: usize,
        time: u32,
        source: pal::AxisSource,
        direction: pal::AxisDirection,
        value: pal::AxisValue,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Cursor button");
        let events = self
            .seat_manager
            .cursor_axis(id, time, source, direction, value)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);

        self.postprocess_events(events)
    }
    pub fn add_output(
        &mut self,
        id: usize,
        handle: std::sync::Arc<pal::wgpu::Surface>,
        size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Output added");
        let events = self
            .output_manager
            .add_output(id, handle, size)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }
    pub fn del_output(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Output removed");
        let events = self
            .output_manager
            .del_output(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }
    pub fn resize_output(
        &mut self,
        id: usize,
        size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Output resized");
        let events = self
            .output_manager
            .resize_output(id, size)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn get_surface_optimal_size(&self) -> pal::Size2D<u32> {
        self.output_manager.get_surface_optimal_size()
    }

    pub fn get_surface_optimal_position(
        &self,
        size: &pal::Size2D<u32>,
    ) -> (pal::Position2D<i32>, u32) {
        (self.output_manager.get_surface_optimal_position(size), 0)
    }

    pub fn surfaces_ref(&self) -> impl Iterator<Item = &Surface> {
        self.surface_manager.surfaces_ref()
    }
    pub fn surface_ref(&self, id: usize) -> Option<&Surface> {
        self.surface_manager.surface_ref(id)
    }

    pub fn add_surface(
        &mut self,
        id: usize,
        kind: SurfaceKind,
        position: pal::Position2D<i32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Surface added");
        let events = self
            .surface_manager
            .add_surface(id, kind, position)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);

        self.postprocess_events(events)
    }

    pub fn del_surface(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Surface removed");
        let events = self
            .surface_manager
            .del_surface(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn interactive_resize_start(
        &mut self,
        id: usize,
        serial: u32,
        edge: ews::ResizeEdge,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Interactive resize started on surface {}", id);
        let events = self
            .surface_manager
            .interactive_resize_start(id, serial, edge)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn interactive_resize_end(
        &mut self,
        id: usize,
        serial: u32,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Interactive resize stopped on surface {}", id);
        let events = self
            .surface_manager
            .interactive_resize_end(id, serial)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn resize_surface(
        &mut self,
        id: usize,
        size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Surface {} resized to {}", id, size);
        let events = self
            .surface_manager
            .resize_surface(id, size)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn maximize_surface(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Surface {} maximized", id);
        let events = self
            .surface_manager
            .maximize_surface(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn unmaximize_surface(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Surface {} unmaximized", id);
        let events = self
            .surface_manager
            .unmaximize_surface(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn interactive_resize_surface(
        &mut self,
        id: usize,
        serial: u32,
        inner_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Surface {} resized interactively to {}", id, inner_size);
        let events = self
            .surface_manager
            .interactive_resize_surface(id, serial, inner_size)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn configure_surface(
        &mut self,
        id: usize,
        inner_geometry: Option<pal::Rectangle<i32, u32>>,
        min_size: pal::Size2D<u32>,
        max_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Configuring surface {}", id);
        let events = self
            .surface_manager
            .configure_surface(id, inner_geometry, min_size, max_size)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn move_surface(
        &mut self,
        id: usize,
        position: pal::Position2D<i32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Surface moved");
        let events = self
            .surface_manager
            .move_surface(id, position)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn attach_buffer(
        &mut self,
        id: usize,
        handle: ews::WlBuffer,
        inner_geometry: pal::Rectangle<i32, u32>,
        suggested_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Buffer attached");
        let events = self
            .surface_manager
            .attach_buffer(id, handle, inner_geometry, suggested_size)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn detach_buffer(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target:"WComp","Geometry manager | Buffer detached");
        let events = self
            .surface_manager
            .detach_buffer(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    pub fn commit_surface(&mut self, id: usize) -> impl Iterator<Item = WCompEvent> + Clone {
        log::info!(target: "WComp","Geometry manager | Committed surface: {}",id);
        let events = self
            .surface_manager
            .commit_surface(id)
            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
            .map(WCompEvent::from);
        self.postprocess_events(events)
    }

    fn postprocess_events(
        &mut self,
        events: impl Iterator<Item = WCompEvent> + Clone,
    ) -> impl Iterator<Item = WCompEvent> + Clone {
        let mut additional_events = Vec::new();
        events.clone().for_each(|event| {
            match event {
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Keyboard(KeyboardEvent::Focus { id: _, surface }),
                } => {
                    additional_events.append(
                        &mut self
                            .surface_manager
                            .focus_surface(surface)
                            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
                            .map(WCompEvent::from)
                            .collect(),
                    );
                }
                WCompEvent::Seat {
                    serial: _,
                    event: SeatEvent::Cursor(CursorEvent::Moved { id, position }),
                } => {
                    let focus = self.get_surface_at(&position).map(|surface| surface.id());
                    additional_events.append(
                        &mut self
                            .seat_manager
                            .focus_cursor(id, focus)
                            .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
                            .map(WCompEvent::from)
                            .collect(),
                    );
                }
                WCompEvent::Output {
                    serial: _,
                    event: OutputEvent::Removed { id: _ },
                } => {
                    /*
                    let screen_size = self.output_manager.screen_size();
                    let surfaces = self.surface_manager.surfaces_mut().filter_map(|surface| {
                        if screen_size.contains(&surface.position) {
                            Some(surface)
                        } else {
                            None
                        }
                    });
                    */
                    /*
                    for surface in surfaces {
                        if let Some(size) = surface.size().cloned() {
                            surface.update_position(
                                self.output_manager
                                    .get_surface_optimal_position(&size),
                            );
                        };
                        //TODO Missing propagate resize to client
                    }
                    */
                }
                WCompEvent::Output {
                    serial: _,
                    event: OutputEvent::Moved { id: _, position: _ },
                } => {
                    /*
                    let surfaces = self.surface_manager.surfaces_mut().filter_map(|surface| {
                        if old.geometry.contains(&surface.position) {
                            Some(surface)
                        } else {
                            None
                        }
                    });
                    */
                    /*
                    for surface in surfaces {
                        if let Some(size) = surface.size().cloned() {
                            surface.update_position(
                                self.output_manager
                                    .get_surface_optimal_position(&size),
                            );
                        }
                        //TODO Missing propagate resize to client
                    }
                    */
                }
                WCompEvent::Surface {
                    serial: _,
                    event: SurfaceEvent::Maximized { id },
                } => {
                    let mut events = self
                        .surface_manager
                        .surface_ref(id)
                        .map(|surface| surface.position())
                        .cloned()
                        .map(|position| self.output_manager.get_output_at(&position))
                        .flatten()
                        .map(|output| output.geometry.clone())
                        .map(|output_geometry| {
                            std::iter::empty()
                                .chain(
                                    self.surface_manager
                                        .move_surface(id, output_geometry.position),
                                )
                                .chain(
                                    self.surface_manager
                                        .resize_surface(id, output_geometry.size),
                                )
                        })
                        .into_iter()
                        .flatten()
                        .map(|event| (ews::SERIAL_COUNTER.next_serial().into(), event))
                        .map(WCompEvent::from)
                        .collect::<Vec<_>>();

                    additional_events.append(&mut events);
                }
                _ => (),
            }
            //self.events.push(event);
        });

        events.chain(additional_events.into_iter())
    }

    fn reposition_surfaces(&mut self) {}
}
