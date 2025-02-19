use std::collections::HashMap;

pub type Annotation = HashMap<String, AnnotationElement>;

pub enum AnnotationElement {
    Text(String),
    Bool(bool),
    Int(isize),
    Float(f64),
    None,
}
