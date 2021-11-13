mod geometry_events;
mod platform_events;
mod wayland_requests;

use crate::geometry_manager;
use crate::wcomp::WComp;
use pal::PlatformBackend;
use std::fmt::Debug;

#[derive(Debug)]
pub enum WCompMessage<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> {
    Platform(pal::Event),
    Wayland(ews::WaylandRequest),
    Geometry(geometry_manager::GeometryEvent<C,S,O>)
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> From<pal::Event> for WCompMessage<C,S,O> {
    fn from(message: pal::Event)->Self {
        Self::Platform(message)
    }
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> From<ews::WaylandRequest> for WCompMessage<C,S,O> {
    fn from(message: ews::WaylandRequest)->Self {
        Self::Wayland(message)
    }
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> From<geometry_manager::GeometryEvent<C,S,O>> for WCompMessage<C,S,O> {
    fn from(message: geometry_manager::GeometryEvent<C,S,O>)->Self {
        Self::Geometry(message)
    }
}




impl WComp {
    pub(crate) fn process_messages(&mut self)->bool{
        let mut redraw = false;
        {
            let mut messages = self.messages.borrow_mut();
            messages.append(&mut self.platform.events().into_iter().map(WCompMessage::from).collect());
            messages.append(&mut self.ews.dispatch(std::time::Duration::from_secs(1)).into_iter().map(WCompMessage::from).collect());
            messages.append(&mut self.geometry_manager.events().map(WCompMessage::from).collect());
        }

        while !self.messages.borrow().is_empty(){
            let messages: Vec<_> = self.messages.borrow_mut().drain(..).collect();
            for message in messages {
                redraw |= match message {
                    WCompMessage::Platform(message)=>self.process_platform_event(message),
                    WCompMessage::Wayland(message)=>self.process_wayland_request(message),
                    WCompMessage::Geometry(message)=>self.process_geometry_event(message)
                };
            }
        }

        redraw
    }
}
