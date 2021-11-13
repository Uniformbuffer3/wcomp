

#[derive(Debug,Clone)]
pub enum SeatEvent {
    Added{id: usize, handle: C, position: pal::Position2D<i32>, image: Option<usize>},
    Removed{id: usize},
    Moved{id: usize,position: pal::Position2D<i32>},
    Button {id: usize, time: u32,code: u32,key: Option<pal::Button>,state: pal::State},
    Focus{id: u32},
    Entered{id: usize, output_id: usize},
    Left{id: usize, output_id: usize}
}

#[derive(Debug,Clone)]
pub struct Seat {
    id: usize,
}

#[derive(Debug)]
pub struct SeatManager {
    seats: Vec<Seat>
}

impl SeatManager {
    pub fn new()->Self {
        let seats = Vec::new();
        Self {seats}
    }

    pub fn get_cursor_size(&self)->pal::Size2D<u32> {pal::Size2D{width: 10,height: 10}}

    pub fn enter_cursor(&mut self, id: usize, output_id: usize)->impl Iterator<Item = CursorEvent<C>>+Clone{
        self.seat_mut(id).map(|cursor|{
            cursor.output=Some(output_id);
            CursorEvent::Entered{id,output_id}
        }).into_iter()
    }

    pub fn left_cursor(&mut self, id: usize, output_id: usize)->impl Iterator<Item = CursorEvent<C>>+Clone{
        self.cursor_mut(id).map(|cursor|{
            if let Some(current_output) = cursor.output {
                if current_output != output_id {panic!()}
                else{
                    cursor.output = None;
                    Some(CursorEvent::Entered{id,output_id: current_output})
                }
            }
            else{None}
        }).flatten().into_iter()
    }

    pub fn add_cursor(&mut self, id: usize, handle: C, position: pal::Position2D<i32>, image: Option<usize>)->impl Iterator<Item = CursorEvent<C>>+Clone{
        let output = None;
        let cursor = Cursor {id,handle: handle.clone(),position,image,output};
        self.cursors.push(cursor.clone());
        vec![CursorEvent::Added{id,handle,position,image}].into_iter()
    }

    pub fn del_cursor(&mut self, id: usize)->impl Iterator<Item = CursorEvent<C>>+Clone{
        self.cursors.iter().position(|cursor|cursor.id == id).map(|index|{
            let cursor = self.cursors.remove(index);
            CursorEvent::Removed{id}
        }).into_iter()
    }

    pub fn move_cursor(&mut self, id: usize, position: pal::Position2D<i32>)->impl Iterator<Item = CursorEvent<C>>+Clone{
        self.seat_mut(id).map(|cursor|{
            seat.position = position;
            CursorEvent::Moved{id,position}
        }).into_iter()
    }

    pub fn seat_ref(&self,id: usize)->Option<&Seat> {
        self.seats.iter().find_map(|seat|{
            if seat.id == id {Some(seat)}
            else{None}
        })
    }
    fn seat_mut(&mut self,id: usize)->Option<&mut Seat> {
        self.cursors.iter_mut().find_map(|seat|{
            if seat.id == id {Some(seat)}
            else{None}
        })
    }
}
