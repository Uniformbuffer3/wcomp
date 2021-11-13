mod surface_manager;
pub use surface_manager::{SurfaceManager,SurfaceEvent,Surface};

mod output_manager;
pub use output_manager::{OutputManager,OutputEvent};

mod cursor_manager;
pub use cursor_manager::{CursorManager,CursorEvent};

//mod seat_manager;
//pub use seat_manager::{SeatManager,SeatEvent};

use std::fmt::Debug;

pub enum TmpGeometryEvent<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> {
    Cursor(CursorEvent<C>),
    Surface(SurfaceEvent<S>),
    Output(OutputEvent<O>)
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> From<CursorEvent<C>> for TmpGeometryEvent<C,S,O> {
    fn from(event: CursorEvent<C>) -> Self {
        Self::Cursor(event)
    }
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> From<SurfaceEvent<S>> for TmpGeometryEvent<C,S,O> {
    fn from(event: SurfaceEvent<S>) -> Self {
        Self::Surface(event)
    }
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> From<OutputEvent<O>> for TmpGeometryEvent<C,S,O> {
    fn from(event: OutputEvent<O>) -> Self {
        Self::Output(event)
    }
}

#[derive(Debug)]
pub enum GeometryEvent<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> {
    Cursor{serial: u32, event: CursorEvent<C>},
    Surface{serial: u32, event: SurfaceEvent<S>},
    Output{serial: u32, event: OutputEvent<O>}
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> GeometryEvent<C,S,O> {
    pub fn serial(&self)->u32 {
        match self {
            Self::Cursor{serial,event:_}=>*serial,
            Self::Surface{serial,event:_}=>*serial,
            Self::Output{serial,event:_}=>*serial,
        }
    }
}
impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> From<(u32,TmpGeometryEvent<C,S,O>)> for GeometryEvent<C,S,O> {
    fn from(tuple: (u32,TmpGeometryEvent<C,S,O>)) -> Self {
        match tuple {
            (serial,TmpGeometryEvent::Cursor(event))=>Self::Cursor{serial,event},
            (serial,TmpGeometryEvent::Surface(event))=>Self::Surface{serial,event},
            (serial,TmpGeometryEvent::Output(event))=>Self::Output{serial,event},
        }
    }
}
/*
impl<S: Clone + Debug> From<(u32,CursorEvent)> for GeometryEvent<S> {
    fn from(tuple: (u32,CursorEvent)) -> Self {
        let serial = tuple.0;
        let event = tuple.1;
        Self::Cursor{serial,event}
    }
}
impl<S: Clone + Debug> From<(u32,SurfaceEvent<S>)> for GeometryEvent<S> {
    fn from(tuple: (u32,SurfaceEvent<S>)) -> Self {
        let serial = tuple.0;
        let event = tuple.1;
        Self::Surface{serial,event}
    }
}
impl<S: Clone + Debug> From<(u32,OutputEvent)> for GeometryEvent<S> {
    fn from(tuple: (u32,OutputEvent)) -> Self {
        let serial = tuple.0;
        let event = tuple.1;
        Self::Output{serial,event}
    }
}
*/

#[derive(Debug)]
pub struct GeometryManager<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> {
    cursor_manager: CursorManager<C>,
    surface_manager: SurfaceManager<S>,
    output_manager: OutputManager<O>,
    events: Vec<GeometryEvent<C,S,O>>,
    //postprocess_callback: Option<Box<dyn FnMut(GeometryEvent)>>
}

impl<C: Clone + Debug,S: Clone + Debug,O: Clone + Debug> GeometryManager<C,S,O> {
    pub fn new()->Self {
        let cursor_manager = CursorManager::new();
        let surface_manager = SurfaceManager::new();
        let output_manager = OutputManager::new();
        let events = Vec::new();
        Self {cursor_manager,surface_manager,output_manager,events}
    }

    pub fn events(&mut self)->impl Iterator<Item=GeometryEvent<C,S,O>> +'_{
        self.events.drain(..)
    }

    pub fn get_cursor_size(&self)->pal::Size2D<u32> {self.cursor_manager.get_cursor_size()}
    pub fn get_surface_at(&mut self, position: pal::Position2D<i32>)->Option<&Surface<S>> {self.surface_manager.get_surface_at(position)}

    pub fn enter_cursor(&mut self, id: usize, output_id: usize){
    /*
        self.cursor_manager.cursor_ref(id).map(|cursor|{
            cursor.
        })
        self.surface_manager.add_cursor_surface(id, handle, offset, space);
    */
        log::info!(target:"WComp","Geometry manager | Cursor entered");
        let events = self.cursor_manager.enter_cursor(id, output_id).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }
    pub fn left_cursor(&mut self, id: usize, output_id: usize){
        log::info!(target:"WComp","Geometry manager | Cursor left");
        let events = self.cursor_manager.left_cursor(id, output_id).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    pub fn add_cursor(&mut self, id: usize, handle: C, position: pal::Position2D<i32>, image: Option<usize>){
        log::info!(target:"WComp","Geometry manager | Cursor added");
        let events = self.cursor_manager.add_cursor(id, handle, position,image).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    pub fn del_cursor(&mut self, id: usize){
        log::info!(target:"WComp","Geometry manager | Cursor removed");
        let events = self.cursor_manager.del_cursor(id).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }
/*
    pub fn relative_move_cursor(&mut self, id: usize, position: pal::Position2D<i32>){
        let events = self.cursor_manager.cursor_ref(id).map(|cursor|cursor.output()).flatten().map(|output_id|{
            self.output_manager.relative_to_absolute(output_id,position).map(|absolute_position|{
                self.cursor_manager.move_cursor(id, absolute_position)
            })
        }).flatten().into_iter().flatten().map(TmpGeometryEvent::from);

        self.postprocess_events(events);
    }
*/
    pub fn relative_move_cursor(&mut self, id: usize, position: pal::Position2D<i32>)->Option<pal::Position2D<i32>> {
        self.cursor_manager.cursor_ref(id).map(|cursor|cursor.output()).flatten().map(|output_id|{
            self.output_manager.relative_to_absolute(output_id,position)
        }).flatten()
    }

    pub fn absolute_move_cursor(&mut self, id: usize, position: pal::Position2D<i32>){
        log::info!(target:"WComp","Geometry manager | Cursor moved");
        let events = self.cursor_manager.move_cursor(id, position).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    pub fn add_output(&mut self, id: usize, handle: O, size: pal::Size2D<u32>){
        log::info!(target:"WComp","Geometry manager | Output added");
        let events = self.output_manager.add_output(id, handle, size).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }
    pub fn del_output(&mut self, id: usize){
        log::info!(target:"WComp","Geometry manager | Output removed");
        let events = self.output_manager.del_output(id).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }
    pub fn resize_output(&mut self, id: usize, size: pal::Size2D<u32> ){
        log::info!(target:"WComp","Geometry manager | Output resized");
        let events = self.output_manager.resize_output(id,size).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    pub fn get_surface_optimal_size(&self)->pal::Size2D<u32>{
        self.output_manager.get_surface_optimal_size()
    }

    pub fn get_surface_optimal_position(&self, size: &pal::Size2D<u32>)->pal::Position3D<i32>{
        pal::Position3D::from((self.output_manager.get_surface_optimal_position(size),0))
    }

    pub fn surfaces_ref(&self)->impl Iterator<Item=&Surface<S>> {
        self.surface_manager.surfaces_ref()
    }
    pub fn surface_ref(&self,id: usize)->Option<&Surface<S>> {
        self.surface_manager.surface_ref(id)
    }

    pub fn add_surface(&mut self, id: usize, handle: S, inner_geometry: pal::Rectangle<i32,u32>, geometry: pal::Rectangle<i32,u32>,) {
        log::info!(target:"WComp","Geometry manager | Surface added - inner_geometry: {:#?}, geometry: {:#?}",inner_geometry,geometry);
        let events = self.surface_manager.add_surface(id, handle, inner_geometry,geometry).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    pub fn del_surface(&mut self, id: usize) {
        log::info!(target:"WComp","Geometry manager | Surface removed");
        let events = self.surface_manager.del_surface(id).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    pub fn resize_surface(&mut self, id: usize, size: pal::Size2D<u32>, edge: ews::ResizeEdge) {
        log::info!(target:"WComp","Geometry manager | Surface resized");
        let events = self.surface_manager.resize_surface(id, size, edge).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    pub fn move_surface(&mut self, id: usize, position: pal::Position2D<i32>) {
        log::info!(target:"WComp","Geometry manager | Surface moved");
        let events = self.surface_manager.move_surface(id, position).map(TmpGeometryEvent::from);
        self.postprocess_events(events);
    }

    fn postprocess_events(&mut self, events: impl Iterator<Item=TmpGeometryEvent<C,S,O>>){
        for tmp_event in events {
            let serial: u32 = ews::SERIAL_COUNTER.next_serial().into();
            let event = GeometryEvent::from((serial,tmp_event));
            match &event {
                GeometryEvent::Output{serial: _,event: OutputEvent::Removed{id}}=>{
                    let screen_size = self.output_manager.screen_size();
                    let surfaces = self.surface_manager.surfaces_mut().filter_map(|surface|{
                        if screen_size.contains(surface.geometry().position.into()){Some(surface)}
                        else{None}
                    });

                    for surface in surfaces {
                        surface.update_position(self.output_manager.get_surface_optimal_position(&surface.geometry().size));
                        //TODO Missing propagate resize to client
                    };
                }
                GeometryEvent::Output{serial: _,event: OutputEvent::Moved{old, new_position}}=>{
                    let surfaces = self.surface_manager.surfaces_mut().filter_map(|surface|{
                        if old.geometry.contains(surface.geometry().position.into()){Some(surface)}
                        else{None}
                    });

                    for surface in surfaces {
                        surface.update_position(self.output_manager.get_surface_optimal_position(&surface.geometry().size));
                        //TODO Missing propagate resize to client
                    }
                }
                _=>()
            }
            //self.events.push(event);
        }
    }

    fn reposition_surfaces(&mut self){

    }
}


