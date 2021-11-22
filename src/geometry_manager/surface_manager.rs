use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum SurfaceRequest {
    Added {
        id: usize,
        kind: SurfaceKind,
    },
    Removed {
        id: usize,
    },
    Moved {
        id: usize,
        position: pal::Position2D<i32>,
        depth: u32,
    },
    InteractiveResizeStart {
        id: usize,
        serial: u32,
        edge: ews::ResizeEdge
    },
    InteractiveResize {
        id: usize,
        serial: u32,
        inner_size: pal::Size2D<u32>
    },
    InteractiveResizeStop {
        id: usize,
        serial: u32
    },
    Resized {
        id: usize,
        size: pal::Size2D<u32>,
    },
    Configuration {
        id: usize,
        geometry: Option<pal::Rectangle<i32, u32>>,
        min_size: pal::Size2D<u32>,
        max_size: pal::Size2D<u32>,
    },
    BufferAttached {
        id: usize,
        handle: ews::WlBuffer,
        inner_geometry: pal::Rectangle<i32, u32>,
        size: pal::Size2D<u32>,
    },
    BufferDetached {
        id: usize,
    },
    Committed {
        id: usize,
    },
}

#[derive(Debug, Clone)]
pub enum SurfaceEvent {
    Added {
        id: usize,
        kind: SurfaceKind,
    },
    Removed {
        id: usize,
    },
    Moved {
        id: usize,
        position: pal::Position2D<i32>,
        depth: u32,
    },
    InteractiveResizeStarted {
        id: usize,
        serial: u32,
        edge: ews::ResizeEdge
    },
    InteractiveResizeStopped {
        id: usize,
        serial: u32
    },
    Resized {
        id: usize,
        size: pal::Size2D<u32>,
    },
    Configuration {
        id: usize,
        size: pal::Size2D<u32>,
    },
    MinSize {
        id: usize,
        size: pal::Size2D<u32>,
    },
    MaxSize {
        id: usize,
        size: pal::Size2D<u32>,
    },
    Geometry {
        id: usize,
        geometry: pal::Rectangle<i32,u32>,
    },
    BufferAttached {
        id: usize,
        handle: ews::WlBuffer,
        inner_geometry: pal::Rectangle<i32, u32>,
        geometry: pal::Rectangle<i32, u32>,
    },
    BufferReplaced {
        id: usize,
        handle: ews::WlBuffer,
        inner_geometry: pal::Rectangle<i32, u32>,
        geometry: pal::Rectangle<i32, u32>,
    },
    BufferDetached {
        id: usize,
    },
    Committed {
        id: usize,
    },
}
/*
bitflags::bitflags! {
    pub struct SurfaceState: u32 {
        const MAXIMIZED     = (1 << 0);
        const FULLSCREEN    = (1 << 1);
        const TILED_LEFT     = (1 << 2);
        const TILED_RIGHT    = (1 << 3);
        const TILED_TOP      = (1 << 4);
        const TILED_BOTTOM   = (1 << 5);
    }
}
*/
#[derive(Debug, Clone)]
pub struct AlteredState {
    original: pal::Rectangle<i32,u32>,
    minimized: bool,
    maximized: bool,
    fullscreen: bool,
    resizing: Option<(u32,ews::ResizeEdge)>, //Serial
    moving: Option<u32>, //Serial
}
impl AlteredState {
    pub fn new(original: pal::Rectangle<i32,u32>)->Self {
        let minimized = false;
        let maximized = false;
        let fullscreen = false;
        let resizing = None;
        let moving = None;
        Self {original,minimized,maximized,fullscreen,resizing,moving}
    }

