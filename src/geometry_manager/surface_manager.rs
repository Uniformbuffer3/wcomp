use std::cmp::Ordering;
use std::collections::VecDeque;

bitflags::bitflags!{
    pub struct SurfaceState: u32 {
        const MAXIMIZED     = (1 << 0);
        const FULLSCREEN    = (1 << 1);
        const TILED_LEFT     = (1 << 2);
        const TILED_RIGHT    = (1 << 3);
        const TILED_TOP      = (1 << 4);
        const TILED_BOTTOM   = (1 << 5);
    }
}

#[derive(Debug,Clone)]
pub enum SurfaceEvent<S: Clone> {
    Moved{id: usize,position: pal::Position2D<i32>,depth: u32},
    Resized{id: usize,size: pal::Size2D<u32>},
    Removed(Surface<S>),
    Added{
        id: usize,
        handle: S,
        inner_geometry: pal::Rectangle<i32,u32>,
        geometry: pal::Rectangle<i32,u32>,
    }
}


#[derive(Debug,Clone)]
pub struct Surface<S: Clone> {
    pub id: usize,
    pub handle: S,
    pub inner_geometry: pal::Rectangle<i32,u32>,
    pub geometry: pal::Rectangle<i32,u32>,
    pub depth: u32,
    pub state: SurfaceState
}
impl<S: Clone> Surface<S> {
    pub fn id(&self)->usize {self.id}
    pub fn handle(&self)->&S {&self.handle}
    pub fn geometry(&self)->&pal::Rectangle<i32,u32> {&self.geometry}
    pub fn inner_geometry(&self)->&pal::Rectangle<i32,u32> {&self.inner_geometry}
    pub fn depth(&self)->u32 {self.depth}
    pub fn state(&self)->SurfaceState {self.state}

    pub fn update_position(&mut self,position: pal::Position2D<i32>){
        self.geometry.position = position;
        self.inner_geometry.position.x = std::cmp::max(self.inner_geometry.position.x,self.geometry.position.x);
        self.inner_geometry.position.y = std::cmp::max(self.inner_geometry.position.y,self.geometry.position.y);
    }
    pub fn update_size(&mut self,size: pal::Size2D<u32>){
        self.geometry.size = size;
        self.inner_geometry.size.width = std::cmp::min(self.inner_geometry.size.width,self.geometry.size.width);
        self.inner_geometry.size.height = std::cmp::min(self.inner_geometry.size.height,self.geometry.size.height);
    }
    pub fn update_geometry(&mut self,geometry: pal::Rectangle<i32,u32>){
        self.update_position(geometry.position);
        self.update_size(geometry.size);
    }
    pub fn resize(&mut self, size: pal::Size2D<u32>, edge: ews::ResizeEdge)->impl Iterator<Item=SurfaceEvent<S>>{
        /*
        match edge {
            ews::ResizeEdge::Top=>{
                self.size.
            }
            _=>()
        }
        */
        Vec::new().into_iter()
    }
}
impl<S: Clone> Ord for Surface<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth.cmp(&other.depth)
    }
}
impl<S: Clone> PartialOrd for Surface<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.depth.cmp(&other.depth))
    }
}

impl<S: Clone> PartialEq for Surface<S> {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth
    }
}
impl<S: Clone> Eq for Surface<S> {}

#[derive(Debug)]
pub struct SurfaceManager<S: Clone>{
    cursor_surfaces: VecDeque<Surface<S>>,
    surfaces: VecDeque<Surface<S>>,
}
impl<S: Clone> SurfaceManager<S> {
    const CURSOR_MIN_DEPTH: u32 = 0;
    const SURFACE_MIN_DEPTH: u32 = 16;
    pub fn new()->Self {
        let cursor_surfaces = VecDeque::new();
        let surfaces = VecDeque::new();
        Self{cursor_surfaces,surfaces}
    }

    pub fn get_surface_at(&mut self, position: pal::Position2D<i32>)->Option<&Surface<S>> {
        self.surfaces.iter().find(|surface|{
            surface.geometry().contains(position)
        })
    }

    pub fn add_cursor_surface(&mut self, id: usize, handle: S, inner_geometry: pal::Rectangle<i32,u32>, geometry: pal::Rectangle<i32,u32>)->impl Iterator<Item=SurfaceEvent<S>>+Clone{
        let state = SurfaceState::empty();
        let depth = 0;
        self.surfaces.push_front(Surface {id,handle,inner_geometry,geometry,depth,state});
        self.update_depth()
    }

    pub fn add_surface(&mut self, id: usize, handle: S, inner_geometry: pal::Rectangle<i32,u32>, geometry: pal::Rectangle<i32,u32>)->impl Iterator<Item=SurfaceEvent<S>>+Clone{
        let state = SurfaceState::empty();
        let depth = 0;
        self.surfaces.push_front(Surface {id,handle,inner_geometry,geometry,depth,state});
        self.update_surfaces_depth()
    }
    pub fn del_surface(&mut self,id: usize)->impl Iterator<Item=SurfaceEvent<S>>+Clone{
        if let Some(position) = self.surfaces.iter().position(|surface|surface.id == id){
            let events: Vec<_> = self.surfaces.remove(position)
            .map(|surface|SurfaceEvent::Removed(surface))
            .into_iter()
            .chain(self.update_surfaces_depth())
            .collect();

            events.into_iter()
        }
        else{Vec::new().into_iter()}
    }
    pub fn move_surface(&mut self, id: usize, position: pal::Position2D<i32>)->impl Iterator<Item=SurfaceEvent<S>>+Clone{
        self.surface_mut(id).map(|surface|{
            surface.geometry.position = position;
            SurfaceEvent::Moved{
                id,
                position: surface.geometry().position,
                depth: surface.depth
            }
        }).into_iter()
    }
    pub fn resize_surface(&mut self, id: usize, size: pal::Size2D<u32>,edge: ews::ResizeEdge)->impl Iterator<Item=SurfaceEvent<S>>+Clone{
        unimplemented!();
        Vec::new().into_iter()
    }


