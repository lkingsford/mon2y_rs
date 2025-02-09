use linked_hash_set::LinkedHashSet;
use std::cmp::{max, min};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::sync::LazyLock;

use crate::game::Game;
use crate::mon2y::game::{Action, Actor, State};

/*
OK - here's the deal. This is to help me playtest something.
It's a lot quicker for me to shove the data directly in the
source file, though I know it would be better for it to be in
data files. It's serving its purpose, and it doesn't need to
be built for maintainability.
*/

enum EndGameReason {
    Shares,
    Bonds,
    Track,
    Resources,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum ChoosableAction {
    BuildTrack,
    AuctionShare,
    TakeResources,
    IssueBond,
    Merge,
    PayDividend,
}

const ACTION_CUBE_SPACES: [ChoosableAction; 11] = [
    ChoosableAction::BuildTrack,
    ChoosableAction::BuildTrack,
    ChoosableAction::BuildTrack,
    ChoosableAction::AuctionShare,
    ChoosableAction::AuctionShare,
    ChoosableAction::TakeResources,
    ChoosableAction::TakeResources,
    ChoosableAction::TakeResources,
    ChoosableAction::IssueBond,
    ChoosableAction::Merge,
    ChoosableAction::PayDividend,
];

type ActionCubeSpaces = [bool; 11];

const ACTION_CUBE_INIT: ActionCubeSpaces = [
    // This might not be the most helpful way to mentally consider this
    false, false, false, false, false, true, true, true, false, false, true,
];

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
struct Bond {
    face_value: u32,
    interest: u32,
}
const BONDS: [Bond; 7] = [
    Bond {
        face_value: 5,
        interest: 1,
    },
    Bond {
        face_value: 5,
        interest: 1,
    },
    Bond {
        face_value: 10,
        interest: 3,
    },
    Bond {
        face_value: 10,
        interest: 3,
    },
    Bond {
        face_value: 10,
        interest: 4,
    },
    Bond {
        face_value: 15,
        interest: 4,
    },
    Bond {
        face_value: 15,
        interest: 5,
    },
];

static INITIAL_CASH: LazyLock<HashMap<u8, u32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(2, 20);
    m.insert(3, 13);
    m.insert(4, 10);
    m.insert(5, 8);
    m
});

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
enum Terrain {
    Nothing(CommonAttributes),
    Plain(CommonAttributes),
    Forest(CommonAttributes),
    Mountain(CommonAttributes),
    Town(CommonAttributes),
    Port(CommonAttributes),
}

#[derive(Debug, Clone)]
struct Feature {
    feature_type: FeatureType,
    location_name: Option<String>,
    revenue: Option<Vec<u32>>,
    additional_cost: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FeatureType {
    Port,
    Town,
    Water1,
    Water2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Company {
    EBRC,
    LW,
    TMLC,
    GT,
    NMFT,
    NED,
    MLM,
}

const IPO_ORDER: [Company; 4] = [Company::LW, Company::TMLC, Company::EBRC, Company::GT];

struct CompanyFixedDetails {
    starting: Option<Coordinate>,
    private: bool,
    stock_available: u32,
    track_available: u32,
    initial_treasury: u32,
    initial_interest: u32,
}

type Coordinate = (usize, usize);

static COMPANY_FIXED_DETAILS: LazyLock<HashMap<Company, CompanyFixedDetails>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            Company::EBRC,
            CompanyFixedDetails {
                starting: Some((3, 5)),
                private: false,
                stock_available: 5,
                track_available: 10,
                initial_treasury: 0,
                initial_interest: 0,
            },
        );
        m.insert(
            Company::LW,
            CompanyFixedDetails {
                starting: Some((9, 4)),
                private: false,
                stock_available: 3,
                track_available: 10,
                initial_treasury: 0,
                initial_interest: 0,
            },
        );
        m.insert(
            Company::TMLC,
            CompanyFixedDetails {
                starting: Some((9, 4)),
                private: false,
                stock_available: 4,
                track_available: 10,
                initial_treasury: 0,
                initial_interest: 0,
            },
        );
        m.insert(
            Company::GT,
            CompanyFixedDetails {
                starting: Some((2, 4)),
                private: true,
                stock_available: 1,
                track_available: 0,
                initial_treasury: 10,
                initial_interest: 2,
            },
        );
        m.insert(
            Company::NMFT,
            CompanyFixedDetails {
                starting: None,
                private: true,
                stock_available: 1,
                track_available: 0,
                initial_treasury: 0,
                initial_interest: 0,
            },
        );
        m.insert(
            Company::NED,
            CompanyFixedDetails {
                starting: None,
                private: true,
                stock_available: 1,
                track_available: 0,
                initial_treasury: 15,
                initial_interest: 3,
            },
        );
        m.insert(
            Company::MLM,
            CompanyFixedDetails {
                starting: None,
                private: true,
                stock_available: 1,
                track_available: 0,
                initial_treasury: 20,
                initial_interest: 5,
            },
        );
        m
    });

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
struct CommonAttributes {
    feature_cost: u32,
    symbol: Option<&'static str>,
    buildable: bool,
    multiple_allowed: bool,
}

