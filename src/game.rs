use crate::mon2y::{Action, State};
pub trait Game {
    type StateType: State<ActionType = Self::ActionType>;
    type ActionType: Action<StateType = Self::StateType>;
    fn get_human_turn(&self, state: &Self::StateType) -> Self::ActionType;
    fn init_game(&self) -> Self::StateType;
}
