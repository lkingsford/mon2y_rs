// src/games/cs.rs
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::game::Game;
use crate::mon2y::game::{Action, Actor, State};

/// Column lengths in the game
static COLUMNS: LazyLock<HashMap<u8, u8>> = LazyLock::new(|| {
    HashMap::from([
        (2, 3),
        (3, 5),
        (4, 7),
        (5, 9),
        (6, 11),
        (7, 13),
        (8, 11),
        (9, 9),
        (10, 7),
        (11, 5),
        (12, 3),
    ])
});

/// List of all dice actions from 4 d6s with weights
/// (so - 1,1,1,1 is weighted 1 - because there's only 1 way to get that combo )
static DICE_ACTIONS: LazyLock<Vec<(CSAction, u32)>> = LazyLock::new(|| {
    let mut actions_and_weights: HashMap<CSAction, u32> = HashMap::new();
    for d1 in 1..6 {
        for d2 in 1..6 {
            for d3 in 1..6 {
                for d4 in 1..6 {
                    let mut sorted = [d1, d2, d3, d4];
                    sorted.sort_unstable();
                    let action = CSAction::DiceRoll(sorted[0], sorted[1], sorted[2], sorted[3]);
                    let old_weight = actions_and_weights.get(&action).unwrap_or(&0);
                    actions_and_weights.insert(action, old_weight + 1);
                }
            }
        }
    }
    actions_and_weights
        .iter()
        .map(|(action, weight)| (*action, *weight))
        .collect()
});
// Python code to do almost what we're doing here
// all_combos = [str(sorted(l)) for l in itertools.product([1,2,3,4,5,6],[1,2,3,4,5,6],[1,2,3,4,5,6],[1,2,3,4,5,6])]
// set_combos = set(all_combos)
// [(i,len([_ for _ in all_combos if _ == i])) for i in sorted(list(set_combos))]

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum CSAction {
    DiceRoll(u8, u8, u8, u8),
    Move(u8, Option<u8>),
    Done,
}

impl Action for CSAction {
    type StateType = CSState;
    fn execute(&self, state: &Self::StateType) -> Self::StateType {
        todo!();
    }
}

#[derive(Clone, Debug)]
pub struct CSState {
    player_turn: u8,
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

#[cfg(test)]
mod tests {}
