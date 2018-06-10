pub const BOARD_SIZE: usize = 8;
pub const BOARD_SIZE_SQUARE: usize = BOARD_SIZE * BOARD_SIZE;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Black,
    White,
}

impl Cell {
    pub fn opposite(&self) -> Cell {
        match *self {
            Cell::Black => Cell::White,
            Cell::White => Cell::Black,
            Cell::Empty => {
                //js! { console.log(@{format!("Empty cell does not have opposite")});};
                panic!("No you can't")
            }
        }
    }
}

#[derive(Clone)]
pub struct Board {
    cells: [Cell; BOARD_SIZE_SQUARE],
}

impl Board {
    pub fn new() -> Self {
        let mut cells = [Cell::Empty; BOARD_SIZE_SQUARE];
        /*
            0 1 2 3 4 5 6 7
            8 9 0 1 2 3 4 5
            6 7 8 9 0 1 2 3
            4 5 6 B W 9 0 1
            2 3 4 W B 7 8 9
            0 1 2 3 4 5 6 7
            8 9 0 1 2 3 4 5
            6 7 8 9 0 1 2 3
        */
        let centerc = (BOARD_SIZE as f64 / 2.0).ceil() as usize;
        let centerf = centerc - 1; 

        cells[centerf * BOARD_SIZE + centerf] = Cell::Black;
        cells[centerf * BOARD_SIZE + centerc] = Cell::White;
        cells[centerc * BOARD_SIZE + centerf] = Cell::White;
        cells[centerc * BOARD_SIZE + centerc] = Cell::Black;

        Board {
            cells,
        }
    }

    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        let pos = x + y * BOARD_SIZE;
        &self.cells[pos]
    }

    pub fn rawcell(&self, pos: usize) -> &Cell {
        &self.cells[pos]
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) -> Result<(), ()> {
        let pos = x + y * 8 as usize;
        let mut collected: Vec<usize> = vec![];
        if !self.get_possibilities_collect_pos(cell, Some(pos), Some(&mut collected))
            .contains(&pos)
        {
            return Err(());
        }
        for pos in collected {
            self.cells[pos] = cell;
        }
        return Ok(());
    }

    pub fn get_possibilities(&self, cell: Cell) -> Vec<usize> {
        self.get_possibilities_collect_pos(cell, None, None)
    }

    pub fn get_possibilities_collect_pos(
        &self,
        cell: Cell,
        pos: Option<usize>,
        mut matched: Option<&mut Vec<usize>>,
    ) -> Vec<usize> {
        let mut result = vec![];
        let opposite = cell.opposite();

        for i in 0..BOARD_SIZE_SQUARE {
            if self.cells[i as usize] == cell {
                if let Some(r) = self.traverse_vertical_up(i, cell, pos, &mut matched, opposite) {
                    result.push(r);
                }

                if let Some(r) = self.traverse_vertical_down(i, cell, pos, &mut matched, opposite) {
                    result.push(r);
                }

                if let Some(r) = self.traverse_horizontal_up(i, cell, pos, &mut matched, opposite) {
                    result.push(r);
                }

                if let Some(r) = self.traverse_horizontal_down(i, cell, pos, &mut matched, opposite)
                {
                    result.push(r);
                }

                if let Some(r) =
                    self.traverse_horizontal_up_vertical_up(i, cell, pos, &mut matched, opposite)
                {
                    result.push(r);
                }

                if let Some(r) =
                    self.traverse_horizontal_down_vertical_up(i, cell, pos, &mut matched, opposite)
                {
                    result.push(r);
                }

                if let Some(r) =
                    self.traverse_horizontal_up_vertical_down(i, cell, pos, &mut matched, opposite)
                {
                    result.push(r);
                }

                if let Some(r) = self.traverse_horizontal_down_vertical_down(
                    i,
                    cell,
                    pos,
                    &mut matched,
                    opposite,
                ) {
                    result.push(r);
                }
            }
        }
        result
    }

    fn traverse_vertical_up(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        if pos < BOARD_SIZE {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos - BOARD_SIZE;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result < BOARD_SIZE {
                break;
            }
            collected.push(result.clone());
            result -= BOARD_SIZE;
            if self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

    fn traverse_vertical_down(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        if pos >= BOARD_SIZE_SQUARE - BOARD_SIZE {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos + BOARD_SIZE;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result > BOARD_SIZE_SQUARE - BOARD_SIZE {
                break;
            }
            collected.push(result.clone());
            result += BOARD_SIZE;
            if result > BOARD_SIZE_SQUARE - 1 || self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_up(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        if pos % BOARD_SIZE == (BOARD_SIZE - 1) {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos + 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == (BOARD_SIZE - 1) {
                break;
            }
            collected.push(result.clone());
            result += 1;
            if self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_down(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        if pos % BOARD_SIZE == 0 {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos - 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == 0 {
                break;
            }
            collected.push(result.clone());
            result -= 1;
            if self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_up_vertical_up(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        /*
            0 1 2 3 4 5 6 7
            8 9 0 1 2 3 4 5
            6 7 8 9 0 1 2 3
            4 5 6 B W 9 0 1
            2 3 4 W B 7 8 9
            0 1 2 3 4 5 6 7
            8 9 0 1 2 3 4 5
            6 7 8 9 0 1 2 3
        */

        if self.cells[pos] != color {
            return None;
        }
        if pos % BOARD_SIZE == (BOARD_SIZE - 1) {
            return None;
        }
        if pos < BOARD_SIZE {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos - BOARD_SIZE + 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == (BOARD_SIZE - 1) {
                break;
            }
            if result < BOARD_SIZE {
                break;
            }
            collected.push(result.clone());
            result -= BOARD_SIZE - 1;
            if self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_down_vertical_up(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        if pos % BOARD_SIZE == 0 {
            return None;
        }
        if pos < BOARD_SIZE {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos - BOARD_SIZE - 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == 0 {
                break;
            }
            if result < BOARD_SIZE {
                break;
            }
            collected.push(result.clone());
            result -= BOARD_SIZE + 1;
            if self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_up_vertical_down(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        if pos % BOARD_SIZE == (BOARD_SIZE - 1) {
            return None;
        }
        if pos >= BOARD_SIZE_SQUARE - BOARD_SIZE - 1 {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos + BOARD_SIZE + 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == (BOARD_SIZE - 1) {
                break;
            }
            if result > BOARD_SIZE_SQUARE - BOARD_SIZE {
                break;
            }
            collected.push(result.clone());
            result += BOARD_SIZE + 1;
            if result > BOARD_SIZE_SQUARE - 1 || self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_down_vertical_down(
        &self,
        pos: usize,
        color: Cell,
        match_pos: Option<usize>,
        matched: &mut Option<&mut Vec<usize>>,
        opposite: Cell,
    ) -> Option<usize> {
        /*
            0 1 2 3 4 5 6 7
            8 9 0 1 2 3 4 5
            6 7 8 9 0 1 2 3
            4 5 6 7 9 9 0 1    
            2 3 4 5 6 7 8 9
            0 1 B 3 4 5 6 7
            8 W 0 1 2 3 4 5
            6 7 8 9 0 1 2 3
        */

        if self.cells[pos] != color {
            return None;
        }
        if pos % BOARD_SIZE == 0 {
            return None;
        }
        if pos >= BOARD_SIZE_SQUARE - BOARD_SIZE {
            return None;
        }
        let mut collected: Vec<usize> = vec![];
        let mut result = pos + BOARD_SIZE - 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == 0 {
                break;
            }
            if result > BOARD_SIZE_SQUARE - BOARD_SIZE {
                break;
            }
            collected.push(result.clone());
            result += BOARD_SIZE - 1;
            if result > BOARD_SIZE_SQUARE - 1 || self.cells[result] != opposite {
                break;
            }
        }

        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            if let &mut Some(ref mut cells) = matched {
                if match_pos.is_some() && match_pos.unwrap() == result {
                    collected.push(result.clone());
                    cells.extend(collected.as_slice());
                }
            }
            return Some(result);
        }
        None
    }

}
