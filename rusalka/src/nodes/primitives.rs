
// DemoComponent

use std::sync::{Arc, RwLock};
use mangui::{SharedNode, nodes::{primitives, Style}, taffy::prelude::Size, femtovg::{Paint, Color}};

use crate::{component::Component, WeakSharedComponent};

use super::{insert, detach};

pub struct Rectangle {
    node: Arc<RwLock<primitives::Rectangle>>,
    attrs: RectangleAttributes,
    selfref: WeakSharedComponent<Self>
}

#[derive(Default)]
pub struct RectangleAttributes {
    pub radius: f32
}

#[derive(Default)]
pub struct PartialRectangleAttributes {
    pub radius: Option<f32>
}

impl From<RectangleAttributes> for PartialRectangleAttributes {
    fn from(attrs: RectangleAttributes) ->
    Self {
        Self {
            radius: Some(attrs.radius)
        }
    }
}

impl Component for Rectangle {
    type ComponentAttrs = RectangleAttributes;
    type PartialComponentAttrs = PartialRectangleAttributes;
    const UPDATE_LENGTH : usize = 1;
    fn new(attrs: Self::ComponentAttrs, selfref: WeakSharedComponent<Self>) -> Self {
        Self {
            node: Arc::new(RwLock::new(primitives::Rectangle {
                style: Style {
                    layout: mangui::nodes::TaffyStyle {
                        min_size: Size {
                            width: mangui::taffy::style::Dimension::Length(50.),
                            height: mangui::taffy::style::Dimension::Length(100.)
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                fill: Paint::color(Color::rgb(0, 0, 255)),
                radius: attrs.radius,
                ..Default::default()
            })),
            attrs,
            selfref
        }
    }

    fn set(&mut self, attrs: Self::PartialComponentAttrs) {
        let mut to_update = [0; Self::UPDATE_LENGTH];
        if let Some(radius) = attrs.radius {
            self.attrs.radius = radius;
            to_update[0] |= 1;
        }
        if to_update.into_iter().reduce(|a,b| a+b).unwrap() != 0 {
            self.update(&to_update);
        }
    }
    fn get(&self) -> &Self::ComponentAttrs { &self.attrs }

    fn mount(&self, parent: &SharedNode, before: Option<&SharedNode>) {
        insert(parent, &{self.node.clone()}, before);
    }

    fn update(&self, bitmap: &[u32]) {
        self.check_update(bitmap);

        if bitmap[0] & 1 != 0 {
            self.node.write().unwrap().radius = self.attrs.radius;
        }

        if bitmap[0] & 1 != 0 {}
    }

    fn unmount(&self) {
        detach(&{self.node.clone()});
    }
}