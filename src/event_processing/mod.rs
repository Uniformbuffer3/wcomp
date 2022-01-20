//! Module containing events processing functions.

mod platform_events;
mod wayland_requests;
mod wcomp_events;
mod wcomp_requests;

use crate::geometry_manager;
use crate::wcomp::WComp;
use pal::PlatformBackend;
use std::fmt::Debug;

#[derive(Debug)]
/// Possible messages of [WComp][WComp].
pub enum WCompMessage {
    Platform(pal::Event),
    Wayland(ews::WaylandRequest),
    Geometry(geometry_manager::WCompRequest),
}
impl From<pal::Event> for WCompMessage {
    fn from(message: pal::Event) -> Self {
        Self::Platform(message)
    }
}
impl From<ews::WaylandRequest> for WCompMessage {
    fn from(message: ews::WaylandRequest) -> Self {
        Self::Wayland(message)
    }
}
impl From<geometry_manager::WCompRequest> for WCompMessage {
    fn from(message: geometry_manager::WCompRequest) -> Self {
        Self::Geometry(message)
    }
}

impl WComp {
    /// Gather all the events and requests and process them.
    pub(crate) fn process_messages(&mut self) -> bool {
        let async_requests = self
            .async_requests
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>();
        let platform_requests = self.platform.events().into_iter();
        let wayland_requests = self.ews.dispatch().into_iter();

        let requests = std::iter::empty()
            .chain(async_requests)
            .chain(self.process_platform_requests(platform_requests))
            .chain(self.process_wayland_requests(wayland_requests));

        let events = self.process_requests(requests);
        let redraw = self.process_events(events);
        redraw
    }
}
