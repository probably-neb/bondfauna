use std::{iter::zip, rc::Rc};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use crate::letters::Chr;
use roget::Correctness;
use yew::prelude::*;

#[derive(Default, Clone, Copy, PartialEq)]
struct TileState {
    current: Option<Chr>,
    goodness: Option<Correctness>,
}

#[derive(Properties, PartialEq, Eq)]
pub struct TileProps {
    col: usize,
    row: usize,
}

#[function_component]
pub fn Tile(props: &TileProps) -> Html {
    let state = use_state_eq(TileState::default);
    let ctx = use_context::<GameContext>().unwrap();
    state.set(ctx.board_state[props.row][props.col]);
    // let pos = format!("{},{}", props.col, props.row);
    let disp = state.current.map(|c| c.to_char()).unwrap_or(' ');
    // let disp = "_";
    let data_guess = if let Some(correctness) = state.goodness {
        format!("{:?}", correctness)
    } else {
        "empty".to_string()
    };
    html!(
        <div class="tile" data-guess={data_guess}>{disp}</div>
    )
}

#[derive(Properties, PartialEq, Eq)]
pub struct RowProps {
    row: usize,
}

#[function_component]
pub fn Row(props: &RowProps) -> Html {
    // let row_height = (10.0 / 6.0) as u32;
    let row = props.row;
    let boxes = (0..5).map(|col| html! { <Tile {row} {col} /> });
    html!(
        <div class="row">{boxes.collect::<Html>()}</div>
    )
}

// #[derive(Properties, PartialEq)]
// pub struct InnerBoardProps {
//     parent_size: (u32,u32)
// }

#[function_component]
pub fn InnerBoard(/* props: &InnerBoardProps */) -> Html {
    let rows = (0..6).map(|row| html! { <Row {row}/> });
    let div_ref = use_node_ref();
    let get_updated_dimensions = {
        let div_ref = div_ref.clone();
        move || -> (i32, i32) {
            // r = (n = Math.floor(e.clientHeight * (5 / 6)), i = t, Math.min(Math.max(n, i), 350));
            //             var n,
            //             i;
            // const l = 6 * Math.floor(r / 5);

            let h: Option<i32> = div_ref
                .get()
                .and_then(|this| this.parent_element())
                .map(|parent| parent.client_height());

            // log::info!("height {:?}", h);
            let h: i32 = match h {
                Some(h) => h,
                None => return (300, 360),
            };
            let n: i32 = (h as f32 * (5. / 6.)).floor() as i32;
            // let i = 0;
            let i = 300;
            // can make this return some and use ? instead of unwrap above
            let r = i32::min(i32::max(n, i), 350);
            let l = 6 * (r as f32 / 5.).floor() as i32;
            log::info!("resized: r: {r} l: {l} n: {n} i: {i} h: {h}");
            (r, l)
        }
    };

    let dimensions = use_state(get_updated_dimensions.clone());

    // let parent_size = use_memo(
    let resize = Callback::from({
        let dimensions = dimensions.clone();
        move |(x, y)| dimensions.set((x, y))
    });

    // let onload = Callback::from({
    //     let resize = resize.clone();
    //     move |e| resize.emit(e)
    // });
    // let _dimensions = dimensions.clone();

    let _div_ref = div_ref.clone();

    use_effect(|| {
        let handler = gloo::events::EventListener::new(&gloo::utils::window(), "resize", {
            move |event| {
                // r = (n = Math.floor(e.clientHeight * (5 / 6)), i = t, Math.min(Math.max(n, i), 350));
                //             var n,
                //             i;
                // const l = 6 * Math.floor(r / 5);

                let h = gloo::utils::document()
                    .get_element_by_id("board-outer")
                    .map(|parent| parent.client_height());
                // let h: Option<i32> = div_ref.get()
                //     .and_then(|this| this.parent_element())
                //     .map(|parent| parent.client_height());

                // log::info!("height {:?}", h);
                let (x, y): (i32, i32) = match h {
                    Some(h) => {
                        let n: i32 = (h as f32 * (5. / 6.)).floor() as i32;
                        // let i = 0;
                        let i = 300;
                        // can make this return some and use ? instead of unwrap above
                        let r = i32::min(i32::max(n, i), 350);
                        let l = 6 * (r as f32 / 5.).floor() as i32;
                        log::info!("resized: r: {r} l: {l} n: {n} i: {i} h: {h}");
                        (r, l)
                    }
                    None => (300, 360),
                };
                resize.emit((x, y));
                // resize.emit(event.clone());
            }
        });
        || {
            drop(handler);
        }
    });

    let dimensions = dimensions.clone();
    let (r, l) = (dimensions.0, dimensions.1);
    let style = format!("width: {r}px; height: {l}px;");
    log::info!("{}", style);
    html!(
            <div class="board-inner" ref={_div_ref} {style}>
                {rows.collect::<Html>()}
            </div>
    )
}

