//! [SurfaceManager][SurfaceManager] related structures and enumerations.

use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
/// Enumerator containing all the possible surface requests.
pub enum SurfaceRequest {
    Add {
        id: usize,
        kind: SurfaceKind,
        position: pal::Position2D<i32>,
    },
    Remove {
        id: usize,
    },
    Move {
        id: usize,
        position: pal::Position2D<i32>,
    },
    InteractiveResizeStart {
        id: usize,
        serial: u32,
        edge: ews::ResizeEdge,
    },
    InteractiveResize {
        id: usize,
        serial: u32,
        inner_size: pal::Size2D<u32>,
    },
    InteractiveResizeStop {
        id: usize,
        serial: u32,
    },
    Resize {
        id: usize,
        size: pal::Size2D<u32>,
    },
    Configuration {
        id: usize,
        geometry: Option<pal::Rectangle<i32, u32>>,
        min_size: pal::Size2D<u32>,
        max_size: pal::Size2D<u32>,
    },
    AttachBuffer {
        id: usize,
        handle: ews::WlBuffer,
        inner_geometry: pal::Rectangle<i32, u32>,
        size: pal::Size2D<u32>,
    },
    DetachBuffer {
        id: usize,
    },
    Maximize {
        id: usize,
    },
    Unmaximize {
        id: usize,
    },
    Commit {
        id: usize,
    },
}

#[derive(Debug, Clone)]
/// Enumerator containing all the possible surface events.
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
        edge: ews::ResizeEdge,
    },
    InteractiveResizeStopped {
        id: usize,
        serial: u32,
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
        geometry: pal::Rectangle<i32, u32>,
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
    Activated {
        id: usize,
    },
    Deactivated {
        id: usize,
    },
    Maximized {
        id: usize,
    },
    Unmaximized {
        id: usize,
    },
    Committed {
        id: usize,
    },
}

#[derive(Debug, Clone)]
/// Representation of the possible altered states.
pub struct AlteredState {
    original: pal::Rectangle<i32, u32>,
    minimized: bool,
    maximized: bool,
    fullscreen: bool,
    resizing: Option<(u32, ews::ResizeEdge)>, //Serial
    moving: Option<u32>,                      //Serial
}
impl AlteredState {
    pub fn new(original: pal::Rectangle<i32, u32>) -> Self {
        let minimized = false;
        let maximized = false;
        let fullscreen = false;
        let resizing = None;
        let moving = None;
        Self {
            original,
            minimized,
            maximized,
            fullscreen,
            resizing,
            moving,
        }
    }

