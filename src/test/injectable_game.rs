use crate::mon2y::game::{Action, Actor, State};

///
/// A generic test game that can have injected reward, terminal state, and permitted actions
/// to test tree and node related things.
///
#[derive(Clone, Debug)]
pub struct TestGameState {
    pub injected_reward: Vec<f64>,
    pub injected_terminal: bool,
    pub injected_permitted_actions: Vec<TestGameAction>,
    pub player_count: u8,
    pub next_player_id: u8,
}

impl State for TestGameState {
    type ActionType = TestGameAction;
    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        self.injected_permitted_actions.clone()
    }
    fn next_actor(&self) -> Actor<Self::ActionType> {
        Actor::Player(self.next_player_id)
    }
    fn reward(&self) -> Vec<f64> {
        return self.injected_reward.clone();
    }

    fn terminal(&self) -> bool {
        return self.injected_terminal;
    }
}

#[derive(Hash, Copy, Clone, Eq, PartialEq, Debug)]
pub enum TestGameAction {
    Win,
    WinInXTurns(u8),
    NextTurnInjectActionCount(u8),
}
impl Action for TestGameAction {
    type StateType = TestGameState;
    fn execute(&self, state: &Self::StateType) -> Self::StateType {
        let next_player_id = if let Actor::Player(player_id) = state.next_actor() {
            (player_id + 1) % state.player_count
        } else {
            panic!("Not a player")
        };
        match self {
            TestGameAction::NextTurnInjectActionCount(c) => TestGameState {
                injected_permitted_actions: (0..*c)
                    .map(|i| TestGameAction::WinInXTurns(i))
                    .collect(),
                next_player_id,
                ..state.clone()
            },
            TestGameAction::WinInXTurns(turns) => TestGameState {
                injected_permitted_actions: {
                    if (*turns > 0) {
                        vec![TestGameAction::WinInXTurns(turns - 1)]
                    } else {
                        vec![TestGameAction::Win]
                    }
                },
                next_player_id,
                ..state.clone()
            },
            TestGameAction::Win => TestGameState {
                injected_terminal: true,
                injected_reward: vec![1.0],
                next_player_id,
                ..state.clone()
            },
        }
    }
}
