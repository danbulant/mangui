pub mod layout;
pub mod primitives;

use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use femtovg::{Canvas, Color};
use taffy::layout::Layout;
pub use taffy::style::Style as TaffyStyle;
use taffy::Taffy;
use crate::events::Location;
use crate::{NodeLayoutMap, NodePtr, CurrentRenderer};

type SharedTNode = Arc<RwLock<dyn Node>>;

pub struct RenderContext {
    pub canvas: Canvas<CurrentRenderer>,
    pub node_layout: NodeLayoutMap,
    pub taffy: Taffy,
    pub mouse: NodePtr,
    pub keyboard_focus: NodePtr
}

impl RenderContext {
    /// Fills a rectangle area with the specified color, using the current transform of the canvas.
    /// Rotation WILL break this, this is mostly for simple scaling and translation.
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        let transform = self.canvas.transform();
        let x = transform[0] * x as f32 + transform[2] * y as f32 + transform[4];
        let y = transform[1] * x as f32 + transform[3] * y as f32 + transform[5];
        let width = transform[0] * width as f32 + transform[2] * height as f32;
        let height = transform[1] * width as f32 + transform[3] * height as f32;
        self.canvas.clear_rect(x as u32, y as u32, width as u32, height as u32, color);
    }
}

#[derive(Copy, Clone, Default, Debug)]
#[non_exhaustive]
pub enum Overflow {
    #[default]
    /// Content is not clipped and may be rendered outside the element's box
    Visible,
    /// Clips the content at the border of the element
    Hidden,
    // tbd :)
    // Scroll,
    // Auto
}
#[derive(Clone, Default, Debug)]
pub struct Style {
    pub layout: TaffyStyle,
    pub overflow: Overflow
}

type NodeChildren = Vec<SharedTNode>;

pub trait Node: Debug {
    /// Return style. Usually, you just want self.style.
    fn style(&self) -> &Style;
    /// Returns the children of the node. If the node has no children, return None (empty Vec also works, None is mainly for nodes without children support).
    fn children(&self) -> Option<&NodeChildren>;
    /// Render the node, called before rendering it's children
    /// Canvas considers 0, 0 to be top left corner (for location after layouting happens)
    fn render_pre_children(&self, _context: &mut RenderContext, _layout: Layout) {}
    /// Render the node, called after rendering it's children
    /// Canvas considers 0, 0 to be top left corner (for location after layouting happens)
    fn render_post_children(&self, _context: &mut RenderContext, _layout: Layout) {}
    /// Called when an event happens on the node. This is called after the children have been called.
    /// Beware! Events include a path and target with [Arc<RwLock<Node>>]s, but you already have a write lock for this node!
    /// Remember to check if the node is the same as self, and if it is, use self instead of the node in the path to prevent deadlocks!
    fn on_event(&mut self, _event: &crate::events::NodeEvent) {}


    /// Called when the size of window changes on the root node. Layouts do implement this.
    /// Is an optional function instead of another trait because of missing support for trait upcasting
    // TODO: When rust supports trait upcasting, make this a trait
    fn resize(&mut self, _width: f32, _height: f32) {}
}

pub fn get_element_at(node: &SharedTNode, context: &RenderContext, location: Location) -> Option<Vec<SharedTNode>> {
    let node_borrowed = node.read().unwrap();
    let children = node_borrowed.children();
    let taffy_node = context.node_layout.get(node);
    let taffy_node = match taffy_node {
        Some(taffy_node) => taffy_node,
        None => { return None }
    };
    let layout = *context.taffy.layout(*taffy_node).unwrap();

    if layout.location.x <= location.x && layout.location.y <= location.y && layout.location.x + layout.size.width >= location.x && layout.location.y + layout.size.height >= location.y {
        match children {
            None => {
                Some(vec![node.clone()])
            },
            Some(children) => {
                let mut result = vec![node.clone()];
                for child in children {
                    if let Some(mut path) = get_element_at(child, context, location) {
                        result.append(&mut path);
                    }
                }
                Some(result)
            }
        }
    } else {
        None
    }
}

pub fn layout_recursively(node: &SharedTNode, context: &mut RenderContext) -> taffy::node::Node {
    let taffy_node = context.node_layout.get(node);
    let taffy_node = match taffy_node {
        Some(taffy_node) => taffy_node,
        None => {
            let taffy_node = context.taffy.new_leaf(node.read().unwrap().style().layout.to_owned()).unwrap();
            context.node_layout.insert(node.clone(), taffy_node);
            context.node_layout.get(node).unwrap()
        }
    };

    let taffy_node = taffy_node.to_owned();

    match node.read().unwrap().children() {
        None => {},
        Some(children) => {
            let mut t_children = Vec::with_capacity(children.len());
            for child in children {
                t_children.push(layout_recursively(child, context).to_owned());
            }
            context.taffy.set_children(taffy_node, t_children.as_slice()).unwrap();
        }
    }

    taffy_node
}

pub fn render_recursively(node: &SharedTNode, context: &mut RenderContext) {
    let read_node = node.read().unwrap();
    let styles = read_node.style();
    let taffy_node = context.node_layout.get(node).unwrap();
    let layout = *context.taffy.layout(*taffy_node).unwrap();
    let sself = node.clone();
    context.canvas.save();
    context.canvas.translate(layout.location.x, layout.location.y);
    match styles.overflow {
        Overflow::Visible => {},
        Overflow::Hidden => {
            context.canvas.scissor(
                0.,
                0.,
                layout.size.width,
                layout.size.height,
            );
        }
    }
    drop(read_node);
    sself.read().unwrap().render_pre_children(context, layout);
    if let Some(children) = sself.read().unwrap().children() {
        for child in children {
            render_recursively(child, context);
        }
    }
    sself.read().unwrap().render_post_children(context, layout);
    context.canvas.restore();
}