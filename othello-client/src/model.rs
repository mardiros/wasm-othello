use std::collections::HashSet;

pub const BOARD_SIZE: usize = 8;
pub const BOARD_SIZE_SQUARE: usize = BOARD_SIZE * BOARD_SIZE;

macro_rules! traverse_board {

    ($name:ident, $self_:ident, $result:ident,  $step: expr, $( $test:expr ),* )  => {

        fn $name(
            &$self_,
            pos: usize,
            color: Cell,
            match_pos: Option<usize>,
            matched: &mut Vec<usize>,
            opposite: Cell,
        ) -> Option<usize> {

            let mut $result = pos;

            if $self_.cells[pos] != color {
                return None;
            }
            $(
                if $test {
                    return None;
                }
            )*
            let mut collected: Vec<usize> = Vec::new();
            $step;
            if $result >=BOARD_SIZE_SQUARE {
                return None;
            }

            if $self_.cells[$result] != opposite {
                return None;
            }
            loop {
                $(
                    if $test {
                        break;
                    }
                )*
                collected.push($result.clone());
                $step;
                if $result > BOARD_SIZE_SQUARE - 1 || $self_.cells[$result] != opposite {
                    break;
                }
            }

            if $result < BOARD_SIZE_SQUARE && $self_.cells[$result] == Cell::Empty {
                if match_pos.is_some() && match_pos.unwrap() == $result {
                    collected.push($result.clone());
                    matched.extend(collected.as_slice());
                }
                return Some($result);
            }
            None
        }
    }
}

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
pub struct BoardModel {
    cells: [Cell; BOARD_SIZE_SQUARE],
}

impl BoardModel {
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