    pub fn surfaces_ref(&self)->impl Iterator<Item=&Surface<S>>{self.surfaces.iter()}
    pub fn surfaces_mut(&mut self)->impl Iterator<Item=&mut Surface<S>>{self.surfaces.iter_mut()}

    pub fn surface_ref(&self,id: usize)->Option<&Surface<S>> {
        self.surfaces.iter().find(|surface|surface.id == id)
    }
    pub fn surface_mut(&mut self,id: usize)->Option<&mut Surface<S>> {
        self.surfaces.iter_mut().find(|surface|surface.id == id)
    }

    fn update_depth(&mut self)->impl Iterator<Item=SurfaceEvent<S>>+Clone {
        std::iter::empty()
        .chain(self.update_cursor_surfaces_depth())
        .chain(self.update_surfaces_depth())
        //events.into_iter()
    }

    fn update_cursor_surfaces_depth(&mut self)->impl Iterator<Item=SurfaceEvent<S>>+Clone {
        self.cursor_surfaces.iter_mut().enumerate().map(|(index,surface)|{
            surface.depth = index as u32;

            let id = surface.id;
            let position = (surface.geometry().position).into();
            let depth = surface.depth;
            SurfaceEvent::Moved{id,position,depth}
        }).collect::<Vec<_>>().into_iter()
    }
    fn update_surfaces_depth(&mut self)->impl Iterator<Item=SurfaceEvent<S>>+Clone {
        let depth_offset = self.cursor_surfaces.len();
        self.surfaces.iter_mut().enumerate().map(|(index,surface)|{
            surface.depth = (index + depth_offset) as u32;

            let id = surface.id;
            let position = (surface.geometry().position).into();
            let depth = surface.depth;
            SurfaceEvent::Moved{id,position,depth}
        }).collect::<Vec<_>>().into_iter()
    }
}



/*
pub struct GeometryManager {
    outputs: HashMap<usize,Rectangle<u32>>,


    //size: Size2D<u32>,
    //window_space: Rectangle<u32>
}
impl GeometryManager {
    pub fn new()->Self {
        let outputs = HashMap::new();
        let surfaces = HashMap::new();
        let surface_stack = BTreeSet::new();
        //let window_space = Rectangle {};
        Self {outputs,surfaces,surface_stack}
    }


    pub fn add_surface(&mut self, id: usize, size: Size2D<u32>, reserve_space: bool){
        let surface = Surface {position,size,reserve_space};

        if reserve_space {
            if position.y + size.height > self.window_space.position.y {
            }
        }

        self.surfaces.insert(id,surface);
        self.surface_stack.push(id);
    }
    pub fn del_surface(&mut self, id: usize){
        self.surfaces.remove(&id);
        if let Some(index) = self.surface_stack.iter().position(|current_id|current_id == &id){
            self.surface_stack.remove(index);
        }
    }

    pub fn put_on_top(&mut self,id: usize){
        if let Some(index) = self.surface_stack.iter().position(|current_id|current_id == &id){
            let id = self.surface_stack.remove(index);
            self.surface_stack.push(id);
        }
    }

    pub fn cursor_movement(&self, id: SeatId, old_position: Position2D<u32>, new_position: Position2D<u32>)->Vec<Event> {
        let mut events = Vec::new();
        let surface_old = self.surface(old_position);
        let surface_new = self.surface(old_position);

        match (surface_old,surface_new){
            (Some(id1),Some(id2))=>{
                if id1 != id2 {
                    let surface_id = id1.into();
                    let event = SeatEvent::Cursor(CursorEvent::Left{surface_id});
                    events.push(Event::Seat{id,event});

/*
                    let old_position = Point::new(old_position.x as f32,old_position.y as f32);
                    let new_position = Point::new(new_position.x as f32, new_position.y as f32);
                    let segment = Segment::new(old_position,new_position);

                    let ray = Ray::new(old_position,*segment.direction().unwrap());
                    let result = self.shape.cast_ray_and_get_normal(
                        &Isometry::translation(0.0,0.0),
                        &ray,
                        1000000.0,
                        false
                    ).unwrap();
                    let point = ray.point_at(result.toi);

                    //Position2D{x: point.coords.x as u32,y: point.coords.y as u32}
*/


                    let surface_id = id2.into();
                    let position = new_position;
                    let event = SeatEvent::Cursor(CursorEvent::Entered{surface_id,position});
                    events.push(Event::Seat{id,event});
                }

                let position = new_position;
                let event = SeatEvent::Cursor(CursorEvent::AbsoluteMovement{position});
                events.push(Event::Seat{id,event});
            }
            _=>{}
        }

        events
    }

    pub fn surface(&self,position: Position2D<u32>)->Option<usize> {
        for surface_id in &self.surface_stack {
            if let Some(surface) = self.surfaces.get(surface_id){
                if surface.contains(position){return Some(*surface_id);}
            }
        }
        None
    }

}
*/
