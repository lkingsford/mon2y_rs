use super::state::State;
use std::string::String;

pub enum Action {
    Str(String),
    Num(i32),
    NoAct(bool),
}

pub struct ActionLogEntry {
    action: Action,
    player_id: Option<i32>,
    state: Box<dyn State>,
    memo: Option<String>,
}