const FINAL_DIVIDEND_COUNT: usize = 6;

const N: Terrain = Terrain::Nothing(CommonAttributes {
    feature_cost: 0,
    symbol: None,
    buildable: false,
    multiple_allowed: false,
});
const P: Terrain = Terrain::Plain(CommonAttributes {
    feature_cost: 3,
    symbol: Some("\u{1B}[37m-"),
    buildable: true,
    multiple_allowed: true,
});
const F: Terrain = Terrain::Forest(CommonAttributes {
    feature_cost: 4,
    symbol: Some("\u{1B}[32m="),
    buildable: true,
    multiple_allowed: false,
});
const M: Terrain = Terrain::Mountain(CommonAttributes {
    feature_cost: 6,
    symbol: Some("\u{1B}[32m^"),
    multiple_allowed: false,
    buildable: true,
});
const T: Terrain = Terrain::Town(CommonAttributes {
    feature_cost: 4,
    symbol: Some("\u{1B}[33mT"),
    buildable: true,
    multiple_allowed: true,
});
const R: Terrain = Terrain::Port(CommonAttributes {
    feature_cost: 5,
    symbol: Some("\u{1B}[31mP"),
    buildable: true,
    multiple_allowed: true,
});

const TERRAIN: [[Terrain; 14]; 13] = [
    /* */ [N, N, N, N, N, N, N, N, N, N, N, N, N, N],
    /*  */ [N, P, F, P, P, N, N, N, N, N, N, N, P, N],
    /* */ [N, F, F, F, P, R, T, N, P, N, F, F, F, M],
    /*   */ [N, F, F, M, P, P, P, R, P, P, P, F, F, F],
    /* */ [N, N, F, M, M, F, F, P, F, R, P, F, F, F],
    /*   */ [N, N, R, T, M, M, M, F, P, P, P, P, F, F],
    /* */ [N, N, N, F, M, M, M, F, P, P, P, P, F, F],
    /*   */ [N, N, N, M, M, M, M, F, P, P, P, P, P, N],
    /* */ [N, N, N, F, F, M, M, F, P, P, P, P, P, N],
    /*   */ [N, N, N, N, F, F, M, F, F, T, R, P, P, N],
    /* */ [N, N, N, N, N, F, M, F, F, F, N, N, N, N],
    /*   */ [N, N, N, N, N, F, F, F, F, F, N, N, N, N],
    /* */ [N, N, N, N, N, N, N, F, N, N, N, N, N, N],
];

const WATER_1_COST: usize = 1;
const WATER_2_COST: usize = 3;

