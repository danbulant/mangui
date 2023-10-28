use mangui::SharedNode;

/// A rusalka component
pub trait Component {
    type ComponentAttrs;
    fn new(attr: Self::ComponentAttrs) -> Self;
    fn get(&self) -> &Self::ComponentAttrs;
    fn set(&mut self, attr: Self::ComponentAttrs);
    fn mount(&self, parent: &SharedNode, before: Option<&SharedNode>);
    fn update(&self);
    fn unmount(&self);
}