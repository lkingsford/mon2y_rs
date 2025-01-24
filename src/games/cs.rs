// src/games/cs.rs
use std::collections::HashMap;

use crate::game::Game;
use crate::mon2y::game::{Action, Actor, State};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum CSAction {}

impl Action for CSAction {
    type StateType = CSState;
    fn execute(&self, state: &Self::StateType) -> Self::StateType {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct CSState {
    player_turn: u8,
    player_cards: Vec<Vec<u8>>,
    deck: Vec<u8>,
}

impl State for CSState {
    type ActionType = CSAction;

    fn next_actor(&self) -> Actor<CSAction> {
        Actor::Player(self.player_turn)
    }

    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        vec![]
    }

    fn reward(&self) -> Vec<f64> {
        todo!()
    }

    fn terminal(&self) -> bool {
        self.deck.is_empty()
    }
}

pub struct CS {
    pub player_count: u8,
}

impl Game for CS {
    type StateType = CSState;
    type ActionType = CSAction;

    fn init_game(&self) -> Self::StateType {
        todo!()
    }

    fn visualise_state(&self, _state: &Self::StateType) {}
}
