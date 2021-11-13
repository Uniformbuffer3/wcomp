

#[derive(Debug,Clone)]
pub enum CursorEvent<C: Clone> {
    Added{id: usize, handle: C, position: pal::Position2D<i32>, image: Option<usize>},
    Removed{id: usize},
    Moved{id: usize,position: pal::Position2D<i32>},
    Button {id: usize, time: u32,code: u32,key: Option<pal::Button>,state: pal::State},
    Focus{id: u32},
    Entered{id: usize, output_id: usize},
    Left{id: usize, output_id: usize}
}

#[derive(Debug,Clone)]
pub struct Cursor<C: Clone> {
    id: usize,
    handle: C,
    position: pal::Position2D<i32>,
    image: Option<usize>,
    output: Option<usize>
}
impl<C: Clone> Cursor<C> {
    pub fn output(&self)->Option<usize> {self.output}
}

#[derive(Debug)]
pub struct CursorManager<C: Clone> {
    cursors: Vec<Cursor<C>>
}

impl<C: Clone> CursorManager<C> {
    pub fn new()->Self {
        let cursors = Vec::new();
        Self {cursors}
    }

    pub fn get_cursor_size(&self)->pal::Size2D<u32> {pal::Size2D{width: 10,height: 10}}

    pub fn enter_cursor(&mut self, id: usize, output_id: usize)->impl Iterator<Item = CursorEvent<C>>+Clone{
        self.cursor_mut(id).map(|cursor|{
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
        self.cursor_mut(id).map(|cursor|{
            cursor.position = position;
            CursorEvent::Moved{id,position}
        }).into_iter()
    }

    pub fn cursor_ref(&self,id: usize)->Option<&Cursor<C>> {
        self.cursors.iter().find_map(|cursor|{
            if cursor.id == id {Some(cursor)}
            else{None}
        })
    }
    fn cursor_mut(&mut self,id: usize)->Option<&mut Cursor<C>> {
        self.cursors.iter_mut().find_map(|cursor|{
            if cursor.id == id {Some(cursor)}
            else{None}
        })
    }
}