static FEATURES: LazyLock<HashMap<(usize, usize), Feature>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        (2, 5),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Port of Strahan".to_string()),
            revenue: Some(vec![2, 2, 0, 0, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (10, 9),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Hobart".to_string()),
            revenue: Some(vec![5, 5, 4, 4, 3, 3]),
            additional_cost: 0,
        },
    );
    m.insert(
        (9, 9),
        Feature {
            feature_type: FeatureType::Town,
            location_name: Some("New Norfolk".to_string()),
            revenue: Some(vec![2, 2, 2, 2, 2, 2]),
            additional_cost: 0,
        },
    );
    m.insert(
        (2, 5),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Burnie".to_string()),
            revenue: Some(vec![2, 2, 1, 1, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (2, 6),
        Feature {
            feature_type: FeatureType::Town,
            location_name: Some("Ulverstone".to_string()),
            revenue: Some(vec![2, 2, 1, 1, 1, 1]),
            additional_cost: 0,
        },
    );
    m.insert(
        (7, 3),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Devonport".to_string()),
            revenue: Some(vec![3, 3, 1, 1, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (9, 4),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Launceston".to_string()),
            revenue: Some(vec![3, 3, 1, 1, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (3, 5),
        Feature {
            feature_type: FeatureType::Town,
            location_name: Some("Queenstown".to_string()),
            revenue: Some(vec![2, 2, 2, 2, 2, 2]),
            additional_cost: 0,
        },
    );
    let water_features = vec![
        (FeatureType::Water1, (8, 2)),
        (FeatureType::Water1, (8, 3)),
        (FeatureType::Water2, (8, 5)),
        (FeatureType::Water1, (9, 6)),
        (FeatureType::Water2, (3, 7)),
        (FeatureType::Water1, (4, 7)),
        (FeatureType::Water1, (6, 8)),
        (FeatureType::Water1, (6, 9)),
        (FeatureType::Water1, (10, 9)),
        (FeatureType::Water2, (5, 11)),
        (FeatureType::Water2, (9, 11)),
        (FeatureType::Water1, (6, 11)),
    ];

    water_features
        .into_iter()
        .for_each(|(feature_type, (x, y))| {
            let cost = match feature_type {
                FeatureType::Water1 => WATER_1_COST,
                FeatureType::Water2 => WATER_2_COST,
                _ => unreachable!(),
            };
            m.insert(
                (x, y),
                Feature {
                    feature_type,
                    location_name: None,
                    revenue: None,
                    additional_cost: cost,
                },
            );
        });
    m
});

const INITIAL_TRACK: [Track; 4] = [
    Track {
        location: (9, 4),
        track_type: TrackType::CompanyOwned(Company::LW),
    },
    Track {
        location: (9, 4),
        track_type: TrackType::CompanyOwned(Company::TMLC),
    },
    Track {
        location: (3, 5),
        track_type: TrackType::CompanyOwned(Company::EBRC),
    },
    Track {
        location: (2, 4),
        track_type: TrackType::Narrow,
    },
];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EBRAction {
    Bid(usize),
    Pass,
    MoveCube(ChoosableAction, ChoosableAction),
}

impl Action for EBRAction {
    type StateType = EBRState;
    fn execute(&self, state: &Self::StateType) -> Self::StateType {
        match self {
            EBRAction::Bid(bid) => {
                let mut state = state.clone();
                let stage = state.stage;
                match stage {
                    Stage::Auction {
                        lot,
                        initial_auction,
                        passed,
                        ..
                    } => {
                        let Actor::Player(actor) = state.next_actor else {
                            unreachable!()
                        };
                        let mut next_actor = (&actor + 1) % state.player_count;
                        while (passed.contains(&next_actor)) {
                            next_actor = (&next_actor + 1) % state.player_count;
                        }
                        state.stage = Stage::Auction {
                            current_bid: Some(*bid as isize),
                            lot,
                            initial_auction,
                            winning_bidder: Some(actor),
                            passed,
                        };
                        state.next_actor = Actor::Player(next_actor);
                    }
                    _ => unreachable!(),
                }
                state
            }
            EBRAction::Pass => {
                let mut state = state.clone();
                let stage = state.stage.clone();
                match stage {
                    Stage::Auction {
                        current_bid,
                        lot,
                        initial_auction,
                        winning_bidder,
                        mut passed,
                    } => {
                        println!("Passed.len is {:?}", passed.len());
                        // -2 because need all but one to have passed, and one
                        // isn't on the list yet
                        if passed.len() < (state.player_count - 2) as usize {
                            let Actor::Player(mut next_actor) = state.next_actor else {
                                unreachable!()
                            };
                            passed.insert(next_actor as u8);
                            println!("Passed is {:?}", passed);
                            while (passed.contains(&next_actor)) {
                                next_actor = (&next_actor + 1) % state.player_count;
                            }
                            state.next_actor = Actor::Player(winning_bidder.unwrap());
                            state.stage = Stage::Auction {
                                initial_auction,
                                lot,
                                current_bid,
                                winning_bidder,
                                passed: passed,
                            };
                            return state;
                        };
                        println!("Everybody passed");
                        // Everybody has passed.
                        state
                            .holdings
                            .get_mut(&winning_bidder.unwrap())
                            .unwrap()
                            .push(lot.clone());
                        *state.player_cash.get_mut(&winning_bidder.unwrap()).unwrap() -=
                            current_bid.unwrap_or(0) as isize;
                        // Either next player, or next auction (for initial auction)
                        if initial_auction {
                            if lot == Company::GT {
                                // End of initial auction
                                state.stage = Stage::ChooseAction;
                                state.next_actor = Actor::Player(winning_bidder.unwrap());
                            } else {
                                state.stage = Stage::Auction {
                                    initial_auction: true,
                                    current_bid: None,
                                    // Todo: Use the constant
                                    lot: match lot {
                                        Company::LW => Company::TMLC,
                                        Company::TMLC => Company::EBRC,
                                        Company::EBRC => Company::GT,
                                        _ => unreachable!(),
                                    },
                                    winning_bidder: None,
                                    passed: HashSet::new(),
                                }
                            }
                        } else {
                            state.stage = Stage::ChooseAction;
                            state.next_actor =
                                Actor::Player((state.active_player + 1) % state.player_count);
                        }
                    }
                    _ => unreachable!(),
                }
                state
            }
            EBRAction::MoveCube(from, to) => {
                let mut state = state.clone();
                // Find index of cube to remove
                let remove_idx = state
                    .action_cubes
                    .iter()
                    .enumerate()
                    .find(|(i, &cube)| cube && ACTION_CUBE_SPACES[*i] == *from)
                    .unwrap()
                    .0;
                let add_idx = state
                    .action_cubes
                    .iter()
                    .enumerate()
                    .find(|(i, &cube)| !cube && ACTION_CUBE_SPACES[*i] == *to)
                    .unwrap()
                    .0;
                state.action_cubes[remove_idx] = false;
                state.action_cubes[add_idx] = true;
                state
            }
        }
    }
}

type PlayerID = u8;

#[derive(Clone, Debug)]
enum TrackType {
    CompanyOwned(Company),
    Narrow,
}

#[derive(Clone, Debug)]
struct Track {
    location: Coordinate,
    track_type: TrackType,
}

#[derive(Clone, Debug, PartialEq)]
enum Stage {
    Auction {
        initial_auction: bool,
        current_bid: Option<isize>,
        lot: Company,
        winning_bidder: Option<PlayerID>,
        passed: HashSet<PlayerID>,
    },
    BuildTrack {
        company: Company,
        completed_builds: u8,
    },
    ChooseAction,
    TakeResources {
        company: Company,
        taken_resources: u8,
    },
}

#[derive(Clone, Debug)]
pub struct EBRState {
    next_actor: Actor<EBRAction>,
    active_player: PlayerID,
    player_count: u8,
    track: Vec<Track>,
    resources: Vec<Coordinate>,
    stage: Stage,
    holdings: HashMap<PlayerID, Vec<Company>>,
    player_cash: HashMap<PlayerID, isize>,
    action_cubes: ActionCubeSpaces,
}

impl EBRState {}

impl State for EBRState {
    type ActionType = EBRAction;

    fn next_actor(&self) -> Actor<EBRAction> {
        self.next_actor.clone()
    }

    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        match &self.stage {
            Stage::Auction {
                initial_auction,
                current_bid,
                lot,
                winning_bidder,
                passed,
            } => {
                let Actor::Player(next_actor) = self.next_actor else {
                    unreachable!()
                };
                let player_cash = *self.player_cash.get(&next_actor).unwrap();
                if (current_bid.unwrap_or(-1) as isize) < player_cash {
                    [
                        (((current_bid.unwrap_or(0) + 1) as isize)..player_cash)
                            .map(|bid| EBRAction::Bid(bid as usize))
                            .collect(),
                        vec![if *initial_auction && (*current_bid == None) {
                            EBRAction::Bid(0)
                        } else {
                            EBRAction::Pass
                        }],
                    ]
                    .concat()
                } else {
                    vec![if *initial_auction && (*current_bid == None) {
                        EBRAction::Bid(0)
                    } else {
                        EBRAction::Pass
                    }]
                }
            }
            Stage::ChooseAction => {
                let removable_action_cubes = self
                    .action_cubes
                    .iter()
                    .enumerate()
                    .filter(|(_, &cube)| cube)
                    .map(|(i, _)| ACTION_CUBE_SPACES[i])
                    // BTreeSet as wanted the order, and perf was worth it
                    .collect::<BTreeSet<ChoosableAction>>();
                let mut addable_action_cubes = self
                    .action_cubes
                    .iter()
                    .enumerate()
                    .filter(|(_, &cube)| !cube)
                    .map(|(i, _)| ACTION_CUBE_SPACES[i])
                    .collect::<BTreeSet<ChoosableAction>>();
                // placeholders
                let can_merge_any = true;
                let can_build_any = true;
                let can_take_any = true;
                let can_issue_any = true;
                if !can_merge_any {
                    addable_action_cubes.remove(&ChoosableAction::Merge);
                };
                if !can_build_any {
                    addable_action_cubes.remove(&ChoosableAction::BuildTrack);
                }
                if !can_take_any {
                    addable_action_cubes.remove(&ChoosableAction::TakeResources);
                }
                if !can_issue_any {
                    addable_action_cubes.remove(&ChoosableAction::IssueBond);
                }

                let mut actions: Vec<EBRAction> = vec![];
                for remove_action in &removable_action_cubes {
                    for add_action in &addable_action_cubes {
                        if remove_action != add_action {
                            actions.push(EBRAction::MoveCube(*remove_action, *add_action));
                        }
                    }
                }

                actions
            }
            _ => {
                vec![]
            }
        }
    }

    fn reward(&self) -> Vec<f64> {
        vec![0.0f64, 0.0f64, 0.0f64, 0.0f64]
    }

    fn terminal(&self) -> bool {
        false
    }
}

pub struct EBR {
    pub player_count: u8,
}

impl Game for EBR {
    type StateType = EBRState;
    type ActionType = EBRAction;

    fn init_game(&self) -> Self::StateType {
        EBRState {
            next_actor: Actor::Player(0),
            player_count: self.player_count,
            track: INITIAL_TRACK.to_vec(),
            resources: vec![],
            active_player: 0,
            stage: Stage::Auction {
                initial_auction: true,
                current_bid: None,
                lot: Company::LW,
                winning_bidder: None,
                passed: HashSet::new(),
            },
            holdings: (0..self.player_count)
                .map(|i| (i, Vec::new()))
                .collect::<HashMap<u8, Vec<Company>>>(),
            player_cash: (0..self.player_count)
                .map(|i| (i, 24 / self.player_count as isize))
                .collect::<HashMap<u8, isize>>(),
            action_cubes: ACTION_CUBE_INIT,
        }
    }

    fn visualise_state(&self, state: &Self::StateType) {
        println!("Resources: {:?}", state.resources);
        println!("Track:");
        for track in &state.track {
            println!("{:?}", track);
        }
        println!("Stage: {:?}", state.stage);
        println!("Active player: {}", state.active_player);
        println!("Player count: {}", state.player_count);
        println!("{:?}", state);
    }
}
