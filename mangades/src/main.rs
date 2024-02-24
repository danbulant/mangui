use std::sync::{RwLock, Arc, mpsc, Mutex};

use mangui::{self, nodes::{layout::Layout, self, Style, TaffyStyle}, taffy::{self, prelude::Size, style::Dimension}, femtovg::{Paint, Color}, SharedNode, MainEntry, events::NodeEvent};

mod component_demo;
mod component_demo_syntax;
mod anilist;

use rusalka::component::Component;

fn main() {
    let (tx, rx) = mpsc::channel();
    let _tx = Arc::new(tx);
    let mut root = Layout::default();
    root.style.layout.display = taffy::style::Display::Flex;
    root.style.layout.flex_direction = taffy::style::FlexDirection::Row;
    let right_node = Arc::new(RwLock::new(nodes::primitives::Rectangle {
        style: Style {
            layout: TaffyStyle {
                min_size: Size {
                    width: Dimension::Length(50.),
                    height: Dimension::Length(100.)
                },
                ..Default::default()
            },
            cursor: Default::default()
        },
        fill: Paint::color(Color::rgb(0, 0, 255)),
        radius: 0.,
        events: Default::default(),
        parent: None
    }));
    root.children.push(right_node.clone());
    right_node.clone().write().unwrap().events.add_handler(Box::new(move |event| {
        let NodeEvent { target, path, event } = event;
        match event {
            mangui::events::InnerEvent::MouseDown(_) => {
                right_node.write().unwrap().fill = Paint::color(Color::rgb(255, 0, 255));
            },
            mangui::events::InnerEvent::MouseUp(_) => {
                right_node.write().unwrap().fill = Paint::color(Color::rgb(0, 0, 255));
            },
            _ => {}
        }
    }));
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
