use stdweb::web::{CanvasRenderingContext2d, FillRule};
use std::f64::consts::PI;


#[derive(Clone, Copy)]
pub enum Cell {
    Empty,
    Black,
    White,
}

impl Cell {
    fn paint(&self, context: &CanvasRenderingContext2d, x: f64, y: f64, radius: f64) {
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
            Cell::Empty => return,
        }
        context.move_to(x + radius, y);
        context.arc(x, y, radius, 0., 2. * PI, false);
        context.fill(FillRule::NonZero);
        context.stroke();
    }
}

#[derive(Clone)]
pub struct Board {
    cells: [Cell; 64],  // hardcoded 8 * 8
    cell_width: f64,
    margin_width: f64,
}

impl Board {

    pub fn new(cell_width: u32, margin_width: u32) -> Self {
        let mut cells = [Cell::Empty; 64];
        cells[3 * 8 + 3] = Cell::Black;
        cells[3 * 8 + 4] = Cell::White;
        cells[4 * 8 + 3] = Cell::White;
        cells[4 * 8 + 4] = Cell::Black;

        Board {
            cells,
            cell_width: cell_width as f64,
            margin_width: margin_width as f64,
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[x + y * 8 as usize] = cell;
    }

    pub fn paint(&self, context: &CanvasRenderingContext2d) {
        let width = self.cell_width - self.margin_width * 2.;

        for x in 0..8 {
            for y in 0..8 {
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

                let pos = x + y * 8;
                self.cells[pos].paint(&context, posx + width / 2., posy + width / 2., width / 2.5);
            }
        }
    }
}