    pub fn is_minimized(&self)->bool {self.minimized}
    pub fn is_maximized(&self)->bool {self.maximized}
    pub fn is_fullscreen(&self)->bool {self.fullscreen}
    pub fn is_resizing(&self)->bool {self.resizing.is_some()}
    pub fn is_resizing_width(&self, serial: u32)->bool {self.resizing.map(|(serial,_)|serial) == Some(serial)}
    pub fn is_moving(&self)->bool {self.moving.is_some()}
    pub fn is_moving_width(&self, serial: u32)->bool {self.moving == Some(serial)}
    pub fn is_empty(&self)->bool {
        !self.is_minimized() && !self.is_maximized() && !self.is_fullscreen() && !self.is_resizing() && !self.is_moving()
    }
    pub fn start_interactive_resize(&mut self, serial: u32, edge: ews::ResizeEdge)->bool{
        if self.resizing.is_some() | self.moving.is_some() | self.maximized | self.minimized {false}
        else{
            self.resizing = Some((serial,edge));
            true
        }
    }
    pub fn stop_interactive_resize(&mut self, serial: u32)->bool{
        if Some(serial) == self.resizing.map(|(serial,_)|serial) {
            self.resizing = None;
            true
        }
        else{false}
    }
    pub fn check_resize(&self, serial: u32)->bool{
        match &self.resizing {
            Some((current_serial,_))=>current_serial == &serial,
            None=>true
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SurfaceState {
    altered_state: Option<AlteredState>,
}
impl SurfaceState {
    pub fn check_resize(&self, serial: u32)->bool{
        self.altered_state.as_ref().map(|altered_state|altered_state.check_resize(serial)) != Some(false)
    }

    pub fn start_interactive_resize(&mut self, serial: u32, edge: ews::ResizeEdge, inner_geometry: pal::Rectangle<i32,u32>)->bool{
        println!("Current altered state: {:#?}",self.altered_state.clone());
        if self.altered_state.is_none() {self.altered_state = Some(AlteredState::new(inner_geometry));}
        let result = self.altered_state.as_mut().map(|altered_state|{
            altered_state.start_interactive_resize(serial,edge)
        });
        result == Some(true)
    }
    pub fn stop_interactive_resize(&mut self, serial: u32)->bool{
        let result = self.altered_state.as_mut().map(|altered_state|altered_state.stop_interactive_resize(serial));
        if self.is_empty() {self.altered_state = None;}
        result == Some(true)
    }

    pub fn is_minimized(&self)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_minimized()) == Some(true)}
    pub fn is_maximized(&self)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_maximized()) == Some(true)}
    pub fn is_fullscreen(&self)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_fullscreen()) == Some(true)}
    pub fn is_resizing(&self)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_resizing()) == Some(true)}
    pub fn is_resizing_width(&self, serial: u32)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_resizing_width(serial)) == Some(true)}
    pub fn is_moving(&self)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_moving()) == Some(true)}
    pub fn is_moving_width(&self, serial: u32)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_moving_width(serial)) == Some(true)}
    pub fn is_empty(&self)->bool {self.altered_state.as_ref().map(|altered_state|altered_state.is_empty()) == Some(true)}
}

