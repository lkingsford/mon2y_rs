use std::collections::HashMap;

use crate::game::Game;
use crate::mon2y::game::{Action, Actor, State};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum NTAction {
    Take,
    NoThanks,
    Draw(u8),
}

impl Action for NTAction {
    type StateType = NTState;
    fn execute(&self, state: &Self::StateType) -> Self::StateType {
        match self {
            NTAction::Take => {
                todo!()
            }
            NTAction::NoThanks => {
                todo!()
            }
            NTAction::Draw(u8) => {
                todo!()
            }
        }
    }
}

#[derive(Clone)]
enum CardState<Actor> {
    Drawable,
    Taken(Actor),
}

#[derive(Clone)]
pub struct NTState {
    cards: HashMap<u8, CardState<Actor<NTAction>>>,
    tokens: HashMap<u8, u8>,
    next_player: u8,
    to_draw: bool,
    current_card: u8,
    tokens_on_card: u8,
}

impl State for NTState {
    type ActionType = NTAction;

    fn next_actor(&self) -> Actor<NTAction> {
        match self.to_draw {
            true => Actor::Player(self.next_player),
            false => Actor::GameAction(self.possible_non_player_actions()),
        }
    }

    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        todo!()
    }

    fn possible_non_player_actions(&self) -> Vec<(Self::ActionType, f64)> {
        self.cards
            .iter()
            .filter(|(_, card_state)| matches!(card_state, CardState::Drawable))
            .map(|(card, _)| (NTAction::Draw(*card), 1.0))
            .collect()
    }

    fn terminal(&self) -> bool {
        self.cards
            .iter()
            .filter(|(_, card_state)| matches!(card_state, CardState::Taken(_)))
            .count()
            == 24
    }

    fn reward(&self) -> Vec<f64> {
        todo!()
    }
}

pub struct NT {
    player_count: u8,
}

impl Game for NT {
    type StateType = NTState;
    type ActionType = NTAction;

    fn visualise_state(&self, state: &Self::StateType) {
        todo!()
    }

    fn get_human_turn(&self, state: &Self::StateType) -> Self::ActionType {
        todo!()
    }

    fn init_game(&self) -> Self::StateType {
        NTState {
            cards: (3..35)
                .map(|card| (card, CardState::Drawable))
                .collect::<HashMap<_, _>>(),
            tokens: (0..self.player_count - 1)
                .map(|player_id| {
                    (
                        player_id,
                        match self.player_count {
                            0..=5 => 11,
                            6 => 9,
                            _ => 7,
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
            current_card: rand::Rng::gen_range(&mut rand::thread_rng(), 3..35),
            next_player: 0,
            to_draw: false,
            tokens_on_card: 0,
        }
    }
}
