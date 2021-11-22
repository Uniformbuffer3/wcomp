use crate::geometry_manager::{
    CursorRequest, KeyboardRequest, OutputRequest, SeatRequest, SurfaceRequest, WCompEvent,
    WCompRequest,
};
use crate::wcomp::WComp;
use screen_task::ScreenTask;

impl WComp {
    pub fn process_requests(
        &mut self,
        requests: impl IntoIterator<Item = WCompRequest>,
    ) -> impl Iterator<Item = WCompEvent> {
        requests
            .into_iter()
            .flat_map(|request| match request {
                WCompRequest::Output {
                    request: OutputRequest::Added { id, handle, size },
                } => self
                    .geometry_manager
                    .add_output(id, handle, size)
                    .collect::<Vec<_>>(),
                WCompRequest::Output {
                    request: OutputRequest::Removed { id },
                } => self.geometry_manager.del_output(id).collect::<Vec<_>>(),
                WCompRequest::Output {
                    request: OutputRequest::Resized { id, size },
                } => self
                    .geometry_manager
                    .resize_output(id, size)
                    .collect::<Vec<_>>(),
                WCompRequest::Output {
                    request: OutputRequest::Moved { old, new_position },
                } => Vec::new(),
                WCompRequest::Seat {
                    request: SeatRequest::Added { id, name },
                } => self.geometry_manager.add_seat(id, name).collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request: SeatRequest::Removed { id },
                } => self.geometry_manager.del_seat(id).collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request:
                        SeatRequest::Cursor(CursorRequest::Added {
                            id,
                            position,
                            image,
                        }),
                } => self
                    .geometry_manager
                    .add_cursor(id, position, image)
                    .collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request: SeatRequest::Keyboard(KeyboardRequest::Added { id, rate, delay }),
                } => self
                    .geometry_manager
                    .add_keyboard(id, rate, delay)
                    .collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request: SeatRequest::Keyboard(KeyboardRequest::Removed { id }),
                } => self.geometry_manager.del_keyboard(id).collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request:
                        SeatRequest::Keyboard(KeyboardRequest::Key {
                            id,
                            time,
                            code,
                            key,
                            state,
                        }),
                } => self
                    .geometry_manager
                    .keyboard_key(id, time, code, key, state)
                    .collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Removed { id }),
                } => self.geometry_manager.del_cursor(id).collect::<Vec<_>>(),

                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Moved { id, position }),
                } => self
                    .geometry_manager
                    .move_cursor(id, position)
                    .collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request:
                        SeatRequest::Cursor(CursorRequest::Button {
                            id,
                            time,
                            code,
                            key,
                            state,
                        }),
                } => self
                    .geometry_manager
                    .cursor_button(id, time, code, key, state)
                    .collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Focus { id, surface }),
                } => {
                    unimplemented!()
                }
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Entered { id, output_id }),
                } => self
                    .geometry_manager
                    .enter_cursor(id, output_id)
                    .collect::<Vec<_>>(),
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Left { id, output_id }),
                } => self
                    .geometry_manager
                    .left_cursor(id, output_id)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::Added { id, kind },
                } => self
                    .geometry_manager
                    .add_surface(id, kind)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::Removed { id },
                } => self.geometry_manager.del_surface(id).collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request:
                        SurfaceRequest::Moved {
                            id,
                            position,
                            depth,
                        },
                } => self
                    .geometry_manager
                    .move_surface(id, position)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::Resized { id, size },
                } => self
                    .geometry_manager
                    .resize_surface(id, size)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::InteractiveResizeStart { id,serial,edge },
                } => self
                    .geometry_manager
                    .interactive_resize_start(id,serial,edge)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::InteractiveResize { id, serial, inner_size },
                } => self
                    .geometry_manager
                    .interactive_resize_surface(id, serial, inner_size)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::InteractiveResizeStop { id,serial },
                } => self
                    .geometry_manager
                    .interactive_resize_end(id,serial)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request:
                        SurfaceRequest::Configuration {
                            id,
                            geometry,
                            min_size,
                            max_size,
                        },
                } => self
                    .geometry_manager
                    .configure(id, geometry, min_size, max_size)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request:
                        SurfaceRequest::BufferAttached {
                            id,
                            handle,
                            inner_geometry,
                            size,
                        },
                } => self
                    .geometry_manager
                    .attach_buffer(id, handle, inner_geometry, size)
                    .collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::BufferDetached { id },
                } => self.geometry_manager.detach_buffer(id).collect::<Vec<_>>(),
                WCompRequest::Surface {
                    request: SurfaceRequest::Committed { id },
                } => self.geometry_manager.commit_surface(id).collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
