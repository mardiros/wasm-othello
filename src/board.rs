use stdweb::web::{CanvasRenderingContext2d, FillRule};
use std::f64::consts::PI;

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
            Cell::Empty => panic!("No you can't"),
        }
    }

    fn paint(&self, context: &CanvasRenderingContext2d, x: f64, y: f64, width: f64) {
        let mut radius = width * 0.4;
        match *self {
            Cell::Black => {
                context.begin_path();
                context.set_fill_style_color("#111");
                context.set_stroke_style_color("#000");
            }
            Cell::White => {
                context.begin_path();
                context.set_fill_style_color("#eee");
                context.set_stroke_style_color("#fff");
            }
            Cell::Empty => {
                radius = radius * 0.3;
                context.begin_path();
                context.set_fill_style_color("#44f");
                context.set_stroke_style_color("#aaf");
            }
        }
        context.move_to(x + radius, y);
        context.arc(x, y, radius, 0., 2. * PI, false);
        context.fill(FillRule::NonZero);
        context.stroke();
    }
}

#[derive(Clone)]
pub struct Board {
    cells: [Cell; BOARD_SIZE_SQUARE],
    cell_width: f64,
    margin_width: f64,
}

impl Board {
    pub fn new(cell_width: u32, margin_width: u32) -> Self {
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
        // fix that
        cells[3 * BOARD_SIZE + 3] = Cell::Black;
        cells[3 * BOARD_SIZE + 4] = Cell::White;
        cells[4 * BOARD_SIZE + 3] = Cell::White;
        cells[4 * BOARD_SIZE + 4] = Cell::Black;

        Board {
            cells,
            cell_width: cell_width as f64,
            margin_width: margin_width as f64,
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[x + y * 8 as usize] = cell;
    }

    pub fn get_possibilities(&self, cell: Cell) -> Vec<usize> {
        let mut result = vec![];
        let opposite = cell.opposite();

        for i in 0..BOARD_SIZE_SQUARE {
            if self.cells[i as usize] == cell {
                if let Some(r) = self.traverse_vertical_up(i, cell, opposite) {
                    result.push(r);
                }
                if let Some(r) = self.traverse_vertical_down(i, cell, opposite) {
                    result.push(r);
                }
                if let Some(r) = self.traverse_horizontal_up(i, cell, opposite) {
                    result.push(r);
                }
                if let Some(r) = self.traverse_horizontal_down(i, cell, opposite) {
                    result.push(r);
                }
            }
        }
        result
    }

    fn traverse_vertical_up(&self, pos: usize, color: Cell, opposite: Cell) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        let mut result = pos;
        if result < BOARD_SIZE {
            return None;
        }
        result -= BOARD_SIZE;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result < BOARD_SIZE {
                break;
            }
            result -= BOARD_SIZE;
            if self.cells[result] != opposite {
                break;
            }
        }
        if self.cells[result] == Cell::Empty {
            return Some(result);
        }
        None
    }

    fn traverse_vertical_down(&self, pos: usize, color: Cell, opposite: Cell) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        let mut result = pos;
        if result > BOARD_SIZE_SQUARE - BOARD_SIZE {
            return None;
        }
        result += BOARD_SIZE;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result > BOARD_SIZE_SQUARE - BOARD_SIZE {
                break;
            }
            result += BOARD_SIZE;
            if self.cells[result] != opposite {
                break;
            }
        }
        if self.cells[result] == Cell::Empty {
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_up(&self, pos: usize, color: Cell, opposite: Cell) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        let mut result = pos;
        if result % BOARD_SIZE == (BOARD_SIZE - 1) {
            return None;
        }
        result += 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == (BOARD_SIZE - 1) {
                break;
            }
            result += 1;
            if self.cells[result] != opposite {
                break;
            }
        }
        if result > 0 && self.cells[result] == Cell::Empty {
            return Some(result);
        }
        None
    }

    fn traverse_horizontal_down(&self, pos: usize, color: Cell, opposite: Cell) -> Option<usize> {
        if self.cells[pos] != color {
            return None;
        }
        let mut result = pos;
        if result % BOARD_SIZE == 0 {
            return None;
        }
        result -= 1;
        if self.cells[result] != opposite {
            return None;
        }
        loop {
            if result % BOARD_SIZE == 0 {
                break;
            }
            result -= 1;
            if self.cells[result] != opposite {
                break;
            }
        }
        if result < BOARD_SIZE_SQUARE && self.cells[result] == Cell::Empty {
            return Some(result);
        }
        None
    }

    pub fn paint(&self, player: Cell, context: &CanvasRenderingContext2d) {
        let width = self.cell_width - self.margin_width * 2.;

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let posx = self.cell_width * (x as f64);
                let posy = self.cell_width * (y as f64);

                // Borders could use stroke instead
                context.set_fill_style_color("#333");
                context.fill_rect(posx, posy, self.cell_width, self.cell_width);

                context.set_fill_style_color("#383");
                context.fill_rect(
                    posx + self.margin_width,
                    posy + self.margin_width,
                    width,
                    width,
                );

                let pos = x + y * BOARD_SIZE;
                if self.cells[pos] != Cell::Empty {
                    self.cells[pos].paint(&context, posx + width / 2., posy + width / 2., width);
                }
            }
        }

        for pos in self.get_possibilities(player) {
            let width = self.cell_width - self.margin_width * 2.;
            let posx = (pos as f64 % BOARD_SIZE as f64).floor() * self.cell_width;
            let posy = (pos as f64 / BOARD_SIZE as f64).floor() * self.cell_width;

            self.cells[pos].paint(&context, posx + width / 2., posy + width / 2., width);
        }
    }
}