        BoardModel { cells }
    }

    #[cfg(test)]
    fn from_string(boardstr: &str) -> Self {
        let mut cells = [Cell::Empty; BOARD_SIZE_SQUARE];
        let boardstr = boardstr.replace("\n", "");
        let boardstr = boardstr.replace(" ", "");
        for (idx, chr) in boardstr.chars().enumerate() {
            match chr {
                'W' => cells[idx] = Cell::White,
                'B' => cells[idx] = Cell::Black,
                _ => {}
            }
        }
        BoardModel { cells }
    }

    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        let pos = x + y * BOARD_SIZE;
        &self.cells[pos]
    }

    pub fn rawcell(&self, pos: usize) -> &Cell {
        &self.cells[pos]
    }

    pub fn score(&self) -> (usize, usize) {
        let mut score = (0, 0);
        self.cells.iter().for_each(|cell| match cell {
            &Cell::Black => score.0 += 1,
            &Cell::White => score.1 += 1,
            _ => {}
        });
        score
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) -> Result<(), ()> {
        let pos = x + y * 8 as usize;
        let mut collected: Vec<usize> = Vec::new();
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
        let mut result: HashSet<usize> = HashSet::new();
        let opposite = cell.opposite();
        let mut dup_matched: Vec<usize> = Vec::new();

        for i in 0..BOARD_SIZE_SQUARE {
            if self.cells[i as usize] == cell {
                if let Some(r) = self.traverse_vertical_up(i, cell, pos, &mut dup_matched, opposite)
                {
                    result.insert(r);
                }

                if let Some(r) =
                    self.traverse_vertical_down(i, cell, pos, &mut dup_matched, opposite)
                {
                    result.insert(r);
                }

                if let Some(r) =
                    self.traverse_horizontal_up(i, cell, pos, &mut dup_matched, opposite)
                {
                    result.insert(r);
                }

                if let Some(r) =
                    self.traverse_horizontal_down(i, cell, pos, &mut dup_matched, opposite)
                {
                    result.insert(r);
                }

                if let Some(r) = self.traverse_horizontal_up_vertical_up(
                    i,
                    cell,
                    pos,
                    &mut dup_matched,
                    opposite,
                ) {
                    result.insert(r);
                }

                if let Some(r) = self.traverse_horizontal_down_vertical_up(
                    i,
                    cell,
                    pos,
                    &mut dup_matched,
                    opposite,
                ) {
                    result.insert(r);
                }

                if let Some(r) = self.traverse_horizontal_up_vertical_down(
                    i,
                    cell,
                    pos,
                    &mut dup_matched,
                    opposite,
                ) {
                    result.insert(r);
                }

                if let Some(r) = self.traverse_horizontal_down_vertical_down(
                    i,
                    cell,
                    pos,
                    &mut dup_matched,
                    opposite,
                ) {
                    result.insert(r);
                }
            }
        }
        if let Some(ref mut m) = matched {
            let dedup: HashSet<_> = dup_matched.drain(..).collect();
            m.extend(dedup.into_iter())
        }
        let mut res: Vec<usize> = Vec::with_capacity(result.len());
        res.extend(result.into_iter());
        res
    }

    traverse_board!(
        traverse_vertical_up,
        self,
        result,
        result -= BOARD_SIZE,
        result < BOARD_SIZE
    );

    traverse_board!(
        traverse_vertical_down,
        self,
        result,
        result += BOARD_SIZE,
        result > BOARD_SIZE_SQUARE - BOARD_SIZE
    );

    traverse_board!(
        traverse_horizontal_up,
        self,
        result,
        result += 1,
        result % BOARD_SIZE == (BOARD_SIZE - 1)
    );

    traverse_board!(
        traverse_horizontal_down,
        self,
        result,
        result -= 1,
        result % BOARD_SIZE == 0
    );

    traverse_board!(
        traverse_horizontal_up_vertical_up,
        self,
        result,
        result -= BOARD_SIZE - 1,
        result % BOARD_SIZE == (BOARD_SIZE - 1),
        result < BOARD_SIZE
    );

    traverse_board!(
        traverse_horizontal_down_vertical_up,
        self,
        result,
        result -= BOARD_SIZE + 1,
        result % BOARD_SIZE == 0,
        result < BOARD_SIZE
    );

    traverse_board!(
        traverse_horizontal_up_vertical_down,
        self,
        result,
        result += BOARD_SIZE + 1,
        result % BOARD_SIZE == (BOARD_SIZE - 1),
        result > BOARD_SIZE_SQUARE - BOARD_SIZE
    );

    traverse_board!(
        traverse_horizontal_down_vertical_down,
        self,
        result,
        result += BOARD_SIZE - 1,
        result % BOARD_SIZE == 0,
        result > BOARD_SIZE_SQUARE - BOARD_SIZE
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_possibilities_diag() {
        let board = BoardModel::from_string(
            r#"
            . . . . . . . .
            . W . . . . W .
            . . B . . B . .
            . . . . . . . .
            . . . . . . . .
            . . B . . B . .
            . W . . . . W .
            . . . . . . . .
            "#,
        );

        let empty: Vec<usize> = vec![];
        for i in 0..64 {
            let mut collected: Vec<usize> = vec![];
            let mut pos =
                board.get_possibilities_collect_pos(Cell::Black, Some(i), Some(&mut collected));
            pos.sort();
            assert_eq!(pos, vec![0, 7, 56, 63]);
            collected.sort();
            match i {
                0 => assert_eq!(collected, vec![0, 9]),
                7 => assert_eq!(collected, vec![7, 14]),
                56 => assert_eq!(collected, vec![49, 56]),
                63 => assert_eq!(collected, vec![54, 63]),
                _ => {
                    assert_eq!(collected, empty);
                }
            }
        }
    }

    #[test]
    fn test_possibilities_horizontal_vertical() {
        let board = BoardModel::from_string(
            r#"
            . . . . . . . .
            . . W . . W . .
            . W B . . B W .
            . . . . . . . .
            . . . . . . . .
            . W B . . B W .
            . . W . . W . .
            . . . . . . . .
            "#,
        );

        let empty: Vec<usize> = vec![];
        for i in 0..64 {
            let mut collected: Vec<usize> = vec![];
            let mut pos =
                board.get_possibilities_collect_pos(Cell::Black, Some(i), Some(&mut collected));
            pos.sort();
            assert_eq!(pos, vec![2, 5, 16, 23, 40, 47, 58, 61]);
            collected.sort();
            match i {
                2 => assert_eq!(collected, vec![2, 10]),
                5 => assert_eq!(collected, vec![5, 13]),
                16 => assert_eq!(collected, vec![16, 17]),
                23 => assert_eq!(collected, vec![22, 23]),
                40 => assert_eq!(collected, vec![40, 41]),
                47 => assert_eq!(collected, vec![46, 47]),
                58 => assert_eq!(collected, vec![50, 58]),
                61 => assert_eq!(collected, vec![53, 61]),
                _ => {
                    assert_eq!(collected, empty);
                }
            }
        }
    }

    #[test]
    fn test_possibilities_diag_splitted() {
        let board = BoardModel::from_string(
            r#"
            B . . . . . . .
            . W . . . . . .
            . . B . . . . .
            . . . W . . . .
            . . . . W . . .
            . . . . . W . .
            . . . . . . W .
            . . . . . . . .
            "#,
        );

        let empty: Vec<usize> = vec![];
        for i in 0..64 {
            let mut collected: Vec<usize> = vec![];
            let pos =
                board.get_possibilities_collect_pos(Cell::Black, Some(i), Some(&mut collected));
            assert_eq!(pos, vec![63]);
            collected.sort();
            match i {
                63 => assert_eq!(collected, vec![27, 36, 45, 54, 63]),
                _ => {
                    assert_eq!(collected, empty);
                }
            }
        }
    }

    #[test]
    fn test_possibilities_diag_splitted2() {
        let board = BoardModel::from_string(
            r#"
            . . . . . . . .
            . W . . . . . .
            . . W . . . . .
            . . . B . . . .
            . . . . B . . .
            . . . . . W . .
            . . . . . . B .
            . . . . . . . .
            "#,
        );

        let empty: Vec<usize> = vec![];
        for i in 0..64 {
            let mut collected: Vec<usize> = vec![];
            let pos =
                board.get_possibilities_collect_pos(Cell::Black, Some(i), Some(&mut collected));
            assert_eq!(pos, vec![0]);
            collected.sort();
            match i {
                0 => assert_eq!(collected, vec![0, 9, 18]),
                _ => {
                    assert_eq!(collected, empty);
                }
            }
        }
    }

    #[test]
    fn test_possibilities_diag_splitted3() {
        let board = BoardModel::from_string(
            r#"
            . . . . . . . .
            . . . . . . W .
            . . . . . B . .
            . . . . W . . .
            . . . W . . . .
            . . W . . . . .
            . B . . . . . .
            . . . . . . . .
            "#,
        );

        let empty: Vec<usize> = vec![];
        for i in 0..64 {
            let mut collected: Vec<usize> = vec![];
            let pos =
                board.get_possibilities_collect_pos(Cell::Black, Some(i), Some(&mut collected));
            assert_eq!(pos, vec![7]);
            collected.sort();
            match i {
                7 => assert_eq!(collected, vec![7, 14]),
                _ => {
                    assert_eq!(collected, empty);
                }
            }
        }
    }

    #[test]
    fn test_possibilities_many_directions() {
        let board = BoardModel::from_string(
            r#"
            . . B . . . . .
            . . W . . . . .
            B W . W W W W B
            . . W W . . . .
            . . W . W . . .
            . . W . . W . .
            . . W . . . W .
            . . B . . . . W
            "#,
        );

        let empty: Vec<usize> = vec![];
        for i in 0..64 {
            let mut collected: Vec<usize> = vec![];
            let mut pos =
                board.get_possibilities_collect_pos(Cell::Black, Some(i), Some(&mut collected));
            pos.sort();
            assert_eq!(pos, vec![18]);
            collected.sort();
            match i {
                18 => assert_eq!(collected, vec![10, 17, 18, 19, 20, 21, 22, 26, 34, 42, 50]),
                _ => {
                    assert_eq!(collected, empty);
                }
            }
        }
    }

    #[test]
    fn test_score() {
        let board = BoardModel::from_string(
            r#"
            W B B B B B B B
            W W W B B B B B
            B W W W W W W B
            B W W W B B B B
            B W W W W W W W
            B W W W B W B B
            B W W W B B W W
            B W B B B B B W
            "#,
        );
        let score = board.score();
        assert_eq!(score, (33, 31))
    }

    #[test]
    fn test_overflow() {
        let board = BoardModel::from_string(
            r#"
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . B W . . .
            . . B W B . . .
            . . W . . . . .
            B W B . . . . .
            W . . . . . . .
            "#,
        );

        let empty: Vec<usize> = vec![];
        for i in 0..64 {
            let mut collected: Vec<usize> = vec![];
            let mut pos =
                board.get_possibilities_collect_pos(Cell::White, Some(i), Some(&mut collected));
            pos.sort();
            assert_eq!(pos, vec![19, 26, 33, 37, 40, 44, 51, 58]);
            collected.sort();
            match i {
                19 => {
                    assert_eq!(collected, vec![19, 27]);
                }
                26 => {
                    assert_eq!(collected, vec![26, 27, 34]);
                }
                33 => {
                    assert_eq!(collected, vec![33, 34]);
                }
                37 => {
                    assert_eq!(collected, vec![36, 37]);
                }
                40 => {
                    assert_eq!(collected, vec![40, 48]);
                }
                44 => {
                    assert_eq!(collected, vec![36, 44]);
                }
                51 => {
                    assert_eq!(collected, vec![50, 51]);
                }
                58 => {
                    assert_eq!(collected, vec![50, 58]);
                }
                _ => {
                    assert_eq!(collected, empty);
                }
            }
        }
    }

}
