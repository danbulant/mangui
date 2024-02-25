use std::sync::{RwLock, Arc, mpsc, Mutex};

use mangui::{self, nodes::layout::Layout, SharedNode, MainEntry};

mod component_demo_syntax;
mod anilist;
mod slot_demo;

use rusalka::component::Component;

fn main() {
    let (tx, rx) = mpsc::channel();
    let _tx = Arc::new(tx);
    let root = Layout::default();
    let groot: SharedNode = Arc::new(RwLock::new(root));

    let cdemo: Arc<Mutex<component_demo_syntax::ComponentDemo>> = Arc::new_cyclic(|cself|
        Mutex::new(component_demo_syntax::ComponentDemo::new(component_demo_syntax::ComponentDemoAttributes {
            radius: 15.
        }, cself.clone()))
    );
    cdemo.lock().unwrap().mount(&groot, None);

    mangui::run_event_loop(MainEntry {
        root: groot.clone(),
        render: rx
    });
}
