use std::sync::{Arc, Mutex};
use mangui::SharedNode;
use rusalka::{component::Component, nodes::primitives::{Rectangle, RectangleAttributes}, SharedComponent};

pub struct DemoComponent {
    rect: SharedComponent<Rectangle>,
    attrs: DemoComponentAttributes,
}

pub struct DemoComponentAttributes {}

impl Component for DemoComponent {
    type ComponentAttrs = DemoComponentAttributes;
    fn new(attrs: Self::ComponentAttrs) -> Self {
        Self {
            rect: Arc::new(Mutex::new(Rectangle::new(RectangleAttributes {}))),
            attrs,
        }
    }

    fn set(&mut self, attrs: Self::ComponentAttrs) { self.attrs = attrs; }
    fn get(&self) -> &Self::ComponentAttrs { &self.attrs }
    fn mount(&self, parent: &SharedNode, before: Option<&SharedNode>) {
        self.rect.lock().unwrap().mount(parent, before);
    }

    fn unmount(&self) {
        self.rect.lock().unwrap().unmount();
    }

    fn update(&self) {}
}
