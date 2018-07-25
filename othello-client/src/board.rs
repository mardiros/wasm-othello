use yew::prelude::*;

use std::f64::consts::PI;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, FillRule};
use stdweb::web::window;

use stdweb::web::event::ClickEvent;

use super::context::Context;

use model::{BoardModel, Cell, BOARD_SIZE};
use wscommand::Color;

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
        if player == Cell::Empty {
            return;
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
    current_player: Cell,
    local_player: Cell,
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
            current_player: Cell::Black,
            local_player: Cell::Empty, // will be ellected
        }
    }

    fn cell_width(&self) -> u32 {
        self.cell_width
    }

    fn paint(&self, context: &CanvasRenderingContext2d) {
        let player = if self.local_player == self.current_player {
            self.local_player
        } else {
            Cell::Empty
        };
        self.board.paint(player, context);
        let score = self.board.score();
        info!("Black: {} - White: {}", score.0, score.1);
    }

    fn play(&mut self, x: usize, y: usize) -> Result<(), ()> {
        if self.game_over {
            // you cannot play after the game is over
            return Err(());
        }
        if x > BOARD_SIZE && y > BOARD_SIZE {
            // prevent outside of the grid click
            return Err(());
        }
        if let Ok(_) = self.board.board.set_cell(x, y, self.current_player) {
            if self.board.can_play(self.current_player.opposite()) {
                self.current_player = self.current_player.opposite();
                info!("Player {:?} play", self.current_player);
            } else if !self.board.can_play(self.current_player) {
                info!("Game Over");
                self.game_over = true;
            }
        }
        Ok(())
    }

    fn score(&self) -> (usize, usize) {
        self.board.score()
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
}

#[derive(PartialEq)]
enum Status {
    BeingCreated,
    WaitingOpponent,
    Playing,
}

pub struct Board {
    canvas: Option<Canvas>,
    store: Store,
    cell_width: u32,
    status: Status,
    nickname: String,
    opponent: Option<String>,
    onstart: Option<Callback<()>>,
    onclick: Option<Callback<(usize, usize)>>,
    ongameover: Option<Callback<(usize, usize)>>,
}

#[derive(PartialEq, Clone)]
pub struct Props {
    pub color: Option<Color>,
    pub nickname: String,
    pub opponent: Option<String>,
    pub opponent_move: Option<(usize, usize)>,
    pub onstart: Option<Callback<()>>,
    pub onclick: Option<Callback<(usize, usize)>>,
    pub ongameover: Option<Callback<(usize, usize)>>,
}

impl Default for Props {
    fn default() -> Self {
        Props {
            nickname: "".to_string(),
            opponent: None,
            opponent_move: None,
            color: None,
            onstart: None,
            onclick: None,
            ongameover: None,
        }
    }
}

impl Board {
    fn paint(&mut self) {
        if let Some(ref canvas) = self.canvas {
            let context = canvas.context();
            self.store.paint(&context);
        }
    }

    fn canvas_context(&self) -> CanvasRenderingContext2d {
        self.canvas.as_ref().unwrap().context()
    }

    fn view_start_button(&self) -> Html<Context, Self> {
        if self.status == Status::BeingCreated {
            html!{
                <button
                    onclick=|_|Msg::AttachEvent,
                    >{"Join a board"}
                </button>
            }
        } else {
            html! {
                <>
                </>
            }
        }
    }
    fn view_playing(&self, cell: Cell) -> Html<Context, Self> {
        if self.store.current_player == cell {
            html! {
                <>
                {" ◀"}
                </>
            }
        } else {
            html! {
                <>
                </>
            }
        }
    }

    fn view_game_advancement(&self) -> Html<Context, Self> {

        let score = self.store.score();

        if self.store.game_over {
            let result = if score.0 == score.1 {
                "draw".to_string()
            }
            else if (self.store.local_player == Cell::Black && score.0 > score.1) ||
               (self.store.local_player == Cell::White && score.0 < score.1) {
                format!("{} win!", self.nickname)
            }
            else {
                format!("{} win!", self.opponent.as_ref().unwrap())
            };

            html! {
                <div>
                    {"Game Over "}
                    {" Black " } {score.0}
                    {" White " } {score.1}
                    <br/>
                    { result }
                    <br/>
                    <button
                        onclick=|_|Msg::RespawnBoard,
                        >{"Play again"}
                    </button>
                </div>
            }
        }

        else {
            let width = self.store.cell_width() as usize * BOARD_SIZE;
            let percent: f64 = score.0 as f64 * 100. / (score.0 + score.1) as f64;
            html! {
                <div style={ format!("max-width: {}px", width)},>
                    <div style="height:32px; border:1px solid black; background: white;",>
                        <div style={ format!("float left; background: black; height: 100%; width: {}%", percent)},>
                        </div>
                    </div>
                </div>
            }
        }
    }