#[function_component]
pub fn Board() -> Html {
    html!(
        <div class="board-outer" id="board-container">
            <InnerBoard />
        </div>
    )
}

type BoardState = [[TileState; 5]; 6];

// TODO: Move gamestate to it's own module
#[derive(Default, Copy, Clone, PartialEq)]
struct GameState {
    board_state: BoardState,
    cur_pos: (usize, usize),
    answer: &'static str,
}

impl GameState {
    fn update_cur_pos(&mut self, c: Option<Chr>) {
        self.board_state[self.cur_pos.1][self.cur_pos.0].current = c;
    }
    fn current_row(&self) -> &[TileState; 5] {
        &self.board_state[self.cur_pos.1]
    }
    fn current_row_mut(&mut self) -> &mut [TileState; 5] {
        &mut self.board_state[self.cur_pos.1]
    }
    fn evaluate_guess(&self) -> [Correctness; 5] {
        let guess_chars = self
            .current_row()
            .map(|state| state.current.expect("Guess has Chr").to_char());
        let guess = String::from_iter(guess_chars);
        let correctness = Correctness::compute(self.answer, &guess);
        log::info!("Computed Correctness: {:?}", correctness);
        correctness
    }
    fn update_tile_states_from_guess(&mut self, guess: [Correctness; 5]) {
        for (correctness, mut tile_state) in zip(guess, self.current_row_mut()) {
            tile_state.goodness = Some(correctness);
        }
    }

    fn puts(mut self, c: Chr) -> Self {
        if self.cur_pos.1 == 6 {
            return self;
        }
        match c {
            Chr::ENTER => {
                if self.cur_pos.0 == 5 {
                    self.update_tile_states_from_guess(self.evaluate_guess());
                    self.cur_pos.0 = 0;
                    self.cur_pos.1 += 1;
                }
            }
            Chr::DEL => {
                if self.cur_pos.0 != 0 {
                    self.cur_pos.0 -= 1;
                    self.update_cur_pos(None);
                }
            }

            _ => {
                if self.cur_pos.0 != 5 {
                    // let prev = self.cur_pos;
                    self.update_cur_pos(Some(c));
                    self.cur_pos.0 += 1;
                    // log::info!("puts: {:?} -> {:?}", prev, self.cur_pos);
                }
            }
        }
        return self;
    }

    fn new(answer: &'static str) -> Self {
        Self {
            answer,
            ..Default::default()
        }
    }
}

impl Reducible for GameState {
    type Action = Chr;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        // NOTE: This __copies__ out of RC. Will need to clone, do unsafe,
        // or rethink strategy if GameState can no longer be Copy
        self.puts(action).into()
    }
}

type GameContext = UseReducerHandle<GameState>;

#[function_component]
pub fn Header() -> Html {
    html! {
        <div class="appHeader"><div class="appHeader-title">{"Jordle"}</div></div>
    }
}

#[function_component]
pub fn GameInterface() -> Html {
    // NOTE: use_memo takes dependencies:
    // set dependency on game number to calculate
    // a new a answer
    let answer = use_memo(
        |_step| {
            let answer = crate::driver::generate_answer();
            log::info!("Created Answer: {}", answer);
            answer
        },
        (),
    );

    let board_state = use_reducer_eq({ move || GameState::new(&answer) });

    let onclick = Callback::from({
        let board_state = board_state.clone();
        move |code| {
            board_state.dispatch(code);
        }
    });

    {
        let board_state = board_state.clone();
        use_effect(move | | {
            let handler = gloo::events::EventListener::new(&gloo::utils::document(), "keydown", {
                move |event| {
                    let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
                    let key: String = event.key();
                    let chr = Chr::from_str(&key);

                    if let Some(chr) = chr {
                        board_state.dispatch(chr)
                    }
                }
            });

            | | drop(handler)
        });
    }

    let style = "height: 90%;";
    html! {
        <div class="game-outer-container" >
            <Header />
            <div class="game-container" {style}>
                <div class="game">
                    <ContextProvider<GameContext> context={board_state}>
                        <Board />
                        <crate::key::Keyboard {onclick}/>
                    </ContextProvider<GameContext>>
                </div>
            </div>
        </div>
    }
}
