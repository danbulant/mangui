pub mod layout;
pub mod primitives;

use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use femtovg::{Canvas, Renderer, Color};
use taffy::layout::Layout;
pub use taffy::style::Style as TaffyStyle;
use taffy::Taffy;
use crate::{NodeLayoutMap, LayoutNodeMap, TNodePtr};

type SharedTNode<T> = Arc<RwLock<dyn Node<T>>>;

pub struct RenderContext<T: Renderer> {
    pub canvas: Canvas<T>,
    pub node_layout: NodeLayoutMap<T>,
    pub layout_node: LayoutNodeMap<T>,
    pub taffy: Taffy,
    pub mouse: TNodePtr<T>,
    pub keyboard_focus: TNodePtr<T>
}

impl<T: Renderer> RenderContext<T> {
    /// Fills a rectangle area with the specified color, using the current transform of the canvas.
    /// Rotation WILL break this, this is mostly for simple scaling and translation.
    fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
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

type NodeChildren<T> = Vec<SharedTNode<T>>;

pub trait Node<T: Renderer>: Debug {
    /// Return style. Usually, you just want self.style.
    fn style(&self) -> &Style;
    /// Returns the children of the node. If the node has no children, return None (empty Vec also works, None is mainly for nodes without children support).
    fn children(&self) -> Option<&NodeChildren<T>>;
    /// Render the node, called before rendering it's children
    /// Canvas considers 0, 0 to be top left corner (for location after layouting happens)
    fn render_pre_children(&self, _context: &mut RenderContext<T>, _layout: Layout) {}
    /// Render the node, called after rendering it's children
    /// Canvas considers 0, 0 to be top left corner (for location after layouting happens)
    fn render_post_children(&self, _context: &mut RenderContext<T>, _layout: Layout) {}
    /// Called when the size of window changes on the root node. Layouts do implement this.
    /// Is an optional function instead of another trait because of missing support for trait upcasting
    // TODO: When rust supports trait upcasting, make this a trait
    fn resize(&mut self, width: f32, height: f32) {}
}

pub fn layout_recursively<T: Renderer>(node: &SharedTNode<T>, context: &mut RenderContext<T>) -> taffy::node::Node {
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

pub fn render_recursively<T: Renderer>(node: &SharedTNode<T>, context: &mut RenderContext<T>) {
    let read_node = node.read().unwrap();
    let styles = read_node.style();
    let taffy_node = context.node_layout.get(node).unwrap();
    let layout = *context.taffy.layout(*taffy_node).unwrap();
    let sself = node.clone();
    context.canvas.save();
    context.canvas.translate(layout.location.x, layout.location.y);
    // dbg!(node, layout);
    // dbg!(styles, layout);
    // dbg!(context.canvas.transform());
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
    // sself.render(context, layout, & (|context| {
    //     if let Some(children) = sself.children() {
    //         for child in children {
    //             render_recursively(child, context);
    //         }
    //     }
    // }));
    sself.read().unwrap().render_pre_children(context, layout);
    if let Some(children) = sself.read().unwrap().children() {
        for child in children {
            render_recursively(child, context);
        }
    }
    sself.read().unwrap().render_post_children(context, layout);
    context.canvas.restore();
}