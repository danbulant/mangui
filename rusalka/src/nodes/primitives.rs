
// DemoComponent

use std::sync::{Arc, RwLock};
use mangui::{SharedNode, nodes::{primitives, Style}, taffy::prelude::Size, femtovg::{Paint, Color}};

use crate::component::Component;

use super::{insert, detach};

pub struct Rectangle {
    node: SharedNode,
    attrs: RectangleAttributes
}

#[derive(Default)]
pub struct RectangleAttributes {}

impl Component for Rectangle {
    type ComponentAttrs = RectangleAttributes;
    fn new(attrs: Self::ComponentAttrs) -> Self {
        Self {
            node: Arc::new(RwLock::new(primitives::Rectangle {
                style: Style {
                    layout: mangui::nodes::TaffyStyle {
                        min_size: Size {
                            width: mangui::taffy::style::Dimension::Points(50.),
                            height: mangui::taffy::style::Dimension::Points(100.)
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                fill: Paint::color(Color::rgb(0, 0, 255)),
                radius: 5.,
                ..Default::default()
            })),
            attrs
        }
    }

    fn set(&mut self, attrs: Self::ComponentAttrs) { self.attrs = attrs; }
    fn get(&self) -> &Self::ComponentAttrs { &self.attrs }
    fn mount(&self, parent: &SharedNode, before: Option<&SharedNode>) {
        insert(parent, &self.node, before);
    }

    fn update(&self) {}

    fn unmount(&self) {
        detach(&self.node);
    }
}