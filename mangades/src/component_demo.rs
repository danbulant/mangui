use std::sync::{Arc, Mutex, RwLock};
use mangui::{SharedNode, nodes::{layout::Layout, Style}, taffy::prelude::Size};
use rusalka::{component::Component, nodes::{primitives::{Rectangle, RectangleAttributes}, insert, detach}, SharedComponent};

pub struct DemoComponent {
    rect: SharedComponent<Rectangle>,
    attrs: DemoComponentAttributes,
    layout: Arc<RwLock<Layout>>
}

#[derive(Default)]
pub struct DemoComponentAttributes {}
#[derive(Default)]
pub struct PartialDemoComponentAttributes {}

impl From<DemoComponentAttributes> for PartialDemoComponentAttributes {
    fn from(_attrs: DemoComponentAttributes) -> Self {
        Self {}
    }
}


impl Component for DemoComponent {
    type ComponentAttrs = DemoComponentAttributes;
    type PartialComponentAttrs = PartialDemoComponentAttributes;
    fn new(attrs: Self::ComponentAttrs) -> Self {
        Self {
            rect: Arc::new(Mutex::new(Rectangle::new(RectangleAttributes { ..Default::default() }))),
            layout: Arc::new(RwLock::new(Layout {
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
                ..Default::default()
            })),
            attrs,
        }
    }

    fn set(&mut self, _attrs: Self::PartialComponentAttrs) { }
    fn get(&self) -> &Self::ComponentAttrs { &self.attrs }
    fn mount(&self, parent: &SharedNode, before: Option<&SharedNode>) {
        insert(parent, &{self.layout.clone()}, before);
        self.rect.lock().unwrap().mount(&{self.layout.clone()}, None);
    }

    fn unmount(&self) {
        self.rect.lock().unwrap().unmount();
        detach(&{self.layout.clone()});
    }

    fn update(&self, _bitmap: &[u32]) {}
}
