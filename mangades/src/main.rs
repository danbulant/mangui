use std::sync::{RwLock, Arc};

use mangui::{self, nodes::{layout::Layout, self, Style, TaffyStyle}, taffy::{self, prelude::Size, style::Dimension}, femtovg::{Paint, Color}, SharedNode};

fn main() {
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
        radius: 10.
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
                radius: 5.
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
                radius: 5.
            }))
        ]

    })));
    root.children.push(Arc::new(RwLock::new(nodes::primitives::Rectangle {
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
        radius: 0.
    })));
    let groot: SharedNode = Arc::new(RwLock::new(root));

    mangui::run_event_loop(groot);
}
