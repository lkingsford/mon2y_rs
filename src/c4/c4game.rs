use crate::mon2y::action::Action;
use crate::mon2y::node::ActResponse;
use crate::mon2y::state::State;
use std::any::Any;
use std::io;
const BOARD_WIDTH: usize = 7;
const BOARD_HEIGHT: usize = 6;

// I know it's a fixed size, but this makes other things way easier
type Board = Vec<Cell>;

#[derive(Copy, Clone, PartialEq, serde::Serialize)]
enum Cell {
    Empty,
    Filled(u8),
}

#[derive(Clone)]
pub struct C4State {
    pub board: Board,
    pub next_player: u8,
}

impl State for C4State {
    fn loggable(&self) -> serde_json::Value {
        serde_json::json!({"board": self.board})
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn init_game() -> ActResponse {
    let state = C4State {
        board: vec![Cell::Empty; BOARD_HEIGHT * BOARD_WIDTH],
        next_player: 0,
    };
    ActResponse {
        permitted_actions: permitted_actions(&state),
        state: Box::new(state),
        next_player: Some(0),
        reward: None,
        terminated: false,
        next_act_fn: Box::new(act),
        memo: None,
    }
}

#[derive(PartialEq)]
enum Result {
    Winner(u8),
    Stalemate,
    Ongoing,
}

fn check_for_win(board: &Board) -> Result {
    // Check Horizontal win
    for row in 0..BOARD_HEIGHT {
        for column in 0..BOARD_WIDTH - 3 {
            if board[row * BOARD_WIDTH + column] == board[row * BOARD_WIDTH + column + 1]
                && board[row * BOARD_WIDTH + column] == board[row * BOARD_WIDTH + column + 2]
                && board[row * BOARD_WIDTH + column] == board[row * BOARD_WIDTH + column + 3]
                && board[row * BOARD_WIDTH + column] != Cell::Empty
            {
                return Result::Winner(match board[row * BOARD_WIDTH + column] {
                    Cell::Filled(player) => player,
                    _ => unreachable!(),
                });
            }
        }
    }

    // Check Vertical win
    for column in 0..BOARD_WIDTH {
        for row in 0..BOARD_HEIGHT - 3 {
            if board[row * BOARD_WIDTH + column] == board[(row + 1) * BOARD_WIDTH + column]
                && board[row * BOARD_WIDTH + column] == board[(row + 2) * BOARD_WIDTH + column]
                && board[row * BOARD_WIDTH + column] == board[(row + 3) * BOARD_WIDTH + column]
                && board[row * BOARD_WIDTH + column] != Cell::Empty
            {
                return Result::Winner(match board[row * BOARD_WIDTH + column] {
                    Cell::Filled(player) => player,
                    _ => unreachable!(),
                });
            }
        }
    }

    // Check \ win
    for column in 0..BOARD_WIDTH - 3 {
        for row in 0..BOARD_HEIGHT - 3 {
            if board[row * BOARD_WIDTH + column] == board[(row + 1) * BOARD_WIDTH + column + 1]
                && board[row * BOARD_WIDTH + column] == board[(row + 2) * BOARD_WIDTH + column + 2]
                && board[row * BOARD_WIDTH + column] == board[(row + 3) * BOARD_WIDTH + column + 3]
                && board[row * BOARD_WIDTH + column] != Cell::Empty
            {
                return Result::Winner(match board[row * BOARD_WIDTH + column] {
                    Cell::Filled(player) => player,
                    _ => unreachable!(),
                });
            }
        }
    }

    // Check / win
    for column in 0..BOARD_WIDTH - 3 {
        for row in 3..BOARD_HEIGHT {
            if board[row * BOARD_WIDTH + column] == board[(row - 1) * BOARD_WIDTH + column + 1]
                && board[row * BOARD_WIDTH + column] == board[(row - 2) * BOARD_WIDTH + column + 2]
                && board[row * BOARD_WIDTH + column] == board[(row - 3) * BOARD_WIDTH + column + 3]
                && board[row * BOARD_WIDTH + column] != Cell::Empty
            {
                return Result::Winner(match board[row * BOARD_WIDTH + column] {
                    Cell::Filled(player) => player,
                    _ => unreachable!(),
                });
            }
        }
    }

    Result::Ongoing
}

fn permitted_actions(state: &C4State) -> Vec<Action> {
    (0..BOARD_WIDTH)
        .filter(|&i| state.board[i] == Cell::Empty)
        .map(|i| Action::Num(i as i32))
        .collect::<Vec<Action>>()
}

fn act(generic_state: &dyn State, action: Action) -> ActResponse {
    let state = generic_state
        .as_any()
        .downcast_ref::<C4State>()
        .expect("Expected C4State");

    let mut new_state = state.clone();

    let column = match action {
        Action::Num(num) => num,
        _ => unreachable!(),
    };
    for row in (0..BOARD_HEIGHT).rev() {
        if new_state.board[row * BOARD_WIDTH + column as usize] == Cell::Empty {
            new_state.board[row * BOARD_WIDTH + column as usize] =
                Cell::Filled(new_state.next_player);
            break;
        }
    }

    let winner = check_for_win(&new_state.board);

    let reward: Option<Vec<f64>> = match winner {
        Result::Winner(0) => Some(vec![1.0, -1.0]),
        Result::Winner(1) => Some(vec![-1.0, 1.0]),
        Result::Stalemate => Some(vec![-0.5, -0.5]), // Discourage stalemate
        Result::Ongoing => None,
        _ => None,
    };

    new_state.next_player = (new_state.next_player + 1) % 2;
    let next_player = new_state.next_player;

    ActResponse {
        permitted_actions: permitted_actions(&new_state),
        state: Box::new(new_state),
        next_player: Some(next_player),
        reward,
        terminated: winner != Result::Ongoing,
        next_act_fn: Box::new(act),
        memo: None,
    }
}

pub fn get_human_turn(state: &dyn State) -> Action {
    let state = state
        .as_any()
        .downcast_ref::<C4State>()
        .expect("Expected C4State");
    for x in 0..BOARD_WIDTH {
        print!("{}", x);
    }
    print!("\n");
    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            print!(
                "{}",
                match (state.board[y * BOARD_WIDTH + x]) {
                    Cell::Empty => "◌",
                    Cell::Filled(1) => "◍",
                    Cell::Filled(0) => "●",
                    _ => " ",
                }
            )
        }
        print!("\n");
    }
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let action = input.trim().parse().expect("Failed to parse action");
    Action::Num(action)
}
