use std::sync::{Arc, Mutex, RwLock};

use component::Component;
use mangui::nodes::Node;

pub mod component;
pub mod nodes;
pub mod store;

pub type SharedComponent<T: Component> = Arc<Mutex<T>>;
pub type WeakSharedComponent<T: Component> = std::sync::Weak<Mutex<T>>;
pub type SharedNodeComponent<T: Node> = Arc<RwLock<T>>;