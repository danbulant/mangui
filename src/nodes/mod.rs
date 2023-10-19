pub mod layout;

use std::sync::Arc;
use femtovg::{Canvas, Color, Renderer};
use femtovg::renderer::OpenGl;
use taffy::geometry::Size;
use taffy::layout::Layout;
use taffy::prelude::Dimension;
use taffy::style::Style as TaffyStyle;
use taffy::Taffy;
use crate::{GNode, TaffyMap};

pub struct RenderContext<T: Renderer> {
    pub canvas: Canvas<T>,
    pub taffy_map: TaffyMap,
    pub taffy: Taffy
}

#[derive(Copy, Clone, Default)]
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

pub struct Style {
    pub(crate) layout: TaffyStyle,
    pub overflow: Overflow
}

type NodeChildren<T> = Vec<Arc<dyn Node<T>>>;

pub trait Node<T: Renderer> {
    /// Return style. Usually, you just want self.style.
    fn style(&self) -> &Style;
    /// Returns the children of the node. If the node has no children, return None (empty Vec also works, None is mainly for nodes without children support).
    fn children(&self) -> Option<&NodeChildren<T>>;
    /// Render the node and its children. render_children gets ['children'] and calls this function there as well. When drawing, the canvas is translated to the node's location.
    /// Canvas considers 0, 0 to be top left corner (for location after layouting happens)
    fn render(&self, context: &mut RenderContext<T>, layout: Layout, render_children: &dyn Fn(&mut RenderContext<T>));
}

pub fn layout_recursively(node: &Arc<GNode>, context: &mut RenderContext<OpenGl>) -> taffy::node::Node {
    let taffy_node = context.taffy_map.get(node);
    let taffy_node = match taffy_node {
        Some(taffy_node) => taffy_node,
        None => {
            let taffy_node = context.taffy.new_leaf(node.style().layout.to_owned()).unwrap();
            context.taffy_map.insert(node.clone(), taffy_node);
            context.taffy_map.get(node).unwrap()
        }
    };

    let taffy_node = taffy_node.to_owned();

    match node.children() {
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

pub fn render_recursively(node: &Arc<GNode>, context: &mut RenderContext<OpenGl>) {
    let styles = node.style();
    let taffy_node = context.taffy_map.get(node).unwrap();
    let layout = *context.taffy.layout(*taffy_node).unwrap();
    let sself = node.clone();
    context.canvas.save();
    context.canvas.translate(layout.location.x, layout.location.y);
    match styles.overflow {
        Overflow::Visible => {},
        Overflow::Hidden => {
            context.canvas.scissor(
                layout.location.x,
                layout.location.y,
                layout.size.width,
                layout.size.height,
            );
        }
    }
    sself.render(context, layout, & (|context| {
        if let Some(children) = sself.children() {
            for child in children {
                render_recursively(child, context);
            }
        }
    }));
    context.canvas.restore();
}

pub struct RedBoxDemo {
    style: Style
}

impl RedBoxDemo {
    pub(crate) fn new() -> RedBoxDemo {
        RedBoxDemo {
            style: Style {
                layout: TaffyStyle {
                    size: Size {
                        width: Dimension::Points(30.),
                        height: Dimension::Points(30.)
                    },
                    ..TaffyStyle::default()
                },
                overflow: Overflow::Visible
            }
        }
    }
}

impl<T: Renderer> Node<T> for RedBoxDemo {
    fn style(&self) -> &Style {
        &self.style
    }
    fn children(&self) -> Option<&NodeChildren<T>> {
        None
    }
    fn render(&self, context: &mut RenderContext<T>, _layout: Layout, _render_children: &dyn Fn(&mut RenderContext<T>)) {
        context.canvas.clear_rect(
            0,
            0,
            30,
            30,
            Color::rgbf(1., 0., 0.),
        );
    }
}