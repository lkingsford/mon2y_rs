use linked_hash_set::LinkedHashSet;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::LazyLock;

use crate::game::Game;
use crate::mon2y::game::{Action, Actor, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Terrain {
    Nothing,
    Plain,
    Forest,
    Mountain,
    Town,
    Port,
}

#[derive(Debug, Clone)]
struct Feature {
    feature_type: FeatureType,
    location_name: Option<String>,
    revenue: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FeatureType {
    Port,
    Town,
    Water1,
    Water2,
}

const FINAL_DIVIDEND_COUNT: usize = 6;

const TERRAIN: [[Terrain; 14]; 13] = [
    [Terrain::Nothing; 14],
    [
        Terrain::Nothing,
        Terrain::Plain,
        Terrain::Forest,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Plain,
        Terrain::Nothing,
    ],
    [
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Plain,
        Terrain::Port,
        Terrain::Town,
        Terrain::Nothing,
        Terrain::Plain,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Mountain,
    ],
    [
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Port,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Plain,
        Terrain::Forest,
        Terrain::Port,
        Terrain::Plain,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Port,
        Terrain::Town,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Forest,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Forest,
        Terrain::Forest,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Forest,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Forest,
        Terrain::Forest,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Forest,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Nothing,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Mountain,
        Terrain::Forest,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Nothing,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Town,
        Terrain::Port,
        Terrain::Plain,
        Terrain::Plain,
        Terrain::Nothing,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Mountain,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Forest,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
    ],
    [
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Forest,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
        Terrain::Nothing,
    ],
];

const TERRAIN_COSTS: [(Terrain, u32); 5] = [
    (Terrain::Plain, 3),
    (Terrain::Forest, 4),
    (Terrain::Mountain, 6),
    (Terrain::Town, 4),
    (Terrain::Port, 5),
];

const FEATURE_COSTS: [(FeatureType, u32); 2] = [(FeatureType::Water1, 1), (FeatureType::Water2, 3)];

lazy_static::lazy_static! {
    static ref FEATURES: HashMap<(usize, usize), Feature> = {
        let mut m = HashMap::new();
        m.insert((2, 5), Feature { feature_type: FeatureType::Port, location_name: Some("Port of Strahan".to_string()), revenue: Some(vec![2, 2, 0, 0, 0, 0]) });
        m.insert((10, 9), Feature { feature_type: FeatureType::Port, location_name: Some("Hobart".to_string()), revenue: Some(vec![5, 5, 4, 4, 3, 3]) });
        m.insert((9, 9), Feature { feature_type: FeatureType::Town, location_name: Some("New Norfolk".to_string()), revenue: Some(vec![2, 2, 2, 2, 2, 2]) });
        m.insert((8, 2), Feature { feature_type: FeatureType::Water1, location_name: None, revenue: None });
        m.insert((8, 3), Feature { feature_type: FeatureType::Water1, location_name: None, revenue: None });
        m.insert((8, 5), Feature { feature_type: FeatureType::Water2, location_name: None, revenue: None });
        m
    };
}

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

static TEMPORARY_INIT: LazyLock<[Option<u8>; COLUMN_COUNT]> = LazyLock::new(|| {
    [
        None, None, None, None, None, None, None, None, None, None, None,
    ]
});

const COLUMN_COUNT: usize = 11;

/// List of all dice actions from 4 d6s with weights
/// (so - 1,1,1,1 is weighted 1 - because there's only 1 way to get that combo )
static DICE_ACTIONS: LazyLock<Vec<(EBRAction, u32)>> = LazyLock::new(|| {
    let mut actions_and_weights: HashMap<EBRAction, u32> = HashMap::new();
    for d1 in 1..=6 {
        for d2 in 1..=6 {
            for d3 in 1..=6 {
                for d4 in 1..=6 {
                    let mut sorted = [d1, d2, d3, d4];
                    sorted.sort_unstable();
                    let action = EBRAction::DiceRoll(sorted[0], sorted[1], sorted[2], sorted[3]);
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
pub enum EBRAction {
    DiceRoll(u8, u8, u8, u8),
    Move(u8, Option<u8>),
    Roll,
    Done,
}

impl Action for EBRAction {
    type StateType = EBRState;
    fn execute(&self, state: &Self::StateType) -> Self::StateType {
        match self {
            EBRAction::DiceRoll(d1, d2, d3, d4) => {
                let mut new_state = state.clone();
                new_state.last_roll = Some((*d1, *d2, *d3, *d4));
                if new_state.permitted_actions().len() == 0 {
                    // Bust
                    new_state.next_player = (state.next_player + 1) % new_state.player_count;
                    new_state.locked_in_columns.clear();
                    new_state.temp_position = TEMPORARY_INIT.clone();
                    new_state.next_actor = Actor::GameAction(DICE_ACTIONS.to_vec());
                } else {
                    new_state.next_actor = Actor::Player(new_state.next_player);
                }
                new_state
            }
            EBRAction::Move(column, maybe_column) => {
                let mut new_state = state.clone();
                new_state.locked_in_columns.insert(*column);
                new_state.temp_position[*column as usize - 2] = Some(
                    new_state.temp_position[*column as usize - 2].unwrap_or(
                        state.positions[state.next_player as usize][*column as usize - 2],
                    ) + 1,
                );
                if let Some(other_column) = maybe_column {
                    new_state.locked_in_columns.insert(*other_column);
                    new_state.temp_position[*other_column as usize - 2] = Some(
                        new_state.temp_position[*other_column as usize - 2].unwrap_or(
                            state.positions[state.next_player as usize][*other_column as usize - 2],
                        ) + 1,
                    )
                };
                new_state.last_roll = None;
                new_state
            }
            EBRAction::Roll => {
                let mut new_state = state.clone();
                new_state.next_actor = Actor::GameAction(DICE_ACTIONS.clone());
                new_state
            }
            EBRAction::Done => {
                let mut new_state = state.clone();

                for (index, temp_position) in state.temp_position.iter().enumerate() {
                    let column = (index + 2) as u8;
                    if let Some(position) = temp_position {
                        new_state.positions[new_state.next_player as usize][column as usize - 2] =
                            *position;
                        if position >= COLUMNS.get(&column).unwrap() {
                            new_state
                                .claimed_columns
                                .insert(column, Some(state.next_player));
                        };
                    }
                }
                new_state.next_player = (state.next_player + 1) % new_state.player_count;
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
pub struct EBRState {
    next_actor: Actor<EBRAction>,
    // 2 sources of truth here :s - temp_position Nones could be used too.
    locked_in_columns: HashSet<u8>,
    last_roll: Option<(u8, u8, u8, u8)>,
    next_player: u8,
    positions: Vec<[u8; COLUMN_COUNT]>, // Maybe this should be 1 hashmap with a tuple key?
    temp_position: [Option<u8>; COLUMN_COUNT],
    claimed_columns: HashMap<ColumnID, Option<PlayerID>>,
    player_count: u8,
}

impl EBRState {
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

impl State for EBRState {
    type ActionType = EBRAction;

    fn next_actor(&self) -> Actor<EBRAction> {
        self.next_actor.clone()
    }

    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        if self.last_roll.is_none() {
            return vec![EBRAction::Roll, EBRAction::Done];
        }

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
                            && (self.temp_position[col as usize - 2].is_none()
                                || self.temp_position[col as usize - 2]
                                    < COLUMNS.get(&col).copied()),
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
        let mut possible_actions: Vec<EBRAction> = vec![];
        let mut one_match_actions: Vec<EBRAction> = vec![];
        // 1&2/3&4
        let d12 = d1 + d2;
        let d34 = d3 + d4;
        if column_allowed[&d12]
            && column_allowed[&d34]
            && (two_new_columns_allowed
                || self.locked_in_columns.contains(&d12)
                || self.locked_in_columns.contains(&d34))
        {
            possible_actions.push(EBRAction::Move(min(d12, d34), Some(max(d12, d34))));
        } else if column_allowed[&d12] {
            one_match_actions.push(EBRAction::Move(d12, None));
        } else if column_allowed[&d34] {
            one_match_actions.push(EBRAction::Move(d34, None));
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
            possible_actions.push(EBRAction::Move(min(d13, d24), Some(max(d13, d24))));
        } else if column_allowed[&d13] {
            one_match_actions.push(EBRAction::Move(d13, None));
        } else if column_allowed[&d24] {
            one_match_actions.push(EBRAction::Move(d24, None));
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
            possible_actions.push(EBRAction::Move(min(d14, d23), Some(max(d14, d23))));
        } else if column_allowed[&d14] {
            one_match_actions.push(EBRAction::Move(d14, None));
        } else if column_allowed[&d23] {
            one_match_actions.push(EBRAction::Move(d23, None));
        }

        if possible_actions.len() == 0 {
            // Only do the 'single actions' if there's no double actions
            possible_actions.extend(one_match_actions.iter());
        }

        // Remove duplicate actions (which is why they're sorted when there's 2)
        let unique_actions: LinkedHashSet<EBRAction> =
            LinkedHashSet::from_iter(possible_actions.iter().cloned());

        unique_actions.iter().map(|a| *a).collect()
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

pub struct EBR {
    pub player_count: u8,
}

impl Game for EBR {
    type StateType = EBRState;
    type ActionType = EBRAction;

    fn init_game(&self) -> Self::StateType {
        let positions: Vec<[u8; COLUMN_COUNT]> =
            (0..self.player_count).map(|_| [0; COLUMN_COUNT]).collect();
        EBRState {
            positions,
            claimed_columns: HashMap::new(),
            locked_in_columns: HashSet::new(),
            temp_position: TEMPORARY_INIT.clone(),
            last_roll: None,
            next_actor: Actor::GameAction(DICE_ACTIONS.clone()),
            next_player: 0,
            player_count: self.player_count,
        }
    }

    fn visualise_state(&self, state: &Self::StateType) {
        println!("Roll: {:?}", state.last_roll);
        println!("Claimed: {:?}", state.claimed_columns);
        println!("Positions:");
        for i in 0..self.player_count {
            print!("Player {}: ", i);
            for (index, value) in state.positions[i as usize].iter().enumerate() {
                print!("{}: {:?}, ", index + 2, value);
            }
            println!();
        }
        print!("Temporary: ");
        for (index, value) in state.temp_position.iter().enumerate() {
            if let Some(v) = value {
                print!("{}: {}, ", index + 2, v);
            }
        }
        println!();
        println!("Player: {:?}", state.next_player);
        println!("Locked in: {:?}", state.locked_in_columns);
    }
}
