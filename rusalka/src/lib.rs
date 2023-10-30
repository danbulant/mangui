use std::sync::{Arc, Mutex, RwLock};

use component::Component;
use mangui::nodes::Node;

pub mod component;
pub mod nodes;

pub type SharedComponent<T: Component> = Arc<Mutex<T>>;
pub type SharedNodeComponent<T: Node> = Arc<RwLock<T>>;