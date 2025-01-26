// src/games/cs.rs
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
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

static TEMPORARY_INIT: LazyLock<HashMap<u8, Option<u8>>> = LazyLock::new(|| {
    HashMap::from([
        (2, None),
        (3, None),
        (4, None),
        (5, None),
        (6, None),
        (7, None),
        (8, None),
        (9, None),
        (10, None),
        (11, None),
        (12, None),
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
        match self {
            CSAction::DiceRoll(d1, d2, d3, d4) => {
                let mut new_state = state.clone();
                new_state.last_roll = Some((*d1, *d2, *d3, *d4));
                if new_state.permitted_actions().len() == 1 {
                    // Bust
                    new_state.next_player = (state.next_player + 1) % state.positions.len() as u8;
                    new_state.locked_in_columns.clear();
                    new_state.temp_position = TEMPORARY_INIT.clone();
                    new_state.next_actor = Actor::GameAction(DICE_ACTIONS.clone());
                } else {
                    new_state.next_actor = Actor::Player(new_state.next_player)
                }
                new_state
            }
            CSAction::Move(column, maybe_column) => {
                let mut new_state = state.clone();
                new_state.locked_in_columns.insert(*column);
                new_state.temp_position.insert(
                    *column,
                    Some(
                        new_state.temp_position.get(column).unwrap().unwrap_or(
                            *(state
                                .positions
                                .get(&(state.next_player))
                                .unwrap()
                                .get(column)
                                .unwrap_or(&0)),
                        ) + 1,
                    ),
                );
                if let Some(other_column) = maybe_column {
                    new_state.locked_in_columns.insert(*other_column);
                    new_state.temp_position.insert(
                        *other_column,
                        Some(
                            new_state
                                .temp_position
                                .get(other_column)
                                .unwrap()
                                .unwrap_or(
                                    *(state
                                        .positions
                                        .get(&(state.next_player))
                                        .unwrap()
                                        .get(other_column)
                                        .unwrap_or(&0)),
                                )
                                + 1,
                        ),
                    );
                };
                new_state.next_actor = Actor::GameAction(DICE_ACTIONS.clone());
                new_state
            }
            CSAction::Done => {
                let mut new_state = state.clone();
                for (column, temp_position) in new_state.temp_position.iter() {
                    if let Some(position) = temp_position {
                        *new_state
                            .positions
                            .get_mut(&(new_state.next_player))
                            .unwrap()
                            .get_mut(column)
                            .unwrap() = *position;
                        if position >= COLUMNS.get(column).unwrap() {
                            new_state
                                .claimed_columns
                                .insert(*column, Some(state.next_player));
                        };
                    }
                }
                new_state.next_player = (state.next_player + 1) % state.positions.len() as u8;
                new_state.locked_in_columns.clear();
                new_state.temp_position = TEMPORARY_INIT.clone();
                new_state.next_actor = Actor::GameAction(DICE_ACTIONS.clone());
                new_state
            }
        }
    }
}

type PlayerID = u8;
type ColumnID = u8;

#[derive(Clone, Debug)]
pub struct CSState {
    next_actor: Actor<CSAction>,
    // 2 sources of truth here :s - temp_position Nones could be used too.
    locked_in_columns: HashSet<u8>,
    last_roll: Option<(u8, u8, u8, u8)>,
    next_player: u8,
    positions: HashMap<PlayerID, HashMap<ColumnID, u8>>, // Maybe this should be 1 hashmap with a tuple key?
    temp_position: HashMap<ColumnID, Option<u8>>,
    claimed_columns: HashMap<ColumnID, Option<PlayerID>>,
}

impl CSState {
    fn player_claimed_count(&self) -> HashMap<u8, i32> {
        self.claimed_columns.values().filter_map(|&v| v).fold(
            HashMap::new(),
            |mut acc, player_id| {
                *acc.entry(player_id).or_insert(0) += 1;
                acc
            },
        )
    }
}

impl State for CSState {
    type ActionType = CSAction;

    fn next_actor(&self) -> Actor<CSAction> {
        self.next_actor.clone()
    }

    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        let new_column_allowed = self.locked_in_columns.len() < 3;
        let two_new_columns_allowed = self.locked_in_columns.len() < 2;
        let column_allowed = HashMap::from(
            (2..=12)
                .map(|col| {
                    (
                        col,
                        // It's my vanity project, and even I think this might be a little much
                        (new_column_allowed || self.locked_in_columns.contains(&col))
                            && self.claimed_columns.get(&col) == None
                            && (self.temp_position[&col].is_none()
                                || self.temp_position[&col] < COLUMNS.get(&col).copied()),
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
        let mut one_match_actions: Vec<CSAction> = vec![];
        // 1&2/3&4
        let d12 = d1 + d2;
        let d34 = d3 + d4;
        if column_allowed[&d12]
            && column_allowed[&d34]
            && (two_new_columns_allowed
                || self.locked_in_columns.contains(&d12)
                || self.locked_in_columns.contains(&d34))
        {
            possible_actions.push(CSAction::Move(d12, Some(d34)));
        } else if column_allowed[&d12] {
            one_match_actions.push(CSAction::Move(d12, None));
        } else if column_allowed[&d34] {
            one_match_actions.push(CSAction::Move(d34, None));
        };

        // 1&3/2&4
        let d13 = d1 + d3;
        let d24 = d2 + d4;
        if column_allowed[&d13]
            && column_allowed[&d24]
            && (two_new_columns_allowed
                || self.locked_in_columns.contains(&d13)
                || self.locked_in_columns.contains(&d24))
        {
            possible_actions.push(CSAction::Move(d13, Some(d24)));
        } else if column_allowed[&d13] {
            one_match_actions.push(CSAction::Move(d13, None));
        } else if column_allowed[&d24] {
            one_match_actions.push(CSAction::Move(d24, None));
        }

        // 1&4/2&3
        let d14 = d1 + d4;
        let d23 = d2 + d3;
        if column_allowed[&d14]
            && column_allowed[&d23]
            && column_allowed[&d24]
            && (two_new_columns_allowed
                || self.locked_in_columns.contains(&d14)
                || self.locked_in_columns.contains(&d23))
        {
            possible_actions.push(CSAction::Move(d14, Some(d23)));
        } else if column_allowed[&d14] {
            one_match_actions.push(CSAction::Move(d14, None));
        } else if column_allowed[&d23] {
            one_match_actions.push(CSAction::Move(d23, None));
        }

        if possible_actions.len() == 1 {
            // Only do the 'single actions' if there's no double actions
            possible_actions.extend(one_match_actions.iter());
        }

        possible_actions
    }

    fn reward(&self) -> Vec<f64> {
        if !self.terminal() {
            vec![0.0f64; self.positions.len()]
        } else {
            let counts = self.player_claimed_count();
            let max_count = *counts.values().max().unwrap();
            (0..self.positions.len() as u8)
                .map(|player_id| {
                    if counts.get(&player_id) == Some(&max_count) {
                        1.0
                    } else {
                        0.0
                    }
                })
                .collect()
        }
    }

    fn terminal(&self) -> bool {
        self.player_claimed_count()
            .values()
            .any(|&count| count >= 3)
    }
}

pub struct CS {
    pub player_count: u8,
}

impl Game for CS {
    type StateType = CSState;
    type ActionType = CSAction;

    fn init_game(&self) -> Self::StateType {
        let positions: HashMap<u8, HashMap<u8, u8>> = (0..self.player_count)
            .map(|player_id| {
                let inner_map = HashMap::from([
                    (2, 0),
                    (3, 0),
                    (4, 0),
                    (5, 0),
                    (6, 0),
                    (7, 0),
                    (8, 0),
                    (9, 0),
                    (10, 0),
                    (11, 0),
                    (12, 0),
                ]);
                (player_id, inner_map)
            })
            .collect();
        CSState {
            positions,
            claimed_columns: HashMap::new(),
            locked_in_columns: HashSet::new(),
            temp_position: TEMPORARY_INIT.clone(),
            last_roll: None,
            next_actor: Actor::GameAction(DICE_ACTIONS.clone()),
            next_player: 0,
        }
    }

    fn visualise_state(&self, state: &Self::StateType) {
        println!("Roll: {:?}", state.last_roll);
        println!("Claimed: {:?}", state.claimed_columns);
        println!("Positions:");
        for i in 0..self.player_count {
            let mut sorted_positions = state.positions.get(&i).unwrap().iter().collect::<Vec<_>>();
            sorted_positions.sort_by_key(|(k, _)| *k);
            println!("Player {}: {:?}", i, sorted_positions);
        }
        let mut sorted_temp_positions = state.temp_position.iter().collect::<Vec<_>>();
        sorted_temp_positions.sort_by_key(|(k, _)| *k);
        println!("Temporary: {:?}", sorted_temp_positions);
        println!("Player: {:?}", state.next_player);
        println!("Locked in: {:?}", state.locked_in_columns);
    }
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
