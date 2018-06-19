use yew::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts::PI;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, EventListenerHandle, FillRule};

use stdweb::web::event::{ClickEvent, ConcreteEvent};

use super::context::Context;

pub use model::{BoardModel, Cell, BOARD_SIZE};

pub struct BoardUI {
    board: BoardModel,
    cell_width: f64,
    margin_width: f64,
}

impl BoardUI {
    pub fn new(board: BoardModel, cell_width: u32, margin_width: u32) -> Self {
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
    fn score(&self) -> (usize, usize) {
        self.board.score()
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
        let board = BoardModel::new();
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
        self.board.paint(self.player, context);
        let score = self.board.score();
        info!("Black: {} - White: {}", score.0, score.1);
    }

    fn play(&mut self, x: usize, y: usize) -> Result<(), ()> {
        if x > BOARD_SIZE && y > BOARD_SIZE {
            // prevent outside of the grid click
            return Err(());
        }
        if let Ok(_) = self.board.board.set_cell(x, y, self.player) {
            if self.board.can_play(self.player.opposite()) {
                self.player = self.player.opposite();
                info!("Player {:?} play", self.player);
            } else if !self.board.can_play(self.player) {
                info!("Game Over");
                self.game_over = true;
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

pub struct Board {
    canvas: Option<Canvas>,
    store: Rc<RefCell<Store>>,

    started: bool,
    onstart: Option<Callback<()>>,
    onclick: Option<Callback<(usize, usize)>>,
}

#[derive(PartialEq, Clone)]
pub struct Props {
    pub onstart: Option<Callback<()>>,
    pub onclick: Option<Callback<(usize, usize)>>,
}

impl Default for Props {
    fn default() -> Self {
        Props {
            onstart: None,
            onclick: None,
        }
    }
}

impl Board {
    fn new(
        store: Store,
        canvas: Option<Canvas>,
        onstart: Option<Callback<()>>,
        onclick: Option<Callback<(usize, usize)>>,
    ) -> Self {
        let store_rc = Rc::new(RefCell::new(store));
        Board {
            canvas: canvas,
            store: store_rc,
            onstart: onstart,
            onclick: onclick,
            started: false,
        }
    }
    fn paint(&mut self) {
        if let Some(ref canvas) = self.canvas {
            let context = canvas.context();
            let store = self.store.clone();
            store.borrow_mut().paint(&context);
        }
    }

    fn canvas_context(&self) -> CanvasRenderingContext2d {
        self.canvas.as_ref().unwrap().context()
    }

    fn view_start_button(&self) -> Html<Context, Self> {
        if !self.started {
            html!{
                <button
                    onclick=|_|Msg::AttachEvent,
                    >{"Start"}
                </button>
            }
        } else {
            html! {
                <>
                </>
            }
        }
    }
}

pub enum Msg {
    AttachEvent,
    Clicked(ClickEvent),
}

impl Component<Context> for Board {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, env: &mut Env<Context, Self>) -> Self {
        info!("Creating the board");
        let store = Store::new(60);
        Board::new(store, None, props.onstart, props.onclick)
    }

    fn update(&mut self, msg: Self::Message, _: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::AttachEvent => {
                let canvas = {
                    let store = self.store.borrow();
                    Canvas::new("#game", &store)
                };
                self.canvas = Some(canvas);
                self.paint();
                if let Some(ref onstart) = self.onstart {
                    onstart.emit(());
                }
                self.started = true;
            }
            Msg::Clicked(ref event) => {
                let mut store = self.store.borrow_mut();
                let x = (event.offset_x() / store.cell_width() as f64) as usize;
                let y = (event.offset_y() / store.cell_width() as f64) as usize;
                if let Ok(_) = store.play(x, y) {
                    let context = self.canvas_context();
                    store.paint(&context);
                    if let Some(ref onclick) = self.onclick {
                        onclick.emit((x, y));
                    }
                }
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties, _: &mut Env<Context, Self>) -> ShouldRender {
        self.onclick = props.onclick;
        true
    }
}

impl Renderable<Context, Board> for Board {
    fn view(&self) -> Html<Context, Self> {
        html! {
            <div>
                { self.view_start_button() }
                <canvas
                    id="game",
                    onclick=|event|Msg::Clicked(event),
                    ></canvas>
            </div>
        }
    }
}
