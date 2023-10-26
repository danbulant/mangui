use std::sync::{RwLock, Arc, mpsc};

use mangui::{self, nodes::{layout::Layout, self, Style, TaffyStyle}, taffy::{self, prelude::Size, style::Dimension}, femtovg::{Paint, Color}, SharedNode, MainEntry};

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(tx);
    let mut root = Layout::default();
    root.style.layout.display = taffy::style::Display::Flex;
    root.style.layout.flex_direction = taffy::style::FlexDirection::Row;
    root.children.push(Arc::new(RwLock::new(nodes::primitives::Rectangle {
        style: Style {
            overflow: nodes::Overflow::Visible,
            layout: TaffyStyle {
                min_size: Size {
                    width: Dimension::Points(100.),
                    height: Dimension::Points(100.)
                },
                ..Default::default()
            }
        },
        fill: Paint::color(Color::rgb(255, 0, 0)),
        radius: 10.,
        events: Default::default()
    })));
    root.children.push(Arc::new(RwLock::new(Layout {
        style: Style {
            overflow: nodes::Overflow::Visible,
            layout: TaffyStyle {
                min_size: Size {
                    width: Dimension::Points(100.),
                    height: Dimension::Points(100.)
                },
                flex_grow: 1.,
                display: taffy::style::Display::Flex,
                flex_direction: taffy::style::FlexDirection::Column,
                ..Default::default()
            }
        },
        children: vec![
            Arc::new(RwLock::new(nodes::primitives::Rectangle {
                style: Style {
                    overflow: nodes::Overflow::Visible,
                    layout: TaffyStyle {
                        min_size: Size {
                            width: Dimension::Points(50.),
                            height: Dimension::Points(50.)
                        },
                        flex_grow: 1.,
                        ..Default::default()
                    }
                },
                fill: Paint::color(Color::rgb(0, 255, 0)),
                radius: 5.,
                events: Default::default()
            })),
            Arc::new(RwLock::new(nodes::primitives::Rectangle {
                style: Style {
                    overflow: nodes::Overflow::Visible,
                    layout: TaffyStyle {
                        min_size: Size {
                            width: Dimension::Points(50.),
                            height: Dimension::Points(50.)
                        },
                        ..Default::default()
                    }
                },
                fill: Paint::color(Color::rgb(0, 255, 255)),
                radius: 5.,
                events: Default::default()
            }))
        ],
        events: Default::default()
    })));
    let right_node = Arc::new(RwLock::new(nodes::primitives::Rectangle {
        style: Style {
            overflow: nodes::Overflow::Visible,
            layout: TaffyStyle {
                min_size: Size {
                    width: Dimension::Points(50.),
                    height: Dimension::Points(100.)
                },
                ..Default::default()
            }
        },
        fill: Paint::color(Color::rgb(0, 0, 255)),
        radius: 0.,
        events: Default::default()
    }));
    root.children.push(right_node.clone());
    right_node.clone().write().unwrap().events.add_handler(Box::new(move |event| {
        match event.event {
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

    mangui::run_event_loop(MainEntry {
        root: groot.clone(),
        render: rx
    });
}
