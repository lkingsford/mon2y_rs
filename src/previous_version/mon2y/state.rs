use serde_json::Value;
use std::any::Any;

pub trait State {
    fn loggable(&self) -> Value;
    fn as_any(&self) -> &dyn Any;
}