#[derive(Debug, Clone)]
pub enum SurfaceKind {
    Toplevel {
        handle: ews::ToplevelSurface,
        state: SurfaceState,
    },
    Popup {
        handle: ews::PopupSurface,
    },
}
impl SurfaceKind {
    pub fn handle(&self) -> Option<&ews::WlSurface> {
        match self {
            Self::Toplevel { handle, .. } => handle.get_surface(),
            Self::Popup { handle } => handle.get_surface(),
        }
    }
    pub fn check_resize(&self, serial: u32)->bool{
        match self {
            Self::Toplevel{handle,state}=>state.check_resize(serial),
            Self::Popup{handle}=>true
        }
    }
}
impl From<ews::ToplevelSurface> for SurfaceKind {
    fn from(handle: ews::ToplevelSurface) -> Self {
        let state = SurfaceState::default();
        Self::Toplevel { handle, state }
    }
}
impl From<ews::PopupSurface> for SurfaceKind {
    fn from(handle: ews::PopupSurface) -> Self {
        Self::Popup { handle }
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub handle: ews::WlBuffer,
    pub geometry: pal::Rectangle<i32, u32>,
    //pub size: pal::Size2D<u32>
}
impl Buffer {
    pub fn geometry(&self) -> &pal::Rectangle<i32, u32> {
        &self.geometry
    }
    pub fn size(&self) -> pal::Size2D<u32> {
        use ews::Buffer;
        match ews::buffer_type(&self.handle) {
            Some(ews::BufferType::Shm) => ews::with_buffer_contents(&self.handle, |data, info| {
                pal::Size2D::from((info.width as u32, info.height as u32))
            })
            .unwrap(),
            Some(ews::BufferType::Dma) => self
                .handle
                .as_ref()
                .user_data()
                .get::<ews::Dmabuf>()
                .map(|dmabuf| pal::Size2D::from((dmabuf.width() as u32, dmabuf.height() as u32)))
                .unwrap(),
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Surface {
    pub id: usize,
    pub kind: SurfaceKind,
    pub buffer: Option<Buffer>,
    pub min_size: pal::Size2D<u32>,
    pub max_size: pal::Size2D<u32>,
    pub position: pal::Position2D<i32>,
    pub depth: u32,
}
impl Surface {
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn position(&self) -> &pal::Position2D<i32> {
        &self.position
    }
    pub fn depth(&self) -> u32 {
        self.depth
    }
    pub fn size(&self) -> Option<pal::Size2D<u32>> {
        self.buffer.as_ref().map(|buffer| buffer.size())
    }
    pub fn geometry(&self) -> Option<pal::Rectangle<i32, u32>> {
        self.size().map(|size|{
            pal::Rectangle::from((self.position().clone(),size))
        })
    }
    pub fn inner_geometry(&self) -> Option<&pal::Rectangle<i32, u32>> {
        self.buffer.as_ref().map(|buffer| buffer.geometry())
    }
    pub fn handle(&self) -> Option<&ews::WlSurface> {
        self.kind.handle()
    }
    pub fn buffer(&self) -> Option<&ews::WlBuffer> {
        self.buffer.as_ref().map(|buffer| &buffer.handle)
    }

    pub fn r#move(&mut self, position: pal::Position2D<i32>, depth: u32)->impl Iterator<Item=SurfaceEvent>+ Clone{
        self.position = position.clone();
        self.depth = depth;
        std::iter::once(SurfaceEvent::Moved {id: self.id,position,depth})
    }

/*
    pub fn resize(&mut self, size: pal::Size2D<u32>)->impl Iterator<Item=SurfaceEvent>+ Clone{
        if !self.kind.check_resize(serial){Vec::new().into_iter()}
        else{
            vec![SurfaceEvent::Resized{id: self.id,size}].into_iter()
        }
    }
*/
    pub fn configure(
        &mut self,
        id: usize,
        inner_geometry: Option<pal::Rectangle<i32, u32>>,
        min_size: pal::Size2D<u32>,
        max_size: pal::Size2D<u32>,
    )->impl Iterator<Item=SurfaceEvent>+ Clone {
        let mut events = Vec::new();
        if self.min_size != min_size {
            self.min_size = min_size.clone();
            events.push(SurfaceEvent::MinSize{id,size: min_size});
        }
        if self.max_size != max_size {
            self.max_size = max_size.clone();
            events.push(SurfaceEvent::MaxSize{id,size: max_size});
        }

        let kind = self.kind.clone();
        let depth = self.depth;
        let current_position = self.position().clone();
        let current_inner_geometry = self.inner_geometry();

        //if let (Some(geometry),current_position,Some(buffer)) = (self.geometry(),&mut self.position,self.buffer.as_mut()){

        if let (Some(inner_geometry),Some(current_inner_geometry)) = (inner_geometry.clone(),current_inner_geometry){
            match kind {
                SurfaceKind::Toplevel { handle, state }=>{
                    if let Some(altered_state) = state.altered_state.as_ref() {
                        match altered_state.resizing {
                            Some((_,edge))=>{
                                match edge {
                                    ews::ResizeEdge::Left => {
                                        let offset = current_inner_geometry.size.width as i32 - inner_geometry.size.width as i32;
                                        let new_position = current_position + pal::Offset2D::from((offset,0));
                                        events.append(&mut self.r#move(new_position, depth).collect());
                                    }
                                    ews::ResizeEdge::Top => {
                                        let offset = current_inner_geometry.size.height as i32 - inner_geometry.size.height as i32;
                                        let new_position = current_position + pal::Offset2D::from((0,offset));
                                        events.append(&mut self.r#move(new_position, depth).collect());
                                    }
                                    ews::ResizeEdge::TopLeft => {
                                        let offset_x = current_inner_geometry.size.width as i32 - inner_geometry.size.width as i32;
                                        let offset_y = current_inner_geometry.size.height as i32 - inner_geometry.size.height as i32;
                                        let new_position = current_position + pal::Offset2D::from((offset_x,offset_y));
                                        events.append(&mut self.r#move(new_position, depth).collect());
                                    }
                                    _=>()
                                }
                            },
                            _=>()
                        }
                    }
                }
                SurfaceKind::Popup { handle }=>(),
            }
        }
        if let (Some(buffer),Some(inner_geometry)) = (self.buffer.as_mut(),inner_geometry) {
            if buffer.geometry != inner_geometry {
                buffer.geometry = inner_geometry.clone();
                events.push(SurfaceEvent::Geometry{id,geometry: inner_geometry.clone()});
                events.push(SurfaceEvent::Resized {
                    id,
                    size: buffer.size(),
                });
            }
        }
        events.into_iter()

    }

    pub fn start_interactive_resize(&mut self,serial: u32,edge: ews::ResizeEdge)->impl Iterator<Item=SurfaceEvent>+ Clone{
        let geometry = self.geometry();
        match &mut self.kind {
            SurfaceKind::Toplevel{handle,state}=>{
                if let Some(geometry) = geometry {
                    if state.start_interactive_resize(serial, edge,geometry) {vec![SurfaceEvent::InteractiveResizeStarted{id: self.id,serial,edge}].into_iter()}
                    else{Vec::new().into_iter()}
                }
                else{Vec::new().into_iter()}
            }
            SurfaceKind::Popup{handle}=>Vec::new().into_iter()
        }
    }

    pub fn stop_interactive_resize(&mut self,serial: u32)->impl Iterator<Item=SurfaceEvent> + Clone{
        match &mut self.kind {
            SurfaceKind::Toplevel{handle,state}=>{
                if state.stop_interactive_resize(serial){
                    vec![SurfaceEvent::InteractiveResizeStopped{id: self.id,serial}].into_iter()
                }
                else{Vec::new().into_iter()}
            }
            SurfaceKind::Popup{handle}=>Vec::new().into_iter()
        }
    }

    pub fn maximize(&mut self){
        self.geometry().map(|geometry|{
            match &mut self.kind {
                SurfaceKind::Toplevel{handle,state}=>{
                    state.altered_state.get_or_insert(AlteredState::new(geometry)).maximized = true;
                },
                SurfaceKind::Popup {..}=>()
            }
        });
    }
    /*
    pub fn unmaximize(&mut self){
        self.geometry().map(|geometry|{
            match &mut self.kind {
                SurfaceKind::Toplevel{handle,state}=>{
                    let to_be_removed = state.altered_state.as_mut().map(|altered_state|{
                        altered_state.maximized = false;
                        if !(altered_state.maximized | altered_state.minimized | altered_state.fullscreen){
                            Some(altered_state.original.clone())
                        }
                        else{None}
                    });
                    if let Some(original) = {
                        self.position = original.position.clone();
                        self.buffer.as_mut().ap
                        state.altered_state = None;
                    }
                },
                SurfaceKind::Popup {..}=>()
            }
        });
    }
    */
    pub fn minimize(&mut self){
        self.geometry().map(|geometry|{
            match &mut self.kind {
                SurfaceKind::Toplevel{handle,state}=>{
                    state.altered_state.get_or_insert(AlteredState::new(geometry)).minimized = true;
                },
                SurfaceKind::Popup {..}=>()
            }
        });
    }
}
impl Ord for Surface {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth.cmp(&other.depth)
    }
}
impl PartialOrd for Surface {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.depth.cmp(&other.depth))
    }
}

impl PartialEq for Surface {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth
    }
}
impl Eq for Surface {}

#[derive(Debug)]
pub struct SurfaceManager {
    cursor_surfaces: VecDeque<Surface>,
    surfaces: VecDeque<Surface>,
}
impl SurfaceManager {
    const CURSOR_MIN_DEPTH: u32 = 0;
    const SURFACE_MIN_DEPTH: u32 = 16;
    pub fn new() -> Self {
        let cursor_surfaces = VecDeque::new();
        let surfaces = VecDeque::new();
        Self {
            cursor_surfaces,
            surfaces,
        }
    }

