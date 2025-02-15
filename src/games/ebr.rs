use linked_hash_set::LinkedHashSet;
use log::warn;
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
pub enum ChoosableAction {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Bond {
    face_value: usize,
    coupon: usize,
}
const BONDS: [Bond; 7] = [
    Bond {
        face_value: 5,
        coupon: 1,
    },
    Bond {
        face_value: 5,
        coupon: 1,
    },
    Bond {
        face_value: 10,
        coupon: 3,
    },
    Bond {
        face_value: 10,
        coupon: 3,
    },
    Bond {
        face_value: 10,
        coupon: 4,
    },
    Bond {
        face_value: 15,
        coupon: 4,
    },
    Bond {
        face_value: 15,
        coupon: 5,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
struct BondDetails {
    bond: Bond,
    deferred: bool,
}

static INITIAL_CASH: LazyLock<HashMap<u8, u32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(2, 20);
    m.insert(3, 13);
    m.insert(4, 10);
    m.insert(5, 8);
    m
});

#[derive(Debug, Clone)]
struct Feature {
    feature_type: FeatureType,
    location_name: Option<String>,
    revenue: [isize; 6],
    additional_cost: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FeatureType {
    Port,
    Town,
    Water1,
    Water2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
enum Company {
    EBRC,
    LW,
    TMLC,
    GT,
    NMFT,
    NED,
    MLM,
}

const ALL_COMPANIES: [Company; 7] = [
    Company::EBRC,
    Company::LW,
    Company::TMLC,
    Company::GT,
    Company::NMFT,
    Company::NED,
    Company::MLM,
];

const IPO_ORDER: [Company; 4] = [Company::LW, Company::TMLC, Company::EBRC, Company::GT];
static PRIVATE_ORDER: LazyLock<Vec<Company>> =
    LazyLock::new(|| vec![Company::GT, Company::NMFT, Company::NED, Company::MLM]);

struct CompanyFixedDetails {
    starting: Option<Coordinate>,
    private: bool,
    stock_available: usize,
    track_available: usize,
    initial_treasury: usize,
    initial_interest: usize,
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

const INITIAL_RESOURCE_CUBES: [Coordinate; 4] = [(2, 4), (2, 3), (3, 4), (3, 4)];
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompanyDetails {
    shares_held: usize,
    shares_remaining: usize,
    merged: Option<bool>,
    cash: isize,
    available: Option<bool>,
    hq: Option<Coordinate>,
    track_remaining: usize,
    bonds: Vec<BondDetails>,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
struct CommonAttributes {
    build_cost: u32,
    symbol: Option<&'static str>,
    buildable: bool,
    multiple_allowed: bool,
    revenue: [isize; 6],
}

const FINAL_DIVIDEND_COUNT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
enum Terrain {
    Nothing,
    Plain,
    Forest,
    Mountain,
    Town,
    Port,
}

static TERRAIN_ATTRIBUTES: LazyLock<HashMap<Terrain, CommonAttributes>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(
        Terrain::Nothing,
        CommonAttributes {
            build_cost: 0,
            symbol: None,
            buildable: false,
            multiple_allowed: false,
            revenue: [0, 0, 0, 0, 0, 0],
        },
    );
    map.insert(
        Terrain::Plain,
        CommonAttributes {
            build_cost: 3,
            symbol: Some("\u{1B}[37m-"),
            buildable: true,
            multiple_allowed: true,
            revenue: [0, 0, 0, 0, 0, 0],
        },
    );
    map.insert(
        Terrain::Forest,
        CommonAttributes {
            build_cost: 4,
            symbol: Some("\u{1B}[32m="),
            buildable: true,
            multiple_allowed: false,
            revenue: [1, 1, 1, 1, 0, 0],
        },
    );
    map.insert(
        Terrain::Mountain,
        CommonAttributes {
            build_cost: 6,
            symbol: Some("\u{1B}[32m^"),
            multiple_allowed: false,
            buildable: true,
            revenue: [0, 0, 0, 0, 0, 0],
        },
    );
    map.insert(
        Terrain::Town,
        CommonAttributes {
            build_cost: 4,
            symbol: Some("\u{1B}[33mT"),
            buildable: true,
            multiple_allowed: true,
            revenue: [0, 0, 0, 0, 0, 0],
        },
    );
    map.insert(
        Terrain::Port,
        CommonAttributes {
            build_cost: 5,
            symbol: Some("\u{1B}[31mP"),
            buildable: true,
            multiple_allowed: true,
            revenue: [0, 0, 0, 0, 0, 0],
        },
    );
    map
});

impl Terrain {
    fn attributes(&self) -> &CommonAttributes {
        &TERRAIN_ATTRIBUTES[self]
    }
}

const N: Terrain = Terrain::Nothing;
const P: Terrain = Terrain::Plain;
const F: Terrain = Terrain::Forest;
const M: Terrain = Terrain::Mountain;
const T: Terrain = Terrain::Town;
const R: Terrain = Terrain::Port;

const HEIGHT: usize = 13;
const WIDTH: usize = 14;

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

static PRIVATE_STARTING_LOCATIONS: LazyLock<Vec<Coordinate>> = LazyLock::new(|| {
    TERRAIN
        .iter()
        .enumerate()
        .flat_map(|(y, column)| {
            column
                .iter()
                .enumerate()
                .filter(|(x, cell)| **cell == Terrain::Mountain || **cell == Terrain::Forest)
                .map(move |(x, _cell)| (x, y))
        })
        .collect::<Vec<Coordinate>>()
});
// Privates can start anywhere on a Forest or Mountain (without an existing HQ,
// but obviously, that bit is state dependent)

static FEATURES: LazyLock<HashMap<(usize, usize), Feature>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        (2, 5),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Port of Strahan".to_string()),
            revenue: ([2, 2, 0, 0, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (10, 9),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Hobart".to_string()),
            revenue: ([5, 5, 4, 4, 3, 3]),
            additional_cost: 0,
        },
    );
    m.insert(
        (9, 9),
        Feature {
            feature_type: FeatureType::Town,
            location_name: Some("New Norfolk".to_string()),
            revenue: ([2, 2, 2, 2, 2, 2]),
            additional_cost: 0,
        },
    );
    m.insert(
        (2, 5),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Burnie".to_string()),
            revenue: ([2, 2, 1, 1, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (2, 6),
        Feature {
            feature_type: FeatureType::Town,
            location_name: Some("Ulverstone".to_string()),
            revenue: ([2, 2, 1, 1, 1, 1]),
            additional_cost: 0,
        },
    );
    m.insert(
        (7, 3),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Devonport".to_string()),
            revenue: ([3, 3, 1, 1, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (9, 4),
        Feature {
            feature_type: FeatureType::Port,
            location_name: Some("Launceston".to_string()),
            revenue: ([3, 3, 1, 1, 0, 0]),
            additional_cost: 0,
        },
    );
    m.insert(
        (3, 5),
        Feature {
            feature_type: FeatureType::Town,
            location_name: Some("Queenstown".to_string()),
            revenue: ([2, 2, 2, 2, 2, 2]),
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
                    revenue: [0, 0, 0, 0, 0, 0],
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
const NARROW_GAUGE_INITIAL: usize = 12;
const MAX_BUILDS: u8 = 3;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EBRAction {
    Bid(usize),
    Pass,
    MoveCube(ChoosableAction, ChoosableAction),
    Stalemate,
    ChooseAuctionCompany(Company),
    StartPrivateAt(Company, Coordinate),
    ChooseBuildCompany(Company),
    BuildTrack(Coordinate),
    BuildPass,
    ChooseBondCompany(Company),
    IssueBond(Company, Bond),
}

impl Action for EBRAction {
    type StateType = EBRState;
    fn execute(&self, state: &Self::StateType) -> Self::StateType {
        match self {
            EBRAction::Stalemate => {
                let mut state = state.clone();
                state.terminal = true;
                state
            }
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
                        // -2 because need all but one to have passed, and one
                        // isn't on the list yet
                        if passed.len() < (state.player_count - 2) as usize {
                            let Actor::Player(mut next_actor) = state.next_actor else {
                                unreachable!()
                            };
                            passed.insert(next_actor as u8);
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
                        // Everybody has passed.
                        state
                            .holdings
                            .get_mut(&winning_bidder.unwrap())
                            .unwrap()
                            .push(lot.clone());
                        *state.player_cash.get_mut(&winning_bidder.unwrap()).unwrap() -=
                            current_bid.unwrap_or(0) as isize;
                        {
                            let company_details = state.company_details.get_mut(&lot).unwrap();
                            company_details.shares_held += 1;
                            company_details.shares_remaining -= 1;
                            company_details.cash += current_bid.unwrap();
                        }
                        if COMPANY_FIXED_DETAILS[&lot].private {
                            let index = PRIVATE_ORDER.iter().position(|c| *c == lot).unwrap();
                            if index != PRIVATE_ORDER.len() - 1 {
                                state
                                    .company_details
                                    .get_mut(&PRIVATE_ORDER[index + 1])
                                    .unwrap()
                                    .available = Some(true);
                            }
                            state.company_details.get_mut(&lot).unwrap().available = Some(false);
                        }
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
                let Actor::Player(next_actor) = state.next_actor else {
                    unreachable!()
                };
                state.active_player = next_actor;
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
                match to {
                    ChoosableAction::AuctionShare => state.stage = Stage::ChooseAuctionCompany,
                    ChoosableAction::PayDividend => state.pay_dividend(),
                    ChoosableAction::BuildTrack => state.stage = Stage::ChooseBuildCompany,
                    ChoosableAction::IssueBond => state.stage = Stage::ChooseBondCompany,
                    _ => warn!("Not implemented yet"),
                }
                state
            }
            EBRAction::ChooseAuctionCompany(company) => {
                let mut state = state.clone();
                if !COMPANY_FIXED_DETAILS[&company].private {
                    state.stage = Stage::Auction {
                        initial_auction: false,
                        current_bid: None,
                        lot: *company,
                        winning_bidder: None,
                        passed: HashSet::new(),
                    };
                } else {
                    state.stage = Stage::ChoosePrivateStart(*company);
                }
                state
            }
            EBRAction::StartPrivateAt(company, location) => {
                let mut state = state.clone();
                // TODO: Add resource cubes around
                state.company_details.get_mut(company).unwrap().hq = Some(*location);
                state.stage = Stage::Auction {
                    initial_auction: false,
                    current_bid: None,
                    lot: *company,
                    winning_bidder: None,
                    passed: HashSet::new(),
                };
                // Place resource cubes around
                let mut potential_locations = get_neighbors(location.clone());
                potential_locations.push(*location);
                for location in potential_locations {
                    if location.0 >= WIDTH || location.1 >= HEIGHT {
                        continue;
                    }
                    let terrain = TERRAIN[location.1][location.0];
                    match terrain {
                        Terrain::Forest => state.resource_cubes.push(location),
                        Terrain::Mountain => {
                            state.resource_cubes.push(location);
                            state.resource_cubes.push(location);
                        }
                        _ => {}
                    };
                }

                state
            }
            EBRAction::ChooseBuildCompany(company) => {
                let mut state = state.clone();
                state.stage = Stage::BuildTrack {
                    company: *company,
                    completed_builds: 0,
                };
                state
            }
            EBRAction::BuildTrack(location) => {
                let mut state = state.clone();
                if let Stage::BuildTrack {
                    company,
                    completed_builds,
                } = state.stage
                {
                    if !COMPANY_FIXED_DETAILS[&company].private {
                        state.track.push(Track {
                            location: *location,
                            track_type: TrackType::CompanyOwned(company.clone()),
                        });
                        let cost = state.owned_cost(*location, None) as isize;
                        if let Some(company_details) = state.company_details.get_mut(&company) {
                            company_details.cash -= cost;
                        }
                    } else {
                        state.track.push(Track {
                            location: *location,
                            track_type: TrackType::Narrow,
                        });
                        let cost = state.narrow_cost(*location) as isize;
                        if let Some(company_details) = state.company_details.get_mut(&company) {
                            company_details.cash -= cost;
                        }
                    }

                    if completed_builds < MAX_BUILDS {
                        state.stage = Stage::BuildTrack {
                            company,
                            completed_builds: completed_builds + 1,
                        }
                    } else {
                        state.stage = Stage::ChooseAction;
                        state.next_actor =
                            Actor::Player((state.active_player + 1) % state.player_count);
                    }
                    state
                } else {
                    unreachable!()
                }
            }
            EBRAction::BuildPass => {
                let mut state = state.clone();
                state.stage = Stage::ChooseAction;
                state.next_actor = Actor::Player((state.active_player + 1) % state.player_count);
                state
            }
            EBRAction::ChooseBondCompany(company) => {
                let mut state = state.clone();
                state.stage = Stage::ChooseBond(company.clone());
                state
            }
            EBRAction::IssueBond(company, bond) => {
                let mut state = state.clone();
                let details = state.company_details.get_mut(&company).unwrap();
                details.cash += bond.face_value as isize;
                details.bonds.push(BondDetails {
                    bond: *bond,
                    deferred: true,
                });
                state.unissued_bonds.retain(|b| *b != *bond);
                state.stage = Stage::ChooseAction;
                state.next_actor = Actor::Player((state.active_player + 1) % state.player_count);
                state
            }
        }
    }
}

type PlayerID = u8;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    ChooseAuctionCompany,
    ChoosePrivateStart(Company),
    ChooseBuildCompany,
    ChooseBondCompany,
    ChooseBond(Company),
}

#[derive(Clone, Debug)]
pub struct EBRState {
    terminal: bool,
    next_actor: Actor<EBRAction>,
    active_player: PlayerID,
    player_count: u8,
    track: Vec<Track>,
    stage: Stage,
    holdings: HashMap<PlayerID, Vec<Company>>,
    player_cash: HashMap<PlayerID, isize>,
    action_cubes: ActionCubeSpaces,
    revenue: HashMap<Company, isize>,
    dividends_paid: usize,
    company_details: HashMap<Company, CompanyDetails>,
    unissued_bonds: Vec<Bond>,
    resource_cubes: Vec<Coordinate>,
    narrow_gauge_remaining: usize,
}

impl EBRState {
    fn min_bid(&self, company: Company) -> isize {
        let rev = self.net_revenue(company.clone());
        let owned_shares = self.company_details[&company].shares_held;
        return max(1, div_ceil(rev, (owned_shares as isize + 1)));
    }

    fn can_auction_any(&self) -> bool {
        let Actor::Player(next_actor) = self.next_actor else {
            unreachable!()
        };
        let cash = self.player_cash[&next_actor];
        if &cash < &1 {
            return false;
        };
        // Check for min bid of at least one company with shares available
        // (including the minors)
        COMPANY_FIXED_DETAILS
            .iter()
            .any(|c| self.can_auction(c.0.clone(), cash))
    }

    fn can_auction(&self, company: Company, cash: isize) -> bool {
        // Not quite sure why this needs a clone
        let company_details = self.company_details[&company].clone();
        let private = COMPANY_FIXED_DETAILS[&company].private;
        ((private
            && company_details
                .available
                .expect("Private Company Details Should Have Available"))
            || (!private && company_details.shares_remaining > 0))
            && (cash >= self.min_bid(company))
    }

    fn can_issue_any(&self) -> bool {
        if self.unissued_bonds.is_empty() {
            return false;
        }
        COMPANY_FIXED_DETAILS
            .iter()
            .any(|c| self.can_issue(c.0.clone()))
    }
    fn can_issue(&self, company: Company) -> bool {
        if COMPANY_FIXED_DETAILS[&company].private {
            return false;
        };

        let Actor::Player(next_actor) = self.next_actor else {
            unreachable!()
        };

        self.holdings[&next_actor].contains(&company)
    }

    fn can_build_any(&self) -> bool {
        let Actor::Player(next_actor) = self.next_actor else {
            unreachable!()
        };
        COMPANY_FIXED_DETAILS
            .iter()
            .any(|c| self.can_build(c.0.clone(), next_actor))
    }

    fn possible_owned_track(&self, company: Company) -> Vec<Coordinate> {
        let company_details = self.company_details.get(&company).unwrap();
        self.track
            .iter()
            .filter(|t| {
                // All owned track
                t.track_type == TrackType::CompanyOwned(company.clone())
            })
            // All neighboring
            .map(|t| get_neighbors(t.location))
            .flatten()
            .collect::<HashSet<Coordinate>>() // Unique
            .iter()
            .filter_map(|t| {
                if t.0 >= WIDTH || t.1 >= HEIGHT {
                    return None;
                }
                let terrain = TERRAIN[t.1][t.0];
                let attr = TERRAIN_ATTRIBUTES[&terrain];
                if !attr.buildable {
                    return None;
                }
                let other_track_in_location = self
                    .track
                    .iter()
                    .map(|ot| ot.clone())
                    .filter(|ot| ot.location == *t)
                    .collect::<Vec<_>>();
                // Can't build more track if not permitted
                if other_track_in_location.len() > 0 && !attr.multiple_allowed {
                    return None;
                }
                // Company can't own multiple track in location
                if other_track_in_location
                    .iter()
                    .any(|t| t.track_type == TrackType::CompanyOwned(company.clone()))
                {
                    return None;
                }
                // Make sure co can pay
                let cost = self.owned_cost(*t, Some(other_track_in_location));
                if (company_details.cash >= cost as isize) {
                    Some(*t)
                } else {
                    None
                }
            })
            .collect()
    }

    fn owned_cost(&self, t: Coordinate, other_track_in_location: Option<Vec<Track>>) -> usize {
        // Other track in location is optional - only calculate if not specified
        let other_track_in_location = other_track_in_location.unwrap_or(
            self.track
                .iter()
                .filter_map(|ot| {
                    if ot.location == t {
                        Some(ot.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<Track>>(),
        );

        // Slight repetition of other places where this is called here
        let terrain = TERRAIN[t.1][t.0];

        let attr = TERRAIN_ATTRIBUTES[&terrain];
        (1 + other_track_in_location.len()) * attr.build_cost as usize
            + FEATURES
                .get(&t)
                .iter()
                .map(|f| f.additional_cost)
                .sum::<usize>()
    }

    fn possible_narrow_track(&self, company: Company) -> Vec<Coordinate> {
        // TODO
        vec![]
    }
    fn narrow_cost(&self, t: Coordinate) -> usize {
        // TODO
        return 0;
    }

    fn can_build(&self, company: Company, player: PlayerID) -> bool {
        let company_details = self.company_details.get(&company).unwrap();
        if !self.holdings.get(&player).unwrap().contains(&company) {
            return false;
        }
        if company_details.merged.unwrap_or(false) {
            return false;
        }
        let company_fixed_details = COMPANY_FIXED_DETAILS.get(&company).unwrap();
        if !company_fixed_details.private {
            if company_fixed_details.track_available == 0 {
                return false;
            }
            self.possible_owned_track(company).len() > 0
        } else {
            if self.narrow_gauge_remaining == 0 {
                return false;
            }
            self.possible_narrow_track(company).len() > 0
        }
    }

    fn net_revenue(&self, company: Company) -> isize {
        let company_track = self
            .track
            .iter()
            .filter(|t| t.track_type == TrackType::CompanyOwned(company.clone()));
        let track_terrain_revenue = company_track
            .clone()
            .map(|t| TERRAIN[t.location.1][t.location.0].attributes().revenue[self.dividends_paid])
            .sum::<isize>();
        let track_feature_revenue = company_track
            .clone()
            .map(
                |t| match FEATURES.get_key_value(&(t.location.0, t.location.1)) {
                    None => 0,
                    Some(feature) => feature.1.revenue[self.dividends_paid],
                },
            )
            .sum::<isize>();
        let bond_interest = self
            .company_details
            .get(&company)
            .unwrap()
            .bonds
            .iter()
            .filter_map(|b| {
                if b.deferred {
                    None
                } else {
                    Some(b.bond.coupon)
                }
            })
            .sum::<usize>();
        track_terrain_revenue + track_feature_revenue - bond_interest as isize
    }

    fn pay_dividend(&mut self) {
        let rev_per_share = self
            .company_details
            .iter()
            .map(|c| {
                (
                    c.0.clone(),
                    if c.1.shares_held > 0 {
                        let rev = self.net_revenue(c.0.clone());
                        // Ceil over 0, floor under 0
                        if rev > 0 {
                            div_ceil(rev, c.1.shares_held as isize)
                        } else {
                            div_ceil(rev * -1, c.1.shares_held as isize) * -1
                        }
                    } else {
                        0
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        self.next_actor = {
            let Actor::Player(actor) = self.next_actor else {
                unreachable!()
            };
            Actor::Player((&actor + 1) % self.player_count)
        };
        self.dividends_paid = self.dividends_paid + 1;
        self.player_cash = self
            .player_cash
            .iter()
            .map(|(player, old_cash)| {
                (
                    *player,
                    old_cash
                        + self.holdings[player]
                            .iter()
                            .map(|company| rev_per_share[company])
                            .sum::<isize>(),
                )
            })
            .collect::<HashMap<u8, isize>>();

        for company in self.company_details.values_mut() {
            for bond in company.bonds.iter_mut() {
                bond.deferred = true;
            }
        }

        self.terminal = self.dividends_paid == 6
            // TODO: Add bankruptcy
            ||
            // Two of these conditions must be met
             vec![
                // No shares unsold
                self.company_details
                    .iter()
                    .filter(|c| c.1.shares_remaining > 0)
                    .count()
                    == 0,
                // <= 2 bonds remaining
                self.unissued_bonds.len() <= 2,
                // TODO: 3/4 charters have no remaining trains
                // <=3 resource cubes on board
                self.resource_cubes.len() <= 3,
                    
            ]
            .iter()
            .filter(|criteria| **criteria)
            .count()
                >= 2
    }
}

impl State for EBRState {
    type ActionType = EBRAction;

    fn next_actor(&self) -> Actor<EBRAction> {
        self.next_actor.clone()
    }

    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        let Actor::Player(next_actor) = self.next_actor else {
            unreachable!()
        };
        match &self.stage {
            Stage::Auction {
                initial_auction,
                current_bid,
                lot,
                winning_bidder,
                passed,
            } => {
                let player_cash = *self.player_cash.get(&next_actor).unwrap();
                if (current_bid.unwrap_or(-1) as isize) < player_cash {
                    let mut actions: Vec<EBRAction> = (((current_bid.unwrap_or(0) + 1) as isize)
                        ..=player_cash)
                        .map(|bid| EBRAction::Bid(bid as usize))
                        .collect();
                    if *initial_auction && (*current_bid == None) {
                        actions.push(EBRAction::Bid(0));
                    } else if (!(*initial_auction) && *current_bid != None)
                        || (*current_bid != None)
                    {
                        actions.push(EBRAction::Pass);
                    }
                    actions
                } else {
                    vec![if *initial_auction && (*current_bid == None) {
                        EBRAction::Bid(0)
                    } else if !(*initial_auction) || (*current_bid != None) {
                        EBRAction::Pass
                    } else {
                        panic!("Somehow, Palapatine has returned")
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
                let can_take_any = true;
                if !can_merge_any {
                    addable_action_cubes.remove(&ChoosableAction::Merge);
                };
                if !self.can_build_any() {
                    addable_action_cubes.remove(&ChoosableAction::BuildTrack);
                }
                if !can_take_any {
                    addable_action_cubes.remove(&ChoosableAction::TakeResources);
                }
                if !self.can_issue_any() {
                    addable_action_cubes.remove(&ChoosableAction::IssueBond);
                }
                if !self.can_auction_any() {
                    addable_action_cubes.remove(&ChoosableAction::AuctionShare);
                }

                let mut actions: Vec<EBRAction> = vec![];
                for remove_action in &removable_action_cubes {
                    for add_action in &addable_action_cubes {
                        if remove_action != add_action {
                            actions.push(EBRAction::MoveCube(*remove_action, *add_action));
                        }
                    }
                }
                if actions.is_empty() {
                    vec![EBRAction::Stalemate]
                } else {
                    actions
                }
            }
            Stage::ChooseAuctionCompany => {
                let cash = self.player_cash[&next_actor];
                COMPANY_FIXED_DETAILS
                    .iter()
                    .filter(|c| self.can_auction(c.0.clone(), cash))
                    .map(|c| EBRAction::ChooseAuctionCompany(c.0.clone()))
                    .collect()
            }
            Stage::ChoosePrivateStart(company) => PRIVATE_STARTING_LOCATIONS
                .iter()
                .filter(|location| {
                    !self
                        .company_details
                        .iter()
                        .any(|c| c.1.hq == Some(**location))
                })
                .map(|location| EBRAction::StartPrivateAt(*company, *location))
                .collect(),
            Stage::ChooseBuildCompany => COMPANY_FIXED_DETAILS
                .iter()
                .filter(|c| self.can_build(c.0.clone(), next_actor))
                .map(|c| EBRAction::ChooseBuildCompany(c.0.clone()))
                .collect(),
            Stage::BuildTrack {
                company,
                completed_builds,
            } => {
                if COMPANY_FIXED_DETAILS[company].private {
                    let mut actions = self
                        .possible_narrow_track(*company)
                        .iter()
                        .map(|coord| EBRAction::BuildTrack(*coord))
                        .collect::<Vec<EBRAction>>();
                    if *completed_builds > 0 {
                        actions.push(EBRAction::BuildPass)
                    };
                    actions
                } else {
                    let mut actions = self
                        .possible_owned_track(*company)
                        .iter()
                        .map(|coord| EBRAction::BuildTrack(*coord))
                        .collect::<Vec<EBRAction>>();
                    if *completed_builds > 0 {
                        actions.push(EBRAction::BuildPass)
                    };
                    actions
                }
            }
            Stage::ChooseBondCompany => COMPANY_FIXED_DETAILS
                .iter()
                .filter(|c| self.can_issue(c.0.clone()))
                .map(|c| EBRAction::ChooseBondCompany(c.0.clone()))
                .collect(),
            Stage::ChooseBond(company) => self
                .unissued_bonds
                .iter()
                .map(|bond| EBRAction::IssueBond(*company, *bond))
                .collect(),
            _ => {
                warn!("Unimplemented Stage in PermittedActions");
                vec![]
            }
        }
    }

    fn reward(&self) -> Vec<f64> {
        // TODO: Improve this - this isn't great. 1 for best, -1 for lost, 0 for others.
        let mut cash_rewards = vec![0f64; self.player_count as usize];
        let mut sorted_cash: Vec<(u8, isize)> = self
            .player_cash
            .iter()
            .map(|(player, cash)| (*player, *cash))
            .collect();
        sorted_cash.sort_by(|a, b| b.1.cmp(&a.1));
        cash_rewards[sorted_cash[0].0 as usize] = 1f64;
        if self.player_count > 1 {
            cash_rewards[sorted_cash[self.player_count as usize - 1].0 as usize] = -1f64;
        }
        cash_rewards
    }

    fn terminal(&self) -> bool {
        self.terminal
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
            terminal: false,
            next_actor: Actor::Player(0),
            player_count: self.player_count,
            track: INITIAL_TRACK.to_vec(),
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
            revenue: ALL_COMPANIES.iter().map(|c| (c.clone(), 0)).collect(),
            action_cubes: ACTION_CUBE_INIT,
            dividends_paid: 0,
            company_details: COMPANY_FIXED_DETAILS
                .iter()
                .map(|d| {
                    (
                        d.0.clone(),
                        CompanyDetails {
                            shares_held: 0,
                            shares_remaining: d.1.stock_available,
                            merged: if d.1.private { Some(false) } else { None },
                            cash: d.1.initial_treasury as isize,
                            available: if d.1.private { Some(false) } else { None },
                            hq: d.1.starting,
                            track_remaining: d.1.track_available,
                            bonds: vec![BondDetails {
                                bond: Bond {
                                    face_value: d.1.initial_treasury,
                                    coupon: d.1.initial_interest,
                                },
                                deferred: true,
                            }],
                        },
                    )
                })
                .collect(),
            unissued_bonds: BONDS.iter().map(|b| b.clone()).collect::<Vec<Bond>>(),
            resource_cubes: INITIAL_RESOURCE_CUBES.to_vec(),
            narrow_gauge_remaining: NARROW_GAUGE_INITIAL,
        }
    }

    fn visualise_state(&self, state: &Self::StateType) {
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

fn div_ceil(numerator: isize, denominator: isize) -> isize {
    // Slightly cheeky
    // Look - it's used enough places that it's worth it, and frankly, it's clearer like this
    (numerator + denominator - 1) / denominator
}

/// Game is a hex map with pointy sides
/// Each row is top, bottom, top, bottom
///
/// 1,1        3, 1       5,1
///      2,1        4, 1
/// 1,2        3, 2,      5,2
///      2,2        4, 2
/// 1,3        3, 3       5,3
///      2,3        4, 3
/// This doesn't take into account the map
fn get_neighbors(coord: Coordinate) -> Vec<Coordinate> {
    let (x, y) = coord;
    if y % 2 == 1 {
        vec![
            (x - 1, y - 1), // Top-left
            (x + 1, y - 1), // Top-right
            (x + 1, y),     // Right
            (x + 1, y + 1), // Bottom-right
            (x, y + 1),     // Bottom-left
            (x - 1, y),     // Left
        ]
    } else {
        vec![
            (x, y - 1),     // Top-left
            (x + 1, y - 1), // Top-right
            (x + 1, y),     // Right
            (x, y + 1),     // Bottom-right
            (x - 1, y + 1), // Bottom-left
            (x - 1, y),     // Left
        ]
    }
}
