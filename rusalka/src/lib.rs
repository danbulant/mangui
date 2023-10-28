use std::sync::{Arc, Mutex};

use component::Component;

pub mod component;
pub mod nodes;

pub type SharedComponent<T: Component> = Arc<Mutex<T>>;