    pub fn get_surface_at(&mut self, position: &pal::Position2D<i32>) -> Option<&Surface> {
        self.surfaces.iter().find(|surface| {
            if let Some(buffer) = &surface.buffer {
                let adjusted_position = surface.position.clone() + buffer.geometry.position.clone();
                pal::Rectangle::from((adjusted_position, buffer.geometry.size.clone()))
                    .contains(&position)
                //pal::Rectangle::from((surface.position.clone(),buffer.size.clone())).contains(&position)
            } else {
                false
            }
        })
    }

    pub fn add_surface(
        &mut self,
        id: usize,
        kind: SurfaceKind,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let min_size = pal::Size2D::from((0, 0));
        let max_size = pal::Size2D::from((0, 0));
        let depth = 0;
        let position = pal::Position2D::from((0, 0));
        let buffer = None;

        let surface = Surface {
            id,
            kind: kind.clone(),
            buffer,
            min_size,
            max_size,
            position,
            depth,
        };
        self.surfaces.push_front(surface);

        std::iter::once(SurfaceEvent::Added { id, kind }).chain(self.update_surfaces_depth())
    }
    pub fn del_surface(&mut self, id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
        if let Some(position) = self.surfaces.iter().position(|surface| surface.id == id) {
            self.surfaces
                .remove(position)
                .map(|removed_surface| {
                    std::iter::empty()
                        .chain(
                            removed_surface
                                .buffer
                                .map(|_| SurfaceEvent::BufferDetached { id })
                                .into_iter(),
                        )
                        .chain(vec![SurfaceEvent::Removed { id }])
                        .chain(self.update_surfaces_depth())
                })
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            Vec::new().into_iter()
        }
    }

