#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Str(String),
    Num(i32),
    NoAct,
}
