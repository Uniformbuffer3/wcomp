#[derive(Debug, Clone)]
pub enum CursorRequest {
    Added {
        id: usize,
        position: pal::Position2D<i32>,
        image: Option<usize>,
    },
    Removed {
        id: usize,
    },
    Moved {
        id: usize,
        position: pal::Position2D<i32>,
    },
    Button {
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Button>,
        state: pal::State,
    },
    Axis {
        id: usize,
        time: u32,
        source: pal::AxisSource,
        direction: pal::AxisDirection,
        value: pal::AxisValue,
    },
    Focus {
        id: usize,
        surface: usize,
    },
    Entered {
        id: usize,
        output_id: usize,
    },
    Left {
        id: usize,
        output_id: usize,
    },
}

#[derive(Debug, Clone)]
pub enum KeyboardRequest {
    Added {
        id: usize,
        rate: i32,
        delay: i32,
    },
    Removed {
        id: usize,
    },
    Key {
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Key>,
        state: pal::State,
    },
    Focus {
        id: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub enum SeatRequest {
    Added { id: usize, name: String },
    Removed { id: usize },
    Cursor(CursorRequest),
    Keyboard(KeyboardRequest),
}
impl From<CursorRequest> for SeatRequest {
    fn from(request: CursorRequest) -> Self {
        Self::Cursor(request)
    }
}

#[derive(Debug, Clone)]
pub enum CursorEvent {
    Added {
        id: usize,
        position: pal::Position2D<i32>,
        image: Option<usize>,
    },
    Removed {
        id: usize,
    },
    Moved {
        id: usize,
        position: pal::Position2D<i32>,
    },
    Button {
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Button>,
        state: pal::State,
    },
    Axis {
        id: usize,
        time: u32,
        source: pal::AxisSource,
        direction: pal::AxisDirection,
        value: pal::AxisValue,
    },
    Focus {
        id: usize,
        surface: Option<usize>,
    },
    Entered {
        id: usize,
        output_id: usize,
    },
    Left {
        id: usize,
        output_id: usize,
    },
}

#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    Added {
        id: usize,
        rate: i32,
        delay: i32,
    },
    Removed {
        id: usize,
    },
    Key {
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Key>,
        state: pal::State,
    },
    Focus {
        id: usize,
        surface: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub enum SeatEvent {
    Added { id: usize, name: String },
    Removed { id: usize },
    Cursor(CursorEvent),
    Keyboard(KeyboardEvent),
}
impl From<CursorEvent> for SeatEvent {
    fn from(event: CursorEvent) -> Self {
        Self::Cursor(event)
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    position: pal::Position2D<i32>,
    focus: Option<usize>,
    image: Option<usize>,
    output: Option<usize>,
}
impl Cursor {
    pub fn position(&self) -> &pal::Position2D<i32> {
        &self.position
    }
    pub fn focus(&self) -> &Option<usize> {
        &self.focus
    }
    pub fn output(&self) -> &Option<usize> {
        &self.output
    }
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    focus: Option<usize>,
    rate: i32,
    delay: i32,
}

#[derive(Debug, Clone)]
pub struct Seat {
    id: usize,
    name: String,
    cursor: Option<Cursor>,
    keyboard: Option<Keyboard>,
}

#[derive(Debug)]
pub struct SeatManager {
    seats: Vec<Seat>,
}

impl SeatManager {
    pub fn new() -> Self {
        let seats = Vec::new();
        Self { seats }
    }
    pub fn seat_ref(&self, id: usize) -> Option<&Seat> {
        let seat = self
            .seats
            .iter()
            .find_map(|seat| if seat.id == id { Some(seat) } else { None });

        if seat.is_none() {
            log::error!(target:"WComp","Seat manager | Seat {} not found",id);
        }
        seat
    }
    fn seat_mut(&mut self, id: usize) -> Option<&mut Seat> {
        let seat = self
            .seats
            .iter_mut()
            .find_map(|seat| if seat.id == id { Some(seat) } else { None });
        if seat.is_none() {
            log::error!(target:"WComp","Seat manager | Seat {} not found",id);
        }
        seat
    }
    pub fn add_seat(&mut self, id: usize, name: String) -> impl Iterator<Item = SeatEvent> + Clone {
        let cursor = None;
        let keyboard = None;
        let seat = Seat {
            id,
            name: name.clone(),
            cursor,
            keyboard,
        };
        self.seats.push(seat);
        log::info!(target:"WComp","Seat manager | Seat added");
        vec![SeatEvent::Added { id, name }].into_iter()
    }
    pub fn del_seat(&mut self, id: usize) -> impl Iterator<Item = SeatEvent> + Clone {
        if let Some(position) = self.seats.iter().position(|seat| seat.id == id) {
            self.seats.remove(position);
            vec![SeatEvent::Removed { id }].into_iter()
        } else {
            log::error!(target:"WComp","Seat manager | Cannot delete Seat: id {} not found",id);
            Vec::new().into_iter()
        }
    }

    pub fn keyboard_ref(&self, id: usize) -> Option<&Keyboard> {
        self.seat_ref(id)
            .map(|seat| seat.keyboard.as_ref())
            .flatten()
    }
    fn keyboard_mut(&mut self, id: usize) -> Option<&mut Keyboard> {
        self.seat_mut(id)
            .map(|seat| seat.keyboard.as_mut())
            .flatten()
    }

    pub fn add_keyboard(
        &mut self,
        id: usize,
        rate: i32,
        delay: i32,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_mut(id)
            .map(|seat| {
                let focus = None;
                seat.keyboard = Some(Keyboard { focus, rate, delay });
                vec![SeatEvent::Keyboard(KeyboardEvent::Added {
                    id,
                    rate,
                    delay,
                })]
                .into_iter()
            })
            .into_iter()
            .flatten()
    }

    pub fn del_keyboard(&mut self, id: usize) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_mut(id)
            .map(|seat| {
                seat.keyboard = None;
                vec![SeatEvent::Keyboard(KeyboardEvent::Removed { id })].into_iter()
            })
            .into_iter()
            .flatten()
    }

    pub fn keyboard_focus(
        &mut self,
        id: usize,
        surface: Option<usize>,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.keyboard_mut(id)
            .map(|keyboard| {
                keyboard.focus = surface;
                vec![SeatEvent::Keyboard(KeyboardEvent::Focus { id, surface })].into_iter()
            })
            .into_iter()
            .flatten()
    }

    pub fn keyboard_key(
        &mut self,
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Key>,
        state: pal::State,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.keyboard_mut(id)
            .map(|_keyboard| {
                vec![SeatEvent::Keyboard(KeyboardEvent::Key {
                    id,
                    time,
                    code,
                    key,
                    state,
                })]
                .into_iter()
            })
            .into_iter()
            .flatten()
    }

    pub fn cursor_ref(&self, id: usize) -> Option<&Cursor> {
        self.seat_ref(id).map(|seat| seat.cursor.as_ref()).flatten()
    }
    fn cursor_mut(&mut self, id: usize) -> Option<&mut Cursor> {
        self.seat_mut(id).map(|seat| seat.cursor.as_mut()).flatten()
    }

    pub fn add_cursor(
        &mut self,
        id: usize,
        position: pal::Position2D<i32>,
        image: Option<usize>,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_mut(id)
            .map(|seat| {
                let output = None;
                let focus = None;
                seat.cursor = Some(Cursor {
                    focus,
                    position: position.clone(),
                    image,
                    output,
                });
                log::info!(target:"WComp","Seat manager | Cursor added");
                SeatEvent::from(CursorEvent::Added {
                    id,
                    position,
                    image,
                })
            })
            .into_iter()
    }

    pub fn del_cursor(&mut self, id: usize) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_mut(id)
            .map(|seat| {
                seat.cursor = None;
                log::info!(target:"WComp","Seat manager | Cursor removed");
                SeatEvent::from(CursorEvent::Removed { id })
            })
            .into_iter()
    }

    pub fn enter_cursor(
        &mut self,
        id: usize,
        output_id: usize,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_mut(id)
            .map(|seat| {
                seat.cursor.as_mut().map(|cursor| {
                    cursor.output = Some(output_id);
                    SeatEvent::from(CursorEvent::Entered { id, output_id })
                })
            })
            .flatten()
            .into_iter()
    }

    pub fn left_cursor(
        &mut self,
        id: usize,
        output_id: usize,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        /*
        self.seat_mut(id).map(|seat|{
            if let Some(current_output) = cursor.output {
                if current_output != output_id {panic!()}
                else{
                    cursor.output = None;
                    Some(CursorEvent::Entered{id,output_id: current_output})
                }
            }
            else{None}
        }).flatten().into_iter()
        */
        vec![SeatEvent::from(CursorEvent::Entered {
            id,
            output_id: output_id,
        })]
        .into_iter()
    }

    pub fn move_cursor(
        &mut self,
        id: usize,
        position: pal::Position2D<i32>,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_mut(id)
            .map(|seat| {
                seat.cursor.as_mut().map(|cursor| {
                    cursor.position = position.clone();
                    SeatEvent::from(CursorEvent::Moved { id, position })
                })
            })
            .flatten()
            .into_iter()
    }
    pub fn focus_cursor(
        &mut self,
        id: usize,
        surface: Option<usize>,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_mut(id)
            .map(|seat| {
                seat.cursor
                    .as_mut()
                    .map(|cursor| {
                        if cursor.focus != surface {
                            cursor.focus = surface;
                            Some(SeatEvent::from(CursorEvent::Focus { id, surface }))
                        } else {
                            None
                        }
                    })
                    .flatten()
            })
            .flatten()
            .into_iter()
    }
    pub fn cursor_button(
        &mut self,
        id: usize,
        time: u32,
        code: u32,
        key: Option<pal::Button>,
        state: pal::State,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_ref(id)
            .map(|seat| {
                seat.cursor.as_ref().map(|_cursor| {
                    SeatEvent::from(CursorEvent::Button {
                        id,
                        time,
                        code,
                        key,
                        state,
                    })
                })
            })
            .flatten()
            .into_iter()
    }
    pub fn cursor_axis(
        &mut self,
        id: usize,
        time: u32,
        source: pal::AxisSource,
        direction: pal::AxisDirection,
        value: pal::AxisValue,
    ) -> impl Iterator<Item = SeatEvent> + Clone {
        self.seat_ref(id)
            .map(|seat| {
                seat.cursor.as_ref().map(|_cursor| {
                    SeatEvent::from(CursorEvent::Axis {
                        id,
                        time,
                        source,
                        direction,
                        value,
                    })
                })
            })
            .flatten()
            .into_iter()
    }
}