    pub fn attach_buffer(
        &mut self,
        id: usize,
        handle: ews::WlBuffer,
        inner_geometry: pal::Rectangle<i32, u32>,
        suggested_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| {
                let event = if let Some(mut buffer) = surface.buffer.as_mut() {
                    buffer.handle = handle.clone();
                    let geometry = pal::Rectangle::from((surface.position.clone(), buffer.size()));
                    SurfaceEvent::BufferReplaced {
                        id,
                        handle,
                        inner_geometry,
                        geometry,
                    }
                } else {
                    surface.buffer = Some(Buffer {
                        handle: handle.clone(),
                        geometry: inner_geometry.clone(),
                    }); //,size: suggested_size.clone()
                    let geometry = pal::Rectangle::from((surface.position.clone(), suggested_size));
                    SurfaceEvent::BufferAttached {
                        id,
                        handle,
                        inner_geometry,
                        geometry,
                    }
                };

                event
            })
            .into_iter()
    }

    pub fn detach_buffer(&mut self, id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| {
                surface.buffer = None;
                SurfaceEvent::BufferDetached { id }
            })
            .into_iter()
    }

    pub fn move_surface(
        &mut self,
        id: usize,
        position: pal::Position2D<i32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| {
                surface.position = position;
                SurfaceEvent::Moved {
                    id,
                    position: surface.position.clone(),
                    depth: surface.depth,
                }
            })
            .into_iter()
    }

    pub fn interactive_resize_start(&mut self,id: usize,serial: u32,edge: ews::ResizeEdge) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id).map(|surface|{
            surface.start_interactive_resize(serial,edge)
        }).into_iter().flatten()
    }

    pub fn interactive_resize_surface(
        &mut self,
        id: usize,
        serial: u32,
        inner_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_ref(id)
            .map(|surface| {
                if surface.kind.check_resize(serial){
                    Some(SurfaceEvent::Configuration { id, size: inner_size })
                }
                else{None}
            })
            .flatten().into_iter()
    }

    pub fn interactive_resize_end(&mut self,id: usize,serial: u32) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id).map(|surface|{
            surface.stop_interactive_resize(serial)
        }).into_iter().flatten()
    }

    pub fn resize_surface(
        &mut self,
        id: usize,
        size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_ref(id)
            .map(|_surface| SurfaceEvent::Configuration { id, size })
            .into_iter()
    }

    pub fn configure(
        &mut self,
        id: usize,
        geometry: Option<pal::Rectangle<i32, u32>>,
        min_size: pal::Size2D<u32>,
        max_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| {
                surface.configure(id,geometry,min_size,max_size)
            })
            .into_iter()
            .flatten()

    }

    pub fn commit_surface(&mut self, id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
        vec![SurfaceEvent::Committed { id }].into_iter()
    }

    pub fn surfaces_ref(&self) -> impl Iterator<Item = &Surface> {
        self.surfaces.iter()
    }
    pub fn surfaces_mut(&mut self) -> impl Iterator<Item = &mut Surface> {
        self.surfaces.iter_mut()
    }

    pub fn surface_ref(&self, id: usize) -> Option<&Surface> {
        self.surfaces.iter().find(|surface| surface.id == id)
    }
    pub fn surface_mut(&mut self, id: usize) -> Option<&mut Surface> {
        self.surfaces.iter_mut().find(|surface| surface.id == id)
    }

    fn update_depth(&mut self) -> impl Iterator<Item = SurfaceEvent> + Clone {
        std::iter::empty()
            .chain(self.update_cursor_surfaces_depth())
            .chain(self.update_surfaces_depth())
        //events.into_iter()
    }

    fn update_cursor_surfaces_depth(&mut self) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.cursor_surfaces
            .iter_mut()
            .enumerate()
            .map(|(index, surface)| {
                surface.depth = index as u32;

                let id = surface.id;
                let position = surface.position.clone();
                let depth = surface.depth;
                SurfaceEvent::Moved {
                    id,
                    position,
                    depth,
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
    fn update_surfaces_depth(&mut self) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let depth_offset = self.cursor_surfaces.len();
        self.surfaces
            .iter_mut()
            .enumerate()
            .filter_map(|(index, surface)| {
                let new_depth = (index + depth_offset) as u32;
                if new_depth != surface.depth {
                    surface.depth = new_depth;
                    let id = surface.id;
                    let position = surface.position.clone();
                    let depth = surface.depth;
                    Some(SurfaceEvent::Moved {
                        id,
                        position,
                        depth,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
