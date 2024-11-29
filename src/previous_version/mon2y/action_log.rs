use super::action::Action;
use super::state::State;
use std::string::String;

pub struct ActionLogEntry {
    pub action: Action,
    pub player_id: Option<i32>,
    pub state: Box<dyn State>,
    pub memo: Option<String>,
}
