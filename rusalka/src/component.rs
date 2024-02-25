use mangui::{SharedNode, nodes::Node};

use crate::WeakSharedComponent;

/// A rusalka component
pub trait Component {
    type ComponentAttrs;
    type ReactiveComponentAttrs: From<Self::ComponentAttrs>;
    type PartialComponentAttrs: Default + From<Self::ComponentAttrs>;
    const UPDATE_LENGTH : usize = 0;
    fn new(attr: Self::ComponentAttrs, selfref: WeakSharedComponent<Self>) -> Self;
    fn get(&self) -> &Self::ReactiveComponentAttrs;
    fn set(&mut self, attr: Self::PartialComponentAttrs);
    fn mount(&self, parent: &SharedNode, before: Option<&SharedNode>);
    fn unmount(&self);
}

pub struct Slot {
    pub mount: Box<dyn FnMut(&SharedNode, Option<&SharedNode>)>,
    pub unmount: Box<dyn FnMut()>,
}