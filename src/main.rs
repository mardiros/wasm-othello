#[macro_use]
extern crate stdweb;

use std::rc::Rc;
use std::cell::RefCell;

use stdweb::traits::*;
use std::f64::consts::PI;

use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, EventListenerHandle, FillRule};

use stdweb::web::event::{ClickEvent, ConcreteEvent};

mod board;

pub use board::{Board, Cell, BOARD_SIZE};

pub struct BoardUI {
    board: Board,
    cell_width: f64,
    margin_width: f64,
}

impl BoardUI {
    pub fn new(board: Board, cell_width: u32, margin_width: u32) -> Self {
        BoardUI {
            board,
            cell_width: cell_width as f64,
            margin_width: margin_width as f64,
        }
    }

    fn paint_cell(
        &self,
        cell: &Cell,
        context: &CanvasRenderingContext2d,
        x: f64,
        y: f64,
        width: f64,
    ) {
        let mut radius = width * 0.4;
        match *cell {
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
                let cell = self.board.cell(x, y);
                if *cell != Cell::Empty {
                    self.paint_cell(cell, &context, posx + width / 2., posy + width / 2., width);
                }
            }
        }

        for pos in self.board.get_possibilities(player) {
            let width = self.cell_width - self.margin_width * 2.;
            let posx = (pos as f64 % BOARD_SIZE as f64).floor() * self.cell_width;
            let posy = (pos as f64 / BOARD_SIZE as f64).floor() * self.cell_width;
            self.paint_cell(
                self.board.rawcell(pos),
                &context,
                posx + width / 2.,
                posy + width / 2.,
                width,
            );
        }
    }
    fn can_play(&self, player: Cell) -> bool {
        self.board.get_possibilities(player).len() > 0
    }
}

pub struct Store {
    board: BoardUI,
    player: Cell,
    game_over: bool,
    cell_width: u32,
}

impl Store {
    fn new(cell_width: u32) -> Self {
        let board = Board::new();
        let board = BoardUI::new(board, cell_width, 1);
        Store {
            board,
            cell_width,
            game_over: false,
            player: Cell::Black,
        }
    }

    fn cell_width(&self) -> u32 {
        self.cell_width
    }

    fn paint(&self, context: &CanvasRenderingContext2d) {
        js! {
            console.log(@{format!("{:?}", self.player)})
        }
        self.board.paint(self.player, context)
    }

    fn clicked(&mut self, x: usize, y: usize) -> Result<(), ()> {
        if x > BOARD_SIZE && y > BOARD_SIZE {
            // prevent outside of the grid click
            return Ok(());
        }
        if let Ok(_) = self.board.board.set_cell(x, y, self.player) {
            if self.board.can_play(self.player.opposite()) {
                self.player = self.player.opposite();
                js!{ console.log(@{format!("Player {:?} play", self.player)}) }
            } else if self.board.can_play(self.player) {
                js!{ console.log(@{format!("Game Over")}) }
                return Err(());
            }
        }
        Ok(())
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

        let canvas_width = store.cell_width() as u32 * BOARD_SIZE as u32;

        canvas.set_width(canvas_width);
        canvas.set_height(canvas_width);

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
            let x = (event.offset_x() / store.cell_width() as f64) as usize;
            let y = (event.offset_y() / store.cell_width() as f64) as usize;
            let res = store.clicked(x, y);
            store.paint(&context);
            if res.is_err() {
                store.game_over = true;
            }
        });
    }

    fn paint(&mut self) {
        let context = self.canvas.context();
        let store = self.store.clone();
        store.borrow_mut().paint(&context);
    }
}

fn main() {
    js! {
        console.log("Welcome aboard")
    }
    let store = Store::new(60);
    let canvas = Canvas::new("#game", &store);
    let mut ac = AnimatedCanvas::new(store, canvas);
    ac.attach_event();
    ac.paint();
}
