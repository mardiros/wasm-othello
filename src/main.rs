#[macro_use]
extern crate stdweb;

use std::rc::Rc;
use std::cell::RefCell;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, EventListenerHandle};

use stdweb::web::event::{ClickEvent, ConcreteEvent};

mod board;

pub use board::{Board, Cell, BOARD_SIZE};

#[derive(Clone)]
pub struct Store {
    board: Board,
    player: Cell,
    game_over: bool,
    cell_width: u32,
}

impl Store {
    pub fn new(cell_width: u32) -> Store {
        let board = Board::new(cell_width, 1);
        Store {
            board,
            cell_width,
            game_over: false,
            player: Cell::Black,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn cell_width(&self) -> u32 {
        self.cell_width
    }

    pub fn paint(&self, context: &CanvasRenderingContext2d) {
        js! {
            console.log(@{format!("{:?}", self.player)})
        }
        self.board.paint(self.player, context)
    }

    pub fn clicked(&mut self, x: usize, y: usize) {
        if x > BOARD_SIZE && y > BOARD_SIZE {
            // prevent outside of the grid click
            return;
        }
        self.board.set_cell(x, y, self.player);
        self.player = self.player.opposite();
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
        let canvas_height = store.cell_width() as u32 * BOARD_SIZE as u32;

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
            let x = (event.offset_x() / store.cell_width() as f64) as usize;
            let y = (event.offset_y() / store.cell_width() as f64) as usize;
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
    js! {
        console.log("Welcome aboard")
    }
    let store = Store::new(60);
    let canvas = Canvas::new("#game", &store);
    let mut ac = AnimatedCanvas::new(store, canvas);
    ac.attach_event();
    ac.paint();
}
