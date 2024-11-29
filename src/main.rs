use std::collections::HashMap;

trait Action {
    type StateType: State<ActionType = Self>;
    fn execute(&self, state: &Self::StateType) -> Self::StateType;
}

enum C4Action {
    Drop(u8),
}

impl Action for C4Action {
    type StateType = C4State;
    fn execute(&self, state: &C4State) -> C4State {
        todo!()
    }
}

enum Actor {
    Player(u8),
    GameAction,
}

type Reward = Vec<f64>;

trait State {
    type ActionType: Action<StateType = Self>;
    fn permitted_actions(&self) -> Vec<Self::ActionType>;
    fn next_actor(&self) -> Actor;
    fn terminal(&self) -> bool;
}

enum C4Cell {
    Empty,
    Filled(u8),
}

struct C4State {
    board: Vec<C4Cell>,
    next_player: u8,
}

impl State for C4State {
    type ActionType = C4Action;
    fn permitted_actions(&self) -> Vec<Self::ActionType> {
        todo!()
    }
    fn next_actor(&self) -> Actor {
        Actor::Player(self.next_player)
    }
    fn terminal(&self) -> bool {
        todo!()
    }
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

impl ExpandedNode<C4State, C4Action> {
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
