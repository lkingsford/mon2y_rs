use std::collections::HashMap;
pub trait Action: Clone + Copy {
    type StateType: State<ActionType = Self>;
    fn execute(&self, state: &Self::StateType) -> Self::StateType;
}
///
/// An actor is either a player or a game action.
///
/// A player is just an identifier, typically a number between 0 and n-1.
///
/// A game action is a action that the game takes, rather than a player.
pub enum Actor<ActionType> {
    /// A player is just an identifier, typically a number between 0 and n-1.
    Player(u8),
    /// A game action is a action that the game takes (such as rolling a dice, or drawing a card), rather than a player.
    GameAction(ActionType),
}

type Reward = Vec<f64>;

pub trait State {
    type ActionType: Action<StateType = Self>;
    fn permitted_actions(&self) -> Vec<Self::ActionType>;
    fn next_actor(&self) -> Actor<Self::ActionType>;
    fn terminal(&self) -> bool;
}

enum Node<StateType: State, ActionType: Action> {
    Expanded(ExpandedNode<StateType, ActionType>),
    Placeholder,
}

struct ExpandedNode<StateType: State, ActionType: Action> {
    state: StateType,
    children: HashMap<ActionType, Node<StateType, ActionType>>,
    visit_count: u32,
    value_sum: f64,
}

struct Selection<StateType: State, ActionType: Action> {
    state: StateType,
    path: Vec<ActionType>,
}

impl<StateType: State, ActionType: Action> ExpandedNode<StateType, ActionType> {
    fn fully_explored(&self) -> bool {
        todo!()
    }
}

pub struct Tree<StateType: State, ActionType: Action> {
    root: Node<StateType, ActionType>,
}

impl<StateType: State, ActionType: Action> Tree<StateType, ActionType> {
    pub fn best_pick(&self) -> Vec<ActionType> {
        todo!()
    }

    pub fn selection(&self) -> Selection<StateType, ActionType> {
        todo!()
    }

    pub fn expansion(&mut self, selection: Selection<StateType, ActionType>) {
        todo!()
    }

    pub fn play_out(&self, selection: Selection<StateType, ActionType>) -> Option<Reward> {
        todo!()
    }

    pub fn propagate_reward(
        &mut self,
        selection: Selection<StateType, ActionType>,
        reward: Reward,
    ) {
        todo!()
    }
}
