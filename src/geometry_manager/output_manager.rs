use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum OutputRequest {
    Added {
        id: usize,
        handle: Arc<pal::wgpu::Surface>,
        size: pal::Size2D<u32>,
    },
    Removed {
        id: usize,
    },
    Resized {
        id: usize,
        size: pal::Size2D<u32>,
    },
    Moved {
        old: Output,
        new_position: pal::Position2D<i32>,
    },
}

#[derive(Debug, Clone)]
pub enum OutputEvent {
    Added {
        id: usize,
        handle: Arc<pal::wgpu::Surface>,
        size: pal::Size2D<u32>,
    },
    Removed {
        id: usize,
    },
    Resized {
        id: usize,
        size: pal::Size2D<u32>,
    },
    Moved {
        id: usize,
        position: pal::Position2D<i32>,
    },
}

#[derive(Debug, Clone)]
pub struct Output {
    pub id: usize,
    pub geometry: pal::Rectangle<i32, u32>,
}
impl Output {
    pub fn new(id: usize, geometry: impl Into<pal::Rectangle<i32, u32>>) -> Self {
        let geometry = geometry.into();
        Self { id, geometry }
    }
}

#[derive(Debug)]
pub struct OutputManager {
    outputs: Vec<Output>,
}
impl OutputManager {
    pub fn new() -> Self {
        let outputs = Vec::new();
        Self { outputs }
    }

    pub fn add_output(
        &mut self,
        id: usize,
        handle: std::sync::Arc<pal::wgpu::Surface>,
        size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = OutputEvent> + Clone {
        let x_offset = *self
            .outputs
            .last()
            .map(|output| output.geometry.x_offset())
            .get_or_insert(0);
        let position = pal::Position2D::from((x_offset, 0));
        let output = Output::new(id.clone(), (position.clone(), size.clone()));
        self.outputs.push(output);
        vec![
            OutputEvent::Added { id, handle, size },
            OutputEvent::Moved { id, position },
        ]
        .into_iter()
    }
    pub fn del_output(&mut self, id: usize) -> impl Iterator<Item = OutputEvent> + Clone {
        let indexes_to_update = self
            .outputs
            .iter()
            .position(|output| output.id == id)
            .map(|position| {
                self.outputs.remove(position);
                position..self.outputs.len()
            })
            .get_or_insert(0..0)
            .clone();

        self.update_offset(indexes_to_update)
    }
    pub fn resize_output(
        &mut self,
        id: usize,
        size: pal::Size2D<u32>,
    ) -> impl Iterator<Item = OutputEvent> + Clone {
        match self.output_mut(id) {
            Some((index, output)) => {
                let event = OutputEvent::Resized {
                    id: output.id,
                    size: size.clone(),
                };
                output.geometry.size = size;

                let mut events = vec![event];
                events.append(&mut self.update_offset(index..self.outputs.len()).collect());
                events.into_iter()
            }
            None => Vec::new().into_iter(),
        }
    }

    pub fn relative_to_absolute(
        &self,
        id: usize,
        position: pal::Position2D<i32>,
    ) -> Option<pal::Position2D<i32>> {
        if let Some((_index, output)) = self.output_ref(id) {
            Some(output.geometry.position.clone() + position)
        } else {
            log::error!(target: "WComp","Cannot get the absolute position from output the relative one");
            None
        }
    }

    pub fn screen_size(&self) -> pal::Rectangle<i32, u32> {
        self.outputs_ref()
            .last()
            .map(|last_output| {
                let position = pal::Position2D::from((0i32, 0i32));
                let size = {
                    let mut size = last_output.geometry.size.clone();
                    size.width += last_output.geometry.position.x as u32;
                    size.width += last_output.geometry.position.x as u32;
                    size
                };
                pal::Rectangle { position, size }
            })
            .unwrap()
    }

    pub fn output_ref(&self, id: usize) -> Option<(usize, &Output)> {
        let output = self.outputs.iter().enumerate().find_map(|(index, output)| {
            if output.id == id {
                Some((index, output))
            } else {
                None
            }
        });
        if output.is_none() {
            log::error!(target: "Output Manager","Output {} not found",id);
        }
        output
    }
    pub fn outputs_ref(&self) -> impl Iterator<Item = &Output> {
        self.outputs.iter()
    }
    pub fn output_mut(&mut self, id: usize) -> Option<(usize, &mut Output)> {
        self.outputs
            .iter_mut()
            .enumerate()
            .find_map(|(index, output)| {
                if output.id == id {
                    Some((index, output))
                } else {
                    None
                }
            })
    }
    pub fn outputs_mut(&mut self) -> impl Iterator<Item = &mut Output> {
        self.outputs.iter_mut()
    }

    fn update_offset<'a>(
        &'a mut self,
        range: impl std::ops::RangeBounds<usize> + Iterator<Item = usize>,
    ) -> impl Iterator<Item = OutputEvent> + Clone {
        let mut events = Vec::new();
        for index in range {
            let x_offset = if index == 0 {
                0
            } else {
                self.outputs[index - 1].geometry.x_offset()
            };

            if self.outputs[index].geometry.position.x == x_offset {
                return events.into_iter();
            } else {
                let id = self.outputs[index].id;
                self.outputs[index].geometry.position.x = x_offset;
                let position = self.outputs[index].geometry.position.clone();
                let event = OutputEvent::Moved { id, position };
                events.push(event);
            }
        }
        events.into_iter()
    }

    pub fn get_surface_optimal_size(&self) -> pal::Size2D<u32> {
        self.outputs
            .iter()
            .next()
            .map(|output| {
                pal::Size2D::from([
                    output.geometry.size.width / 2,
                    output.geometry.size.height / 2,
                ])
            })
            .unwrap_or(pal::Size2D::from([200, 200]))
    }

    pub fn get_surface_optimal_position(&self, size: &pal::Size2D<u32>) -> pal::Position2D<i32> {
        self.outputs
            .iter()
            .next()
            .map(|output| {
                let mut x = (output.geometry.size.width / 2) as i32 - size.width as i32;
                if x < 0 {
                    x = 0;
                }
                let mut y = (output.geometry.size.height / 2) as i32 - size.height as i32;
                if y < 0 {
                    y = 0;
                }
                pal::Position2D::from([x, y])
            })
            .unwrap_or(pal::Position2D::from([0, 0]))
    }

    pub fn get_output_at(&self, position: &pal::Position2D<i32>) -> Option<&Output> {
        self.outputs
            .iter()
            .find(|output| output.geometry.contains(position))
    }
}
