use crate::wcomp::WComp;
use crate::geometry_manager::{GeometryEvent,OutputEvent,SurfaceEvent,CursorEvent};
use screen_task::ScreenTask;

impl WComp {
    pub fn process_geometry_event(&mut self, message: GeometryEvent<(),ews::WlSurface,()>)->bool{
        let mut redraw = false;
        match message {
            GeometryEvent::Output{serial,event: OutputEvent::Added{id, handle, size}}=>{
                //TODO Aggiungere output al surface_manager
                self.geometry_manager.add_output(id, handle, size);
            },
            GeometryEvent::Output{serial,event: OutputEvent::Removed{id}}=>{
                //TODO Rimuovere output al surface_manager
                self.geometry_manager.del_output(id);
            }
            GeometryEvent::Output{serial,event: OutputEvent::Resized{id,size}}=>{
                //TODO Rimuovere output al surface_manager
                self.geometry_manager.resize_output(id, size);
            }
            GeometryEvent::Output{serial,event: OutputEvent::Moved{old, new_position}}=>{
                //TODO Muovere output nel surface_manager
            }
            GeometryEvent::Cursor{serial,event: CursorEvent::Added{id,handle,position,image}}=>{
                //TODO Aggiungere cursor al surface_manager
                self.geometry_manager.add_cursor(id, handle, position,image);
            },
            GeometryEvent::Cursor{serial,event: CursorEvent::Removed{id}}=>{
                //TODO Rimuovere cursor al surface_manager
                self.geometry_manager.del_cursor(id)
            },
            GeometryEvent::Cursor{serial,event: CursorEvent::Moved{id,position}}=>{
                self.ews.get_cursor(id).map(|cursor_handle|{
                    let focus = self.geometry_manager.get_surface_at(position).map(|surface|{
                        println!("Focus: {:#?}",surface.handle);
                        /*
                        let area = pal::Rectangle::from((*surface.position(),*surface.current_size()));
                        area.relative_to(position).map(|relative_position|{
                            let relative_position: (i32,i32) = relative_position.into();
                            (surface.handle().clone(),relative_position.into())
                        })
                        */
                        let position: (i32,i32) = surface.geometry.position.into();
                        (surface.handle().clone(),position.into())
                    });
                    let position = (position.x as f64,position.y as f64).into();
                    let serial = serial.into();
                    let time = self.timer.elapsed().as_millis() as u32;
                    //println!("Position: {:#?}",position);
                    //println!("Focus: {:#?}",focus);
                    //println!("Serial: {:#?}",serial);
                    //println!("Time: {:#?}",time);
                    cursor_handle.motion(position, focus, serial, time);
                });
                redraw = true;
            },
            GeometryEvent::Cursor{serial,event: CursorEvent::Button{id,time,code,key: _,state}}=>{
                self.ews.get_cursor(id).map(|cursor|{
                    let state = match state {
                        pal::State::Down=>ews::ButtonState::Pressed,
                        pal::State::Up=>ews::ButtonState::Released
                    };
                    cursor.button(code, state, serial.into(), time);
                });
            },
            GeometryEvent::Cursor{serial,event: CursorEvent::Focus{id}}=>{
                //TODO Rimuovere cursor al surface_manager
            },
            GeometryEvent::Cursor{serial,event: CursorEvent::Entered{id,output_id}}=>{
                self.geometry_manager.enter_cursor(id, output_id);
            },
            GeometryEvent::Cursor{serial,event: CursorEvent::Left{id,output_id}}=>{
                self.geometry_manager.left_cursor(id,output_id);
            },
            GeometryEvent::Surface{serial,event: SurfaceEvent::Added{id,handle,inner_geometry,geometry}}=>{
                self.geometry_manager.add_surface(id,handle,inner_geometry,geometry);
            },
            GeometryEvent::Surface{serial,event: SurfaceEvent::Removed(surface)}=>{
                self.geometry_manager.surfaces_ref().for_each(|surface|println!("{:#?}",surface));
                redraw = true;
            },
            GeometryEvent::Surface{serial,event: SurfaceEvent::Moved{id,position,depth}}=>{
                self.wgpu_engine.task_handle_cast_mut(&self.screen_task, |screen_task: &mut ScreenTask|{
                    screen_task.move_surface(id, [position.x,position.y,depth as i32]);
                });
                self.geometry_manager.move_surface(id, position);
                redraw = true;
            },
            _=>()
        }
        redraw
    }
}
