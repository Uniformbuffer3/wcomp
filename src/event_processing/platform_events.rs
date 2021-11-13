
use crate::wcomp::WComp;
use pal::PlatformBackend;
use crate::geometry_manager::{GeometryEvent,OutputEvent,CursorEvent};
use crate::event_processing::WCompMessage;

impl WComp {
    pub(crate) fn process_platform_event(&mut self, message: pal::Event)->bool{
        let mut redraw = false;
        match message {
            pal::Event::Output{time,id,event} => {
                match &event {
                    pal::OutputEvent::Added(_)=>{
                        if self.platform.platform_type() == pal::PlatformType::Direct {
                            self.platform.request(vec![pal::definitions::Request::Surface{request: pal::definitions::SurfaceRequest::Create(Some(id))}]);
                        }
                    }
                    pal::OutputEvent::Removed=>{

                    }
                    _=>{
                    }
                }
            },
            pal::Event::Surface{time,id,event} => {
                match &event {
                    pal::SurfaceEvent::Added(surface_info) => {
                        if let pal::definitions::Surface::WGpu(surface) = &surface_info.surface {
                            let size = surface_info.size;
                            self.wgpu_engine.create_surface(
                                id.into(),
                                String::from("MainSurface"),
                                surface.clone(),
                                size.width,
                                size.height,
                            );
                            let id = id.into();
                            let serial = ews::SERIAL_COUNTER.next_serial().into();
                            let handle = ();
                            let event = GeometryEvent::Output{serial,event: OutputEvent::Added{id,handle,size}};
                            self.messages.borrow_mut().push(WCompMessage::from(event));
                            //self.geometry_manager.add_output(id.into(), (), surface_info.size);
                        } else {
                            panic!("It is not of WGpu type");
                        }
                        redraw = true;
                    },
                    pal::SurfaceEvent::Resized(size) => {
                        self.wgpu_engine.resize_surface(id.into(),size.width,size.height);
                        let id = id.into();
                        let serial = ews::SERIAL_COUNTER.next_serial().into();
                        let size = *size;
                        let event = GeometryEvent::Output{serial,event: OutputEvent::Resized{id,size}};
                        self.messages.borrow_mut().push(WCompMessage::from(event));
                        redraw = true;
                    },
                    pal::SurfaceEvent::Removed => {
                        self.wgpu_engine.destroy_surface(id.into());

                        let id = id.into();
                        let serial = ews::SERIAL_COUNTER.next_serial().into();
                        let event = GeometryEvent::Output{serial,event: OutputEvent::Removed{id}};
                        self.messages.borrow_mut().push(WCompMessage::from(event));
                        redraw = true;
                    },
                    _ => (),
                }
            }
            pal::Event::Seat{time,id,event}=>{
                match event {
                    pal::SeatEvent::Added{name}=>{
                        self.ews.create_seat(id.into(),name);
                    }
                    pal::SeatEvent::Keyboard(pal::KeyboardEvent::Added(_keyboard_info))=>{
                        self.ews.add_keyboard(id.into(),200, 25);
                    }
                    pal::SeatEvent::Keyboard(pal::KeyboardEvent::Removed)=>{
                        self.ews.del_keyboard(id.into());
                    }
                    pal::SeatEvent::Keyboard(pal::KeyboardEvent::Key {code,key: _,state,serial,time})=>{
                        self.ews.get_keyboard(id.into()).map(|keyboard|{
                            let keystate = match state {
                                pal::State::Down=>ews::KeyState::Pressed,
                                pal::State::Up=>ews::KeyState::Released
                            };
                            keyboard.input::<(),_>(
                                code,
                                keystate,
                                serial.into(),
                                time,
                                |_modifier,_keysim|{ews::FilterResult::Forward}
                            );
                        });
                    },
                    pal::SeatEvent::Keyboard(pal::KeyboardEvent::AutoRepeat{rate: _,delay: _})=>{

                    },
                    pal::SeatEvent::Keyboard(pal::KeyboardEvent::LayoutModified {layout: _})=>{
                    },
                    pal::SeatEvent::Cursor(pal::CursorEvent::Added(info))=>{
                        self.ews.add_cursor(id.into());
                        let size = self.geometry_manager.get_cursor_size();
                        let position = pal::Position2D::from(self.geometry_manager.get_surface_optimal_position(&size));
                        let handle = ();
                        let image = None;

                        let id = id.into();
                        let serial = ews::SERIAL_COUNTER.next_serial().into();
                        let event = GeometryEvent::Cursor{serial,event: CursorEvent::Added{id,handle,position,image}};
                        self.messages.borrow_mut().push(WCompMessage::from(event));
                    }
                    pal::SeatEvent::Cursor(pal::CursorEvent::Removed)=>{
                        self.ews.del_cursor(id.into());
                        let id = id.into();
                        let serial = ews::SERIAL_COUNTER.next_serial().into();
                        let event = GeometryEvent::Cursor{serial,event: CursorEvent::Removed{id}};
                        self.messages.borrow_mut().push(WCompMessage::from(event));
                    }
                    pal::SeatEvent::Cursor(pal::CursorEvent::Button{code,key,state})=>{
                        let id = id.into();
                        let serial = ews::SERIAL_COUNTER.next_serial().into();
                        let event = GeometryEvent::Cursor{serial,event: CursorEvent::Button{id,time,code,key,state}};
                        self.messages.borrow_mut().push(WCompMessage::from(event));
                    }
                    pal::SeatEvent::Cursor(pal::CursorEvent::Entered{surface_id,position})=>{
                        let id = id.into();
                        let serial = ews::SERIAL_COUNTER.next_serial().into();
                        let output_id = surface_id.into();
                        let event = GeometryEvent::Cursor{serial,event: CursorEvent::Entered{id,output_id}};
                        self.messages.borrow_mut().push(WCompMessage::from(event));
                    }
                    pal::SeatEvent::Cursor(pal::CursorEvent::Left{surface_id})=>{
                        let id = id.into();
                        let serial = ews::SERIAL_COUNTER.next_serial().into();
                        let output_id = surface_id.into();
                        let event = GeometryEvent::Cursor{serial,event: CursorEvent::Left{id,output_id}};
                        self.messages.borrow_mut().push(WCompMessage::from(event));
                    }
                    pal::SeatEvent::Cursor(pal::CursorEvent::AbsoluteMovement{position})=>{
                        self.geometry_manager.relative_move_cursor(id.into(),position).map(|position|{
                            let id = id.into();
                            let serial = ews::SERIAL_COUNTER.next_serial().into();
                            let event = GeometryEvent::Cursor{serial,event: CursorEvent::Moved{id,position}};
                            self.messages.borrow_mut().push(WCompMessage::from(event));
                        });
                    }
                    _=>{}
                }
            }
            _ => (),
        }
        redraw
    }
}