    pub fn is_minimized(&self) -> bool {
        self.minimized
    }
    pub fn is_maximized(&self) -> bool {
        self.maximized
    }
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }
    pub fn is_resizing(&self) -> bool {
        self.resizing.is_some()
    }
    pub fn is_resizing_width(&self, serial: u32) -> bool {
        self.resizing.map(|(serial, _)| serial) == Some(serial)
    }
    pub fn is_moving(&self) -> bool {
        self.moving.is_some()
    }
    pub fn is_moving_width(&self, serial: u32) -> bool {
        self.moving == Some(serial)
    }
    pub fn is_empty(&self) -> bool {
        !self.is_minimized()
            && !self.is_maximized()
            && !self.is_fullscreen()
            && !self.is_resizing()
            && !self.is_moving()
    }
    pub fn start_interactive_resize(&mut self, serial: u32, edge: ews::ResizeEdge) -> bool {
        if self.resizing.is_some() | self.moving.is_some() | self.maximized | self.minimized {
            false
        } else {
            self.resizing = Some((serial, edge));
            true
        }
    }
    pub fn stop_interactive_resize(&mut self, serial: u32) -> bool {
        if Some(serial) == self.resizing.map(|(serial, _)| serial) {
            self.resizing = None;
            true
        } else {
            false
        }
    }
    pub fn check_resize(&self, serial: u32) -> bool {
        match &self.resizing {
            Some((current_serial, _)) => current_serial == &serial,
            None => true,
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Surface state related data.
pub struct SurfaceState {
    altered_state: Option<AlteredState>,
}
impl SurfaceState {
    pub fn check_resize(&self, serial: u32) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.check_resize(serial))
            != Some(false)
    }

    pub fn start_interactive_resize(
        &mut self,
        serial: u32,
        edge: ews::ResizeEdge,
        inner_geometry: pal::Rectangle<i32, u32>,
    ) -> bool {
        if self.altered_state.is_none() {
            self.altered_state = Some(AlteredState::new(inner_geometry));
        }
        let result = self
            .altered_state
            .as_mut()
            .map(|altered_state| altered_state.start_interactive_resize(serial, edge));
        result == Some(true)
    }
    pub fn stop_interactive_resize(&mut self, serial: u32) -> bool {
        let result = self
            .altered_state
            .as_mut()
            .map(|altered_state| altered_state.stop_interactive_resize(serial));
        if self.is_empty() {
            self.altered_state = None;
        }
        result == Some(true)
    }

    pub fn is_minimized(&self) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_minimized())
            == Some(true)
    }
    pub fn is_maximized(&self) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_maximized())
            == Some(true)
    }
    pub fn is_fullscreen(&self) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_fullscreen())
            == Some(true)
    }
    pub fn is_resizing(&self) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_resizing())
            == Some(true)
    }
    pub fn is_resizing_width(&self, serial: u32) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_resizing_width(serial))
            == Some(true)
    }
    pub fn is_moving(&self) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_moving())
            == Some(true)
    }
    pub fn is_moving_width(&self, serial: u32) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_moving_width(serial))
            == Some(true)
    }
    pub fn is_empty(&self) -> bool {
        self.altered_state
            .as_ref()
            .map(|altered_state| altered_state.is_empty())
            == Some(true)
    }
}

#[derive(Debug, Clone)]
/// Popup state related data.
pub struct PopupState {
    pub anchor: pal::Rectangle<i32, u32>,
    pub anchor_edges: ews::Anchor,
    pub gravity: ews::Gravity,
    pub constraint_adjustment: ews::ConstraintAdjustment,
    pub offset: pal::Offset2D<i32>,
    pub reactive: bool,
}

#[derive(Debug, Clone)]
/// All the possible surface kinds.
pub enum SurfaceKind {
    Toplevel {
        handle: ews::ToplevelSurface,
        state: SurfaceState,
    },
    Popup {
        handle: ews::PopupSurface,
        state: PopupState,
    },
}
impl SurfaceKind {
    pub fn handle(&self) -> Option<&ews::WlSurface> {
        match self {
            Self::Toplevel { handle, .. } => handle.get_surface(),
            Self::Popup { handle, .. } => handle.get_surface(),
        }
    }
    pub fn check_resize(&self, serial: u32) -> bool {
        match self {
            Self::Toplevel { state, .. } => state.check_resize(serial),
            Self::Popup { .. } => true,
        }
    }
}
impl From<ews::ToplevelSurface> for SurfaceKind {
    fn from(handle: ews::ToplevelSurface) -> Self {
        let state = SurfaceState::default();
        Self::Toplevel { handle, state }
    }
}

