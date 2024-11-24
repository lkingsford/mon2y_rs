use serde_json::Value;

pub trait State {
    fn copy(&self) -> Box<dyn State>;
    fn loggable(&self) -> Value;
}
