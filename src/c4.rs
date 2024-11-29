use crate::game::Game;
use crate::mon2y::{Action, Actor, State};

const BOARD_WIDTH: usize = 7;
const BOARD_HEIGHT: usize = 6;

#[derive(Copy, Clone, PartialEq)]
pub enum C4Action {
    Drop(u8),
}

impl Action for C4Action {
    type StateType = C4State;
    fn execute(&self, state: &C4State) -> C4State {
        todo!()
    }
}
#[derive(Copy, Clone, PartialEq)]
enum C4Cell {
    Empty,
    Filled(u8),
}

pub struct C4State {
    board: Vec<C4Cell>,
    next_player: u8,
}

impl State for C4State {
    type ActionType = C4Action;
    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        todo!()
    }
    fn next_actor(&self) -> Actor<C4Action> {
        Actor::Player(self.next_player)
    }
    fn terminal(&self) -> bool {
        todo!()
    }
}

pub struct C4;

impl Game for C4 {
    type StateType = C4State;
    type ActionType = C4Action;
    fn get_human_turn(&self, state: &Self::StateType) -> Self::ActionType {
        todo!()
    }
    fn init_game(&self) -> Self::StateType {
        C4State {
            board: vec![C4Cell::Empty; BOARD_HEIGHT * BOARD_WIDTH],
            next_player: 0,
        }
    }
}
