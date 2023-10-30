use mangui::{SharedNode, nodes::Node};

/// A rusalka component
pub trait Component {
    type ComponentAttrs: Default;
    type PartialComponentAttrs: Default + From<Self::ComponentAttrs>;
    const UPDATE_LENGTH : usize = 0;
    fn new(attr: Self::ComponentAttrs) -> Self;
    fn get(&self) -> &Self::ComponentAttrs;
    fn set(&mut self, attr: Self::PartialComponentAttrs);
    fn mount(&self, parent: &SharedNode, before: Option<&SharedNode>);
    fn update(&self, bitmap: &[u32]);
    fn unmount(&self);

    fn check_update(&self, bitmap: &[u32]) -> () {
        if bitmap.len() != Self::UPDATE_LENGTH {
            panic!("Bitmap length does not match update length");
        }
    }
}
