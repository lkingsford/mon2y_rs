use crate::mon2y::game::{Action, State};
pub trait Game {
    type StateType: State<ActionType = Self::ActionType>;
    type ActionType: Action<StateType = Self::StateType>;
    fn get_human_turn(&self, state: &Self::StateType) -> Self::ActionType;
    fn visualise_state(&self, state: &Self::StateType);
    fn init_game(&self) -> Self::StateType;
}
