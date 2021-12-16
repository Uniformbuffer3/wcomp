use crate::geometry_manager::{
    CursorRequest, KeyboardRequest, OutputRequest, SeatRequest, SurfaceRequest, WCompEvent,
    WCompRequest,
};
use crate::wcomp::WComp;

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
                } => {
                    log::info!(target: "WCompRequest","Output {} added",id);
                    self
                    .geometry_manager
                    .add_output(id, handle, size)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Output {
                    request: OutputRequest::Removed { id },
                } => {
                    log::info!(target: "WCompRequest","Output {} removed",id);
                    self.geometry_manager.del_output(id).collect::<Vec<_>>()
                },
                WCompRequest::Output {
                    request: OutputRequest::Resized { id, size },
                } => {
                    log::info!(target: "WCompRequest","Output {} resized to {:?}",id,size);
                    self
                    .geometry_manager
                    .resize_output(id, size)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Output {
                    request: OutputRequest::Moved { old: _, new_position: _ },
                } => {
                    //log::info!(target: "WCompRequest","Output {} moved to {:?}",id,new_position);
                    Vec::new()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Added { id, name },
                } => {
                    log::info!(target: "WCompRequest","Seat {} added",id);
                    self.geometry_manager.add_seat(id, name).collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Removed { id },
                } => {
                    log::info!(target: "WCompRequest","Seat {} removed",id);
                    self.geometry_manager.del_seat(id).collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request:
                        SeatRequest::Cursor(CursorRequest::Added {
                            id,
                            position,
                            image,
                        }),
                } => {
                    log::info!(target: "WCompRequest","Cursor on seat {} added",id);
                    self
                    .geometry_manager
                    .add_cursor(id, position, image)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Removed { id }),
                } => {
                    log::info!(target: "WCompRequest","Cursor on seat {} removed",id);
                    self.geometry_manager.del_cursor(id).collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Keyboard(KeyboardRequest::Added { id, rate, delay }),
                } => {
                    log::info!(target: "WCompRequest","Keyboard on seat {} added",id);
                    self
                    .geometry_manager
                    .add_keyboard(id, rate, delay)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Keyboard(KeyboardRequest::Removed { id }),
                } => {
                    log::info!(target: "WCompRequest","Keyboard {} removed",id);
                    self.geometry_manager.del_keyboard(id).collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request:
                        SeatRequest::Keyboard(KeyboardRequest::Key {
                            id,
                            time,
                            code,
                            key,
                            state,
                        }),
                } => {
                    log::info!(target: "WCompRequest","Keyboard on seat {} key {:?}",id,key);
                    self
                    .geometry_manager
                    .keyboard_key(id, time, code, key, state)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Moved { id, position }),
                } => {
                    log::info!(target: "WCompRequest","Cursor on seat {} moved to {:?}",id,position);
                    self
                    .geometry_manager
                    .move_cursor(id, position)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request:
                        SeatRequest::Cursor(CursorRequest::Button {
                            id,
                            time,
                            code,
                            key,
                            state,
                        }),
                } => {
                    log::info!(target: "WCompRequest","Cursor on seat {} button {:?}",id,key);
                    self
                    .geometry_manager
                    .cursor_button(id, time, code, key, state)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request:
                        SeatRequest::Cursor(CursorRequest::Axis {
                            id,
                            time,
                            source,
                            direction,
                            value
                        }),
                } => {
                    log::info!(target: "WCompRequest","Cursor on seat {} axis {:?} {:?} {:?}",id,source,direction,value);
                    self
                    .geometry_manager
                    .cursor_axis(id, time, source, direction, value)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Focus { id: _, surface: _ }),
                } => {
                    unimplemented!()
                }
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Entered { id, output_id }),
                } => {
                    log::info!(target: "WCompRequest","Cursor on seat {} entered output {}",id,output_id);
                    self
                    .geometry_manager
                    .enter_cursor(id, output_id)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Seat {
                    request: SeatRequest::Cursor(CursorRequest::Left { id, output_id }),
                } => {
                    log::info!(target: "WCompRequest","Cursor on seat {} left output {}",id,output_id);
                    self
                    .geometry_manager
                    .left_cursor(id, output_id)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::Add { id, kind, position},
                } => {
                    log::info!(target: "WCompRequest","Surface {} added",id);
                    self
                    .geometry_manager
                    .add_surface(id, kind, position)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::Remove { id },
                } => {
                    log::info!(target: "WCompRequest","Surface {} removed",id);
                    self.geometry_manager.del_surface(id).collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request:
                        SurfaceRequest::Move {
                            id,
                            position,
                        },
                } => {
                    log::info!(target: "WCompRequest","Surface {} moved to {:?}",id,position);
                    self
                    .geometry_manager
                    .move_surface(id, position)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::Resize { id, size },
                } => {
                    log::info!(target: "WCompRequest","Surface {} resized to {:?}",id,size);
                    self
                    .geometry_manager
                    .resize_surface(id, size)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::InteractiveResizeStart { id,serial,edge },
                } => {
                    log::info!(target: "WCompRequest","Surface {} interactive resize started on {:?} edge",id,edge);
                    self
                    .geometry_manager
                    .interactive_resize_start(id,serial,edge)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::InteractiveResize { id, serial, inner_size },
                } => {
                    log::info!(target: "WCompRequest","Surface {} interactive resize of {:?}",id,inner_size);
                    self
                    .geometry_manager
                    .interactive_resize_surface(id, serial, inner_size)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::InteractiveResizeStop { id,serial },
                } => {
                    log::info!(target: "WCompRequest","Surface {} interactive resize stopped",id);
                    self
                    .geometry_manager
                    .interactive_resize_end(id,serial)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request:
                        SurfaceRequest::Configuration {
                            id,
                            geometry,
                            min_size,
                            max_size,
                        },
                } => {
                    log::info!(target: "WCompRequest","Surface {} configuration",id);
                    self
                    .geometry_manager
                    .configure_surface(id, geometry, min_size, max_size)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request:
                        SurfaceRequest::AttachBuffer {
                            id,
                            handle,
                            inner_geometry,
                            size,
                        },
                } => {
                    log::info!(target: "WCompRequest","Surface {} buffer attached",id);
                    self
                    .geometry_manager
                    .attach_buffer(id, handle, inner_geometry, size)
                    .collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::DetachBuffer { id },
                } => {
                    log::info!(target: "WCompRequest","Surface {} buffer detached",id);
                    self.geometry_manager.detach_buffer(id).collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::Maximize { id },
                } => {
                    log::info!(target: "WCompRequest","Surface {} maximized",id);
                    self.geometry_manager.maximize_surface(id).collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::Unmaximize { id },
                } => {
                    log::info!(target: "WCompRequest","Surface {} unmaximized",id);
                    self.geometry_manager.unmaximize_surface(id).collect::<Vec<_>>()
                },
                WCompRequest::Surface {
                    request: SurfaceRequest::Commit { id },
                } => {
                    log::info!(target: "WCompRequest","Surface {} committed",id);
                    self.geometry_manager.commit_surface(id).collect::<Vec<_>>()
                },
                _ => Vec::new(),
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
