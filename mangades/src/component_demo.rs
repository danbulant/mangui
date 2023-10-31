use std::sync::{Arc, Mutex, RwLock};
use mangui::{SharedNode, nodes::{layout::Layout, Style}, taffy::prelude::Size};
use rusalka::{component::Component, nodes::{primitives::{Rectangle, RectangleAttributes}, insert, detach}, SharedComponent, WeakSharedComponent, invalidator::Invalidator};

pub struct DemoComponent {
    rect: SharedComponent<Rectangle>,
    attrs: DemoComponentAttributes,
    layout: Arc<RwLock<Layout>>,
    selfref: WeakSharedComponent<Self>,
    test: Arc<Mutex<Invalidator<bool>>>
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
    fn new(attrs: Self::ComponentAttrs, selfref: WeakSharedComponent<Self>) -> Self {
        let test = Arc::new(Mutex::new(Invalidator::new(false)));
        let self_ = Self {
            rect: Arc::new_cyclic(|selfref| Mutex::new(Rectangle::new(RectangleAttributes { ..Default::default() }, selfref.clone()))),
            layout: Arc::new(RwLock::new(Layout {
                style: Style {
                    layout: mangui::nodes::TaffyStyle {
                        min_size: Size {
                            width: mangui::taffy::style::Dimension::Points( if **test.lock().unwrap() { 50. } else { 100. }),
                            height: mangui::taffy::style::Dimension::Points(100.)
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            })),
            attrs,
            selfref,
            test
        };
        let selfref = self_.selfref.clone();
        self_.layout.write().unwrap().events.add_handler(Box::new(move |event| {
            let selfref = selfref.upgrade().unwrap();
            let self_ = selfref.lock().unwrap();
            let test = &self_.test;
            let attrs = &self_.attrs;
            match event.event {
                mangui::events::InnerEvent::MouseDown(_) => {
                    **test.lock().unwrap() = true;
                },
                mangui::events::InnerEvent::MouseUp(_) => {
                    **test.lock().unwrap() = false;
                },
                _ => {}
            }
        }));
        self_
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
