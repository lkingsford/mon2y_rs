use mon2y_rs::c4;
use mon2y_rs::c4::C4;
use mon2y_rs::game::Game;
use mon2y_rs::mon2y::game::{Action, State};
use mon2y_rs::mon2y::node::create_expanded_node;
use mon2y_rs::mon2y::tree::Tree;
use mon2y_rs::mon2y::{calculate_best_turn, BestTurnPolicy};
use test_env_log::test;

#[test]
fn test_c4_one_action_blocks_win() {
    let mut c4_state = C4.init_game();
    for action in vec![
        c4::C4Action::Drop(0),
        c4::C4Action::Drop(1),
        c4::C4Action::Drop(0),
        c4::C4Action::Drop(1),
        c4::C4Action::Drop(0),
    ] {
        c4_state = action.execute(&c4_state);
    }
    let action = calculate_best_turn(100, 1, c4_state, BestTurnPolicy::MostVisits);
    assert_eq!(action, c4::C4Action::Drop(0));
}

#[test]
fn test_c4_one_action_gets_win() {
    let mut c4_state = C4.init_game();
    for action in vec![
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(1),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(1),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(1),
    ] {
        c4_state = action.execute(&c4_state);
    }
    let action = calculate_best_turn(100, 1, c4_state, BestTurnPolicy::MostVisits);
    assert_eq!(action, c4::C4Action::Drop(3));
}

#[test]
fn test_c4_play_out_repeated() {
    env_logger::init();
    let mut c4_state = C4.init_game();
    for action in vec![
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(1),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(1),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(1),
    ] {
        c4_state = action.execute(&c4_state);
    }
    let mut root_node = create_expanded_node(c4_state);
    let tree = Tree::new(root_node);
    let mut p0_wins = 0.0;
    let mut p1_wins = 0.0;
    for _ in 0..1000 {
        let root_ref = tree.root.clone();
        let root = root_ref.read().unwrap();
        let result = tree.play_out(root.state().clone());
        if result[0] > 0.0 {
            p0_wins += 1.0;
        };
        if result[1] > 0.0 {
            p1_wins += 1.0;
        };
    }
    assert!(p0_wins > p1_wins);
}

#[test]
fn test_c4_plays_through_without_crash() {
    let mut c4_state = C4.init_game();
    while !c4_state.terminal() {
        if let mon2y_rs::mon2y::game::Actor::Player(player) = c4_state.next_actor() {
            let action = calculate_best_turn(100, 1, c4_state.clone(), BestTurnPolicy::MostVisits);
            c4_state = action.execute(&c4_state);
        }
    }
}
#[test]
fn test_c4_plays_through_multiple_threads_without_crash() {
    let mut c4_state = C4.init_game();
    while !c4_state.terminal() {
        if let mon2y_rs::mon2y::game::Actor::Player(player) = c4_state.next_actor() {
            let action = calculate_best_turn(100, 4, c4_state.clone(), BestTurnPolicy::MostVisits);
            c4_state = action.execute(&c4_state);
        }
    }
}

#[test]
fn test_c4_full_exploration() {
    // This is more of a test that it doesn't freeze when getting fully explored
    // is very likely.
    let mut c4_state = C4.init_game();
    for action in vec![
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(0),
        c4::C4Action::Drop(1),
        c4::C4Action::Drop(0),
        c4::C4Action::Drop(0),
        c4::C4Action::Drop(0),
        c4::C4Action::Drop(4),
        c4::C4Action::Drop(4),
        c4::C4Action::Drop(4),
        c4::C4Action::Drop(6),
        c4::C4Action::Drop(5),
        c4::C4Action::Drop(6),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(3),
        c4::C4Action::Drop(2),
        c4::C4Action::Drop(2),
        c4::C4Action::Drop(2),
        c4::C4Action::Drop(2),
        c4::C4Action::Drop(2),
    ] {
        c4_state = action.execute(&c4_state);
    }

    calculate_best_turn(100000, 8, c4_state, BestTurnPolicy::MostVisits);
}
