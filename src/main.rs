//#[macro_use]
extern crate stdweb;

use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts::PI;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, EventListenerHandle, FillRule};

use stdweb::web::event::{ClickEvent, ConcreteEvent};

#[derive(Clone, Copy)]
enum Cell {
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
struct Board {
    cells: [Cell; 64],
    cell_width: f64,
    margin_width: f64,
}

impl Board {
    fn new(cell_width: u32, margin_width: u32) -> Self {
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

    fn paint(&self, context: &CanvasRenderingContext2d) {
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

#[derive(Clone)]
struct Store {
    board: Board,
    game_over: bool,
    cell_width: u32,
    row_count: u32,
}

impl Store {
    fn new(cell_width: u32, row_count: u32) -> Store {
        let board = Board::new(cell_width, 1);
        Store {
            board,
            cell_width,
            row_count,
            game_over: false,
        }
    }

    fn paint(&self, context: &CanvasRenderingContext2d) {
        self.board.paint(&context)
    }

    fn clicked(&mut self, x: usize, y: usize) {
        self.board.cells[x + y * self.row_count as usize] = Cell::White;
    }
}

struct Canvas {
    canvas: CanvasElement,
}

impl Canvas {
    fn new(selector: &str, store: &Store) -> Canvas {
        let canvas: CanvasElement = document()
            .query_selector(selector)
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();

        let canvas_width = store.cell_width * store.row_count;
        let canvas_height = store.cell_width * store.row_count;

        canvas.set_width(canvas_width);
        canvas.set_height(canvas_height);

        Canvas { canvas }
    }

    fn context(&self) -> CanvasRenderingContext2d {
        self.canvas.get_context().unwrap()
    }

    fn add_event_listener<T, F>(&self, listener: F) -> EventListenerHandle
    where
        T: ConcreteEvent,
        F: FnMut(T) + 'static,
    {
        self.canvas.add_event_listener(listener)
    }
}

struct AnimatedCanvas {
    canvas: Canvas,
    store: Rc<RefCell<Store>>,
}

impl AnimatedCanvas {
    fn new(store: Store, canvas: Canvas) -> AnimatedCanvas {
        let store_rc = Rc::new(RefCell::new(store));
        AnimatedCanvas {
            canvas,
            store: store_rc,
        }
    }
    fn attach_event(&mut self) {
        let context = self.canvas.context();
        let store = self.store.clone();
        self.canvas.add_event_listener(move |event: ClickEvent| {
            let mut store = store.borrow_mut();
            let x = (event.offset_x() as f64 / store.board.cell_width) as usize;
            let y = (event.offset_y() as f64 / store.board.cell_width) as usize;
            store.clicked(x, y);
            store.paint(&context);
        });
    }

    fn paint(&mut self) {
        let context = self.canvas.context();
        let store = self.store.clone();
        store.borrow_mut().paint(&context);
    }
}

fn main() {
    let store = Store::new(60, 8);
    let canvas = Canvas::new("#game", &store);
    let mut ac = AnimatedCanvas::new(store, canvas);
    ac.attach_event();
    ac.paint();
}
