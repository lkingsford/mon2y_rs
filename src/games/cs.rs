// src/games/cs.rs
use std::collections::{HashMap, HashSet};
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
    for d1 in 1..=6 {
        for d2 in 1..=6 {
            for d3 in 1..=6 {
                for d4 in 1..=6 {
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
    locked_in_columns: HashSet<u8>,
    last_roll: Option<(u8, u8, u8, u8)>,
}

impl State for CSState {
    type ActionType = CSAction;

    fn next_actor(&self) -> Actor<CSAction> {
        Actor::Player(self.player_turn)
    }

    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        let new_column_allowed = self.locked_in_columns.len() < 3;
        let column_allowed = HashMap::from(
            (2..=12)
                .map(|col| {
                    (
                        col,
                        new_column_allowed || self.locked_in_columns.contains(&col),
                    )
                })
                .collect::<HashMap<_, _>>(),
        );

        // This could be done more programmatically (with less repetition), but the action space is small enough
        // that I'm not worried
        let (d1, d2, d3, d4) = match self.last_roll {
            Some((d1, d2, d3, d4)) => (d1, d2, d3, d4),
            None => panic!("Dice haven't been rolled"),
        };
        let mut possible_actions: Vec<CSAction> = vec![CSAction::Done];
        // 1&2/3&4
        let d12 = d1 + d2;
        let d34 = d3 + d4;
        if column_allowed[&d12] && column_allowed[&d34] {
            possible_actions.push(CSAction::Move(d12, Some(d34)));
        } else if column_allowed[&d12] {
            possible_actions.push(CSAction::Move(d12, None));
        } else if column_allowed[&d34] {
            possible_actions.push(CSAction::Move(d34, None));
        };

        // 1&3/2&4
        let d13 = d1 + d3;
        let d24 = d2 + d4;
        if column_allowed[&d13] && column_allowed[&d24] {
            possible_actions.push(CSAction::Move(d13, Some(d24)));
        } else if column_allowed[&d13] {
            possible_actions.push(CSAction::Move(d13, None));
        } else if column_allowed[&d24] {
            possible_actions.push(CSAction::Move(d24, None));
        }

        // 1&4/2&3
        let d14 = d1 + d4;
        let d23 = d2 + d3;
        if column_allowed[&d14] && column_allowed[&d23] {
            possible_actions.push(CSAction::Move(d14, Some(d23)));
        } else if column_allowed[&d14] {
            possible_actions.push(CSAction::Move(d14, None));
        } else if column_allowed[&d23] {
            possible_actions.push(CSAction::Move(d23, None));
        }

        possible_actions
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
mod tests {
    use super::*;

    #[test]
    fn test_dice_actions_weights() {
        let test_cases = vec![
            (CSAction::DiceRoll(1, 1, 1, 1), 1),
            (CSAction::DiceRoll(1, 4, 4, 4), 4),
            (CSAction::DiceRoll(1, 3, 3, 5), 12),
            (CSAction::DiceRoll(1, 3, 4, 6), 24),
        ];

        let actions: HashMap<CSAction, u32> = HashMap::from(
            DICE_ACTIONS
                .iter()
                .map(|(action, weight)| (*action, *weight))
                .collect::<HashMap<_, _>>(),
        );

        for (action, expected_weight) in test_cases {
            let actual_weight = actions.get((&action)).unwrap_or(&0);
            assert_eq!(
                *actual_weight, expected_weight,
                "Action {:?} has weight {}, expected {}",
                action, actual_weight, expected_weight
            );
        }
    }
}