    fn view_player_score(&self) -> Html<Context, Self> {
        match self.status {
            Status::BeingCreated => {
                html!{
                    <>
                    </>
                }
            }
            Status::WaitingOpponent => {
                match self.store.local_player {
                    Cell::Black => {
                        html!{
                            <ul>
                                <li>{"Black: "}{ self.nickname.as_str() }</li>
                                <li>{"White: Waiting for another player"}</li>
                            </ul>
                        }
                    }
                    Cell::White => {
                        html!{
                            <ul>
                                <li>{"Black: Waiting for another player"}</li>
                                <li>{"White: "}{ self.nickname.as_str() }</li>
                            </ul>
                        }
                    }
                    Cell::Empty => {
                        error!("No color defined for player while waiting for opponent");
                        html!{
                            <>
                            </>
                        }
                    }
                }
            }
            Status::Playing => match self.opponent {
                Some(ref opponent) => {

                    match self.store.local_player {
                        Cell::Black => {
                            html!{
                                <>
                                    <ul>
                                        <li>{"Black: "}{ self.nickname.as_str() }{ self.view_playing(Cell::Black) }</li>
                                        <li>{"White: "}{ opponent }{ self.view_playing(Cell::White) }</li>
                                    </ul>
                                    { self.view_game_advancement() }
                                </>
                            }
                        }
                        Cell::White => {
                            html!{
                                <>
                                    <ul>
                                        <li>{"Black: "}{ opponent } { self.view_playing(Cell::Black) }</li>
                                        <li>{"White: "}{ self.nickname.as_str() } { self.view_playing(Cell::White) }</li>
                                    </ul>
                                    { self.view_game_advancement() }
                                </>
                            }
                        }
                        Cell::Empty => {
                            error!("No color defined for player while playing");
                            html!{
                                <>
                                </>
                            }
                        }
                    }
                }
                None => {
                    html!{
                        <>
                            <p> { "Connection lost with opponent player" } </p>
                            <button
                                onclick=|_|Msg::RespawnBoard,
                                >{"Join another board"}
                            </button>
                        </>

                    }
                }
            },
        }
    }
}

pub enum Msg {
    AttachEvent,
    Clicked(ClickEvent),
    /// Restart the game
    RespawnBoard,
}

impl Component<Context> for Board {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _env: &mut Env<Context, Self>) -> Self {
        info!("Creating the board");
        let cell_width = ((window().inner_width() as u32) / (BOARD_SIZE as u32)) - 2;
        let cell_width = ::std::cmp::min(60, cell_width);
        let cell_width = ::std::cmp::max(32, cell_width);
        Board {
            canvas: None,
            cell_width: cell_width,
            store: Store::new(cell_width),
            nickname: props.nickname,
            opponent: props.opponent,
            onstart: props.onstart,
            onclick: props.onclick,
            ongameover: props.ongameover,
            status: Status::BeingCreated,
        }
    }

    fn update(&mut self, msg: Self::Message, _: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::AttachEvent => {
                let canvas = {
                    Canvas::new("#game", &self.store)
                };
                self.canvas = Some(canvas);
                self.paint();
                if let Some(ref onstart) = self.onstart {
                    onstart.emit(());
                }
            }
            Msg::Clicked(ref event) => {
                if self.opponent == None {
                    info!("Clicked but waiting for an opponent");
                    return false;
                }

                // only the play who play should count
                if self.store.current_player != self.store.local_player {
                    info!("Clicked but it is the turn of the opponent");
                    return false;
                }

                let x = (event.offset_x() / self.store.cell_width() as f64) as usize;
                let y = (event.offset_y() / self.store.cell_width() as f64) as usize;
                if let Ok(_) = self.store.play(x, y) {
                    let context = self.canvas_context();
                    self.store.paint(&context);
                    if let Some(ref onclick) = self.onclick {
                        onclick.emit((x, y));
                    }
                    if self.store.game_over {
                        if let Some(ref ongameover) = self.ongameover {
                            ongameover.emit((x, y));
                        }
                    }
                }
            }
            Msg::RespawnBoard => {
                self.store = Store::new(self.cell_width);
                let canvas = Canvas::new("#game", &self.store);
                self.canvas = Some(canvas);
                self.opponent = None;
                self.status = Status::BeingCreated;
                if let Some(ref onstart) = self.onstart {
                    onstart.emit(());
                }
                let context = self.canvas_context();
                self.store.paint(&context);
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties, _: &mut Env<Context, Self>) -> ShouldRender {
        self.opponent = props.opponent;
        self.nickname = props.nickname;

        if let Some(color) = props.color {
            self.status = Status::WaitingOpponent;
            match color {
                Color::White => {
                    self.store.local_player = Cell::White;
                }
                Color::Black => {
                    self.store.local_player = Cell::Black;
                }
            }
            let context = self.canvas_context();
            self.store.paint(&context);
        }

        if self.opponent.is_some() {
            self.status = Status::Playing;
        }

        if let Some((x, y)) = props.opponent_move {
            if let Ok(_) = self.store.play(x, y) {
                let context = self.canvas_context();
                self.store.paint(&context);
            }
        }
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
                { self.view_player_score() }
            </div>
        }
    }
}