#[derive(Debug, Clone)]
/// Representation of a surface buffer.
pub struct Buffer {
    handle: ews::WlBuffer,
    geometry: pal::Rectangle<i32, u32>,
    //pub size: pal::Size2D<u32>
}
impl Buffer {
    pub fn handle(&self) -> &ews::WlBuffer {
        &self.handle
    }
    pub fn geometry(&self) -> &pal::Rectangle<i32, u32> {
        &self.geometry
    }
    pub fn size(&self) -> pal::Size2D<u32> {
        use ews::Buffer;
        match ews::buffer_type(&self.handle) {
            Some(ews::BufferType::Shm) => ews::with_buffer_contents(&self.handle, |_data, info| {
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
/// Representation of a surface.
pub struct Surface {
    id: usize,
    kind: SurfaceKind,
    buffer: Option<Buffer>,
    min_size: pal::Size2D<u32>,
    max_size: pal::Size2D<u32>,
    position: pal::Position2D<i32>,
    depth: u32,
    children: VecDeque<Box<Surface>>,
}
impl Surface {
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn kind(&self) -> &SurfaceKind {
        &self.kind
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
        self.size()
            .map(|size| pal::Rectangle::from((self.position().clone(), size)))
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

    pub fn add_child(&mut self, surface: Surface) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let event = SurfaceEvent::Added {
            id: surface.id,
            kind: surface.kind.clone(),
        };
        self.children.push_front(Box::new(surface));
        std::iter::once(event) //.chain(self.update_children())
    }
    pub fn del_child(&mut self, id: usize) -> Option<impl Iterator<Item = SurfaceEvent> + Clone> {
        self.children
            .iter()
            .position(|surface| surface.id == id)
            .map(|position| {
                self.children
                    .remove(position)
                    .map(|removed_surface| {
                        removed_surface
                            .buffer
                            .map(|_| SurfaceEvent::BufferDetached { id })
                            .into_iter()
                            .chain(std::iter::once(SurfaceEvent::Removed { id }))
                    })
                    .into_iter()
                    .flatten()
            })
    }

    pub fn children_ref(&self) -> impl Iterator<Item = &Surface> + Clone {
        self.children.iter().map(|child| child.as_ref())
    }

    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Surface> {
        self.children.iter_mut().map(|child| child.as_mut())
    }

    pub fn update_depth(&mut self, depth: &mut u32) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.children_mut()
            .map(|child| child.update_depth(depth))
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()
            .chain({
                let new_depth = *depth;
                *depth += 1;
                if self.depth != new_depth {
                    self.depth = new_depth;

                    Some(SurfaceEvent::Moved {
                        id: self.id,
                        position: self.position.clone(),
                        depth: self.depth,
                    })
                } else {
                    None
                }
                .into_iter()
            })

        /*
        let mut events = Vec::new();
        for child in self.children_mut() {
            events.append(&mut child.update_depth(depth).collect());
        }

        let new_depth = *depth;
        if self.depth != new_depth {
            self.depth = new_depth;
            *depth = new_depth;

            events.push(SurfaceEvent::Moved {
                id: self.id,
                position: self.position.clone(),
                depth: self.depth,
            });
        }

        *depth += 1;

        events.into_iter()
        */
    }

    pub fn r#move(
        &mut self,
        position: pal::Position2D<i32>,
        depth: u32,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.position = position.clone();
        self.depth = depth;
        std::iter::once(SurfaceEvent::Moved {
            id: self.id,
            position,
            depth,
        })
        .chain(self.update_children())
    }

    pub fn surfaces_ref(&self) -> impl Iterator<Item = &Surface> {
        self.children
            .iter()
            .map(|surface| surface.as_ref())
            .chain(std::iter::once(self))
    }

    /*
        pub fn surfaces_mut(&mut self) -> impl Iterator<Item = &mut Surface> {
            self.children.iter_mut().map(|child|child.as_mut()).chain(std::iter::once(self))
            /*
            let mut surfaces = Vec::new();
            surfaces.push(self);
            for surface in &self.childs {
                surfaces.push(surface.as_mut());
            }

            surfaces.into_iter()
            */
        }
    */
    pub fn surface_parent_mut(&mut self, id: usize) -> Option<&mut Surface> {
        if self.surface_mut(id).is_some() {
            Some(self)
        } else {
            None
        }
    }
    pub fn surface_parent_ref(&self, id: usize) -> Option<&Surface> {
        if self.surface_ref(id).is_some() {
            Some(self)
        } else {
            None
        }
    }

    pub fn surface_ref(&self, id: usize) -> Option<&Surface> {
        if self.id == id {
            Some(self)
        } else {
            self.children
                .iter()
                .find(|child| child.id == id)
                .map(|surface| surface.as_ref())
        }
    }
    pub fn surface_mut(&mut self, id: usize) -> Option<&mut Surface> {
        if self.id == id {
            Some(self)
        } else {
            self.children
                .iter_mut()
                .find(|child| child.id == id)
                .map(|surface| surface.as_mut())
        }
    }

    /*
        pub fn surface_ref(&self, id: usize) -> Option<&Surface> {
            if self.id == id {
                Some(self)
            } else {
                for child in &self.children {
                    if child.id == id {
                        return Some(child);
                    }
                }
                None
            }
        }
        pub fn surface_mut(&mut self, id: usize) -> Option<&mut Surface> {
            if self.id == id {
                Some(self)
            } else {
                for child in &mut self.children {
                    if child.id == id {
                        return Some(child);
                    }
                }
                None
            }
        }
    */
    pub fn update_children(&mut self) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let mut events = Vec::new();
        let inner_geometry = self.inner_geometry().cloned();
        for child in &mut self.children {
            if let Some((parent_geometry, child_size)) = inner_geometry.clone().zip(child.size()) {
                match &mut child.kind {
                    SurfaceKind::Popup { state, .. } => {
                        let mut absolute_anchor = state.anchor.clone();
                        //absolute_anchor.position = absolute_anchor.position + self.position.clone();
                        absolute_anchor.position = absolute_anchor.position
                            + self.position.clone()
                            + parent_geometry.position.clone();

                        let position = match state.anchor_edges {
                            ews::Anchor::TopLeft => absolute_anchor.position,
                            ews::Anchor::BottomLeft => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((0, absolute_anchor.size.height as i32))
                            }
                            ews::Anchor::TopRight => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((absolute_anchor.size.width as i32, 0))
                            }
                            ews::Anchor::BottomRight => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((
                                        absolute_anchor.size.width as i32,
                                        absolute_anchor.size.height as i32,
                                    ))
                            }
                            ews::Anchor::Top => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((
                                        absolute_anchor.size.width as i32 / 2,
                                        0,
                                    ))
                            }
                            ews::Anchor::Bottom => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((
                                        absolute_anchor.size.width as i32 / 2,
                                        absolute_anchor.size.height as i32,
                                    ))
                            }
                            ews::Anchor::Left => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((
                                        0,
                                        absolute_anchor.size.height as i32 / 2,
                                    ))
                            }
                            ews::Anchor::Right => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((
                                        absolute_anchor.size.width as i32,
                                        absolute_anchor.size.height as i32 / 2,
                                    ))
                            }
                            ews::Anchor::None => {
                                absolute_anchor.position
                                    + pal::Offset2D::from((
                                        absolute_anchor.size.width as i32 / 2,
                                        absolute_anchor.size.height as i32 / 2,
                                    ))
                            }
                            _ => absolute_anchor.position.clone(),
                        };

                        let aligned_position = match state.gravity {
                            ews::Gravity::TopRight => position,
                            ews::Gravity::BottomRight => {
                                position - pal::Offset2D::from((0i32, child_size.height as i32))
                            }
                            ews::Gravity::TopLeft => {
                                position - pal::Offset2D::from((child_size.width as i32, 0))
                            }
                            ews::Gravity::BottomLeft => {
                                position
                                    - pal::Offset2D::from((
                                        child_size.width as i32,
                                        child_size.height as i32,
                                    ))
                            }

                            ews::Gravity::Bottom => {
                                position - pal::Offset2D::from((child_size.width as i32 / 2, 0))
                            }
                            ews::Gravity::Top => {
                                position
                                    - pal::Offset2D::from((child_size.width as i32 / 2, 0))
                                    - pal::Offset2D::from((0, child_size.height as i32))
                            }
                            ews::Gravity::Right => {
                                position - pal::Offset2D::from((0, child_size.height as i32 / 2))
                            }
                            ews::Gravity::Left => {
                                position
                                    - pal::Offset2D::from((0, child_size.height as i32 / 2))
                                    - pal::Offset2D::from((child_size.width as i32, 0))
                            }
                            ews::Gravity::None => {
                                position
                                    - pal::Offset2D::from((
                                        child_size.width as i32 / 2,
                                        child_size.height as i32 / 2,
                                    ))
                            }
                            _ => position,
                        };
                        //println!("New position: {}", aligned_position);
                        events.append(&mut child.r#move(aligned_position, child.depth).collect());
                    }
                    _ => (),
                }
            } else {
                println!("No buffer found during update_children function");
            }
        }
        events.into_iter()
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
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let mut events = Vec::new();
        if self.min_size != min_size {
            self.min_size = min_size.clone();
            events.push(SurfaceEvent::MinSize { id, size: min_size });
        }
        if self.max_size != max_size {
            self.max_size = max_size.clone();
            events.push(SurfaceEvent::MaxSize { id, size: max_size });
        }

        let kind = self.kind.clone();
        let depth = self.depth;
        let current_position = self.position().clone();
        let current_inner_geometry = self.inner_geometry();

        if let (Some(inner_geometry), Some(current_inner_geometry)) =
            (inner_geometry.clone(), current_inner_geometry)
        {
            match kind {
                SurfaceKind::Toplevel { state, .. } => {
                    if let Some(altered_state) = state.altered_state.as_ref() {
                        match altered_state.resizing {
                            Some((_, edge)) => match edge {
                                ews::ResizeEdge::Left => {
                                    let offset = current_inner_geometry.size.width as i32
                                        - inner_geometry.size.width as i32;
                                    let new_position =
                                        current_position + pal::Offset2D::from((offset, 0));
                                    events.append(&mut self.r#move(new_position, depth).collect());
                                }
                                ews::ResizeEdge::Top => {
                                    let offset = current_inner_geometry.size.height as i32
                                        - inner_geometry.size.height as i32;
                                    let new_position =
                                        current_position + pal::Offset2D::from((0, offset));
                                    events.append(&mut self.r#move(new_position, depth).collect());
                                }
                                ews::ResizeEdge::TopLeft => {
                                    let offset_x = current_inner_geometry.size.width as i32
                                        - inner_geometry.size.width as i32;
                                    let offset_y = current_inner_geometry.size.height as i32
                                        - inner_geometry.size.height as i32;
                                    let new_position = current_position
                                        + pal::Offset2D::from((offset_x, offset_y));
                                    events.append(&mut self.r#move(new_position, depth).collect());
                                }
                                ews::ResizeEdge::TopRight => {
                                    let offset = current_inner_geometry.size.height as i32
                                        - inner_geometry.size.height as i32;
                                    let new_position =
                                        current_position + pal::Offset2D::from((0, offset));
                                    events.append(&mut self.r#move(new_position, depth).collect());
                                }
                                ews::ResizeEdge::BottomLeft => {
                                    let offset = current_inner_geometry.size.width as i32
                                        - inner_geometry.size.width as i32;
                                    let new_position =
                                        current_position + pal::Offset2D::from((offset, 0));
                                    events.append(&mut self.r#move(new_position, depth).collect());
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                }
                SurfaceKind::Popup { .. } => (),
            }
        }
        if let (Some(buffer), Some(inner_geometry)) = (self.buffer.as_mut(), inner_geometry) {
            if buffer.geometry != inner_geometry {
                buffer.geometry = inner_geometry.clone();
                events.push(SurfaceEvent::Geometry {
                    id,
                    geometry: inner_geometry.clone(),
                });
                events.push(SurfaceEvent::Resized {
                    id,
                    size: buffer.size(),
                });
            }
        }
        events.into_iter()
    }

    pub fn start_interactive_resize(
        &mut self,
        serial: u32,
        edge: ews::ResizeEdge,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let geometry = self.geometry();
        match &mut self.kind {
            SurfaceKind::Toplevel { state, .. } => {
                if let Some(geometry) = geometry {
                    if state.start_interactive_resize(serial, edge, geometry) {
                        vec![SurfaceEvent::InteractiveResizeStarted {
                            id: self.id,
                            serial,
                            edge,
                        }]
                        .into_iter()
                    } else {
                        Vec::new().into_iter()
                    }
                } else {
                    Vec::new().into_iter()
                }
            }
            SurfaceKind::Popup { .. } => Vec::new().into_iter(),
        }
    }

    pub fn stop_interactive_resize(
        &mut self,
        serial: u32,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        match &mut self.kind {
            SurfaceKind::Toplevel { state, .. } => {
                if state.stop_interactive_resize(serial) {
                    vec![SurfaceEvent::InteractiveResizeStopped {
                        id: self.id,
                        serial,
                    }]
                    .into_iter()
                } else {
                    Vec::new().into_iter()
                }
            }
            SurfaceKind::Popup { .. } => Vec::new().into_iter(),
        }
    }

    pub fn maximize(&mut self) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.geometry()
            .map(|geometry| match &mut self.kind {
                SurfaceKind::Toplevel { state, .. } => {
                    state
                        .altered_state
                        .get_or_insert(AlteredState::new(geometry))
                        .maximized = true;
                    Some(SurfaceEvent::Maximized { id: self.id })
                }
                SurfaceKind::Popup { .. } => None,
            })
            .flatten()
            .into_iter()
    }

    pub fn unmaximize(&mut self) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.geometry().map(|geometry|{
            let original = match &mut self.kind {
                SurfaceKind::Toplevel{handle,state}=>{
                    let original = state.altered_state.as_mut().map(|altered_state|{
                        altered_state.maximized = false;
                        if altered_state.is_empty() {Some(altered_state.original.clone())}
                        else{None}
                    }).flatten();

                    if original.is_some() {state.altered_state = None;}
                    original
                },
                SurfaceKind::Popup {..}=>None
            };
            original.map(|original|{
                std::iter::once(SurfaceEvent::Unmaximized { id: self.id })
                .chain(self.r#move(original.position,self.depth))
                //.chain(self.res)

            }).into_iter().flatten().collect::<Vec<_>>()
        }).into_iter().flatten()
    }
/*
    pub fn unmaximize(&mut self) -> impl Iterator<Item = SurfaceEvent> + Clone {
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
    pub fn minimize(&mut self) {
        self.geometry().map(|geometry| match &mut self.kind {
            SurfaceKind::Toplevel { state, .. } => {
                state
                    .altered_state
                    .get_or_insert(AlteredState::new(geometry))
                    .minimized = true;
            }
            SurfaceKind::Popup { .. } => (),
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
/// Component responsible to handle the surfaces.
pub struct SurfaceManager {
    cursor_surfaces: VecDeque<Surface>,
    surfaces: VecDeque<Surface>,
    border_grace: u32,
    active_surface: Option<usize>,
}
impl SurfaceManager {
    const CURSOR_MIN_DEPTH: u32 = 0;
    const SURFACE_MIN_DEPTH: u32 = 16;
    pub fn new() -> Self {
        let cursor_surfaces = VecDeque::new();
        let surfaces = VecDeque::new();
        let border_grace = 10;
        let active_surface = None;
        Self {
            cursor_surfaces,
            surfaces,
            border_grace,
            active_surface,
        }
    }

    pub fn get_surface_at(&mut self, position: &pal::Position2D<i32>) -> Option<&Surface> {
        self.surfaces_ref().find(|surface| {
            if let Some(buffer) = &surface.buffer {
                pal::Rectangle::from((
                    surface.position.clone() + buffer.geometry.position.clone()
                        - pal::Offset2D::from((self.border_grace as i32, self.border_grace as i32)),
                    buffer.geometry.size.clone()
                        + pal::Offset2D::from((self.border_grace * 2, self.border_grace * 2)),
                ))
                .contains(&position)
            } else {
                false
            }
        })
    }

    pub fn add_surface(
        &mut self,
        id: usize,
        kind: SurfaceKind,
        position: pal::Position2D<i32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let min_size = pal::Size2D::from((0, 0));
        let max_size = pal::Size2D::from((0, 0));
        let depth = 0;
        let buffer = None;
        let children = VecDeque::new();

        let surface = Surface {
            id,
            kind: kind.clone(),
            buffer,
            min_size,
            max_size,
            position,
            depth,
            children,
        };

        match kind {
            SurfaceKind::Toplevel { .. } => {
                self.surfaces.push_front(surface);
                vec![SurfaceEvent::Added { id, kind }]
            }
            SurfaceKind::Popup { ref handle, .. } => handle
                .get_parent_surface()
                .map(|parent_surface| {
                    ews::with_states(&parent_surface, |surface_data| {
                        let id = ews::surface_id(&surface_data).unwrap();
                        self.surface_mut(id)
                            .map(|parent_surface| parent_surface.add_child(surface))
                    })
                    .unwrap()
                })
                .flatten()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        }
        .into_iter()
        .chain(self.update_surfaces_depth())
    }

    pub fn del_surface(&mut self, id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let mut count = 0;
        let mut top_level_surface = None;
        for surface in &mut self.surfaces {
            if let Some(events) = surface.del_child(id) {
                return events
                    .chain({
                        if self.active_surface == Some(id) {
                            self.active_surface = None;
                            Some(SurfaceEvent::Deactivated { id })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
            }
            if surface.id == id {
                top_level_surface = Some(count);
                break;
            }
            count += 1;
        }
        if let Some(top_level_surface_to_be_removed) = top_level_surface {
            self.surfaces
                .remove(top_level_surface_to_be_removed)
                .map(|removed_surface| {
                    removed_surface
                        .buffer
                        .map(|_| SurfaceEvent::BufferDetached { id })
                        .into_iter()
                        .chain(vec![SurfaceEvent::Removed { id }])
                        .chain({
                            if self.active_surface == Some(id) {
                                self.active_surface = None;
                                Some(SurfaceEvent::Deactivated { id })
                            } else {
                                None
                            }
                        })
                })
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            log::error!(target: "SurfaceManager","Cannot delete surface: surface {} not found",id);
            Vec::new().into_iter()
        }
    }

    pub fn del_surface_old(&mut self, id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
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
                        .chain({
                            if self.active_surface == Some(id) {
                                self.active_surface = None;
                                Some(SurfaceEvent::Deactivated { id })
                            } else {
                                None
                            }
                        })
                        /*
                        self.active_surface.as_mut().map(|current_active_surface|{
                            if current_active_surface == &id {}
                            else{None}
                        }
                        */
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
                    });
                    let geometry = pal::Rectangle::from((surface.position.clone(), suggested_size));
                    SurfaceEvent::BufferAttached {
                        id,
                        handle,
                        inner_geometry,
                        geometry,
                    }
                };
                std::iter::once(event).chain(surface.update_children())
            })
            .map(|events| {
                events.chain(
                    self.surface_parent_mut(id)
                        .map(|surface| surface.update_children())
                        .into_iter()
                        .flatten(),
                )
            })
            .into_iter()
            .flatten()
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
            .map(|surface| surface.r#move(position, surface.depth))
            .into_iter()
            .flatten()
    }

    pub fn focus_surface(
        &mut self,
        id: Option<usize>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let events = id
            .map(|id| {
                self.surfaces
                    .iter()
                    .position(|surface| surface.id == id)
                    .map(|position| {
                        self.surfaces.remove(position).map(|surface| {
                            self.surfaces.push_front(surface);
                            self.update_surfaces_depth()
                        })
                    })
                    .flatten()
            })
            .flatten()
            .into_iter()
            .flatten();

        let current_active_surface = self.active_surface.take();
        self.active_surface = id;

        events
            .chain(
                current_active_surface.map(|current_active_surface| SurfaceEvent::Deactivated {
                    id: current_active_surface,
                }),
            )
            .chain(
                self.active_surface
                    .map(|new_active_surface| SurfaceEvent::Activated {
                        id: new_active_surface,
                    }),
            )
    }

    pub fn maximize_surface(&mut self, id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| surface.maximize())
            .into_iter()
            .flatten()
    }

    pub fn unmaximize_surface(&mut self,id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id).map(|surface|surface.unmaximize()).into_iter().flatten()
    }

    pub fn interactive_resize_start(
        &mut self,
        id: usize,
        serial: u32,
        edge: ews::ResizeEdge,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| surface.start_interactive_resize(serial, edge))
            .into_iter()
            .flatten()
    }

    pub fn interactive_resize_surface(
        &mut self,
        id: usize,
        serial: u32,
        inner_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_ref(id)
            .map(|surface| {
                if surface.kind.check_resize(serial) {
                    Some(SurfaceEvent::Configuration {
                        id,
                        size: inner_size,
                    })
                } else {
                    None
                }
            })
            .flatten()
            .into_iter()
    }

    pub fn interactive_resize_end(
        &mut self,
        id: usize,
        serial: u32,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| surface.stop_interactive_resize(serial))
            .into_iter()
            .flatten()
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

    pub fn configure_surface(
        &mut self,
        id: usize,
        geometry: Option<pal::Rectangle<i32, u32>>,
        min_size: pal::Size2D<u32>,
        max_size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        self.surface_mut(id)
            .map(|surface| surface.configure(id, geometry, min_size, max_size))
            .into_iter()
            .flatten()
    }

    pub fn commit_surface(&mut self, id: usize) -> impl Iterator<Item = SurfaceEvent> + Clone {
        vec![SurfaceEvent::Committed { id }].into_iter()
    }

    pub fn surfaces_ref(&self) -> impl Iterator<Item = &Surface> {
        self.surfaces
            .iter()
            .map(|surface| surface.surfaces_ref())
            .flatten()
    }
    pub fn surfaces_mut(&mut self) -> impl Iterator<Item = &mut Surface> {
        self.surfaces.iter_mut()
    }

    pub fn surface_ref(&self, id: usize) -> Option<&Surface> {
        for surface in &self.surfaces {
            let result = surface.surface_ref(id);
            if result.is_some() {
                return result;
            }
        }
        return None;
    }
    pub fn surface_mut(&mut self, id: usize) -> Option<&mut Surface> {
        for surface in &mut self.surfaces {
            let result = surface.surface_mut(id);
            if result.is_some() {
                return result;
            }
        }
        return None;
    }

    pub fn surface_parent_mut(&mut self, id: usize) -> Option<&mut Surface> {
        self.surfaces
            .iter_mut()
            .find_map(|surface| surface.surface_parent_mut(id))
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
        let depth_offset = self.cursor_surfaces.len() as u32;

        let mut depth = depth_offset;
        self.surfaces
            .iter_mut()
            .map(|surface| surface.update_depth(&mut depth))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()
        /*
        let mut events = Vec::new();
        for surface in &mut self.surfaces {
            events.append(&mut surface.update_depth(&mut depth).collect());
        }
        events.into_iter()
        */

        /*
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
        */
    }

    fn postprocess_events(
        &mut self,
        events: impl Iterator<Item = SurfaceEvent> + Clone,
    ) -> impl Iterator<Item = SurfaceEvent> + Clone {
        let mut additional_events = Vec::new();
        let mut events = events.collect::<Vec<_>>();
        events
            .iter()
            .enumerate()
            .for_each(|(index, event)| match event {
                SurfaceEvent::Removed { id } => {
                    let id = *id;
                    if self.active_surface == Some(id) {
                        self.active_surface = None;
                        additional_events.push((index, SurfaceEvent::Deactivated { id }));
                    }
                }
                _ => (),
            });

        additional_events
            .into_iter()
            .rev()
            .for_each(|(index, event)| {
                events.insert(index, event);
            });

        events.into_iter()
    }
}
