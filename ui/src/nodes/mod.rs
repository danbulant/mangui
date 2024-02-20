pub mod layout;
pub mod primitives;
pub mod image;

use std::fmt::Debug;
use std::sync::Arc;
use femtovg::{Canvas, Color};
use taffy::layout::Layout;
use taffy::Taffy;
use crate::events::Location;
use crate::events::handler::InnerEventHandlerDataset;
use crate::{NodeLayoutMap, NodePtr, CurrentRenderer, SharedNode, WeakNode};

pub use taffy::style::Style as TaffyStyle;

pub type CanvasRenderer = Canvas<CurrentRenderer>;

pub struct RenderContext {
    pub canvas: CanvasRenderer,
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
#[derive(Copy, Clone, Default, Debug)]
pub enum Cursor {
    #[default]
    Default
}
#[derive(Clone, Default, Debug)]
pub struct Style {
    pub layout: TaffyStyle,
    pub overflow: Overflow,
    pub cursor: Cursor
}

type NodeChildren = Vec<SharedNode>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChildAddError {
    ChildrenNotSupported,
    /// The index is out of bounds (cannot be thrown from add_child as long as the [`Node`] implementation is correct)
    OutOfBounds,
    /// Generic error text (subject to change for better error handling)
    GenericError(String)
}

/// A node in the UI tree. This is the main trait for nodes.
///
/// Implementation for style, children (readable list) and one of [`Node::render_pre_children`] or [`Node::render_post_children`] is required.
///
/// # Root node
/// If you are implementing a root node, you will need to implement [`Node::resize`] to resize your node correctly based on window size.
/// Simply updating the style is sufficient
///
/// # Children
///
/// If you don't need to support children, you can return None from [`Node::children`].
///
/// If you support children in a read only matter, you can return a [`NodeChildren`] from [`Node::children`].
///
/// If you also need to be able to update children, implement [`Node::add_child_at`] in addition to [`Node::children`].
/// Other child adding methods are implemented using [`Node::add_child_at`] and/or [`Node::children`] automatically.
/// Implement [`Node::remove_child`] to support removing children.
///
/// # Events
///
/// If you need to handle events, implement [`Node::event_handlers`].
pub trait Node: Debug {
    /// Return style.
    ///
    /// If you're using [`Style`] in your struct directly, your implementation can be as simple as:
    /// ```rust
    /// fn style(&self) -> &Style { &self.style }
    /// ```
    fn style(&self) -> &Style;
    /// Returns the children of the node. If the node has no children, return None (empty Vec also works, None is mainly for nodes without children support).
    ///
    /// If you're using [`NodeChildren`] in your struct directly, your implementation can be as simple as:
    /// ```rust
    /// fn children(&self) -> Option<&NodeChildren> { Some(&self.children) }
    /// ```
    fn children(&self) -> Option<&NodeChildren>;

    /// Render the node, called before rendering it's children
    /// Canvas considers 0, 0 to be top left corner (for location after layouting happens)
    fn render_pre_children(&mut self, _context: &mut RenderContext, _layout: Layout) {}
    /// Render the node, called after rendering it's children
    /// Canvas considers 0, 0 to be top left corner (for location after layouting happens)
    fn render_post_children(&mut self, _context: &mut RenderContext, _layout: Layout) {}

    /// Sets the parent node.
    /// May be called multiple times with the same value.
    ///
    /// Implementors: SAVE A WEAK REFERENCE!
    /// Without a weak reference, the tree will have a loop and will never be dropped.
    ///
    /// Example implementation:
    /// ```rust
    /// fn set_parent(&mut self, parent: Option<WeakNode>) {
    ///    self.parent = parent;
    /// }
    fn set_parent(&mut self, parent: Option<WeakNode>);
    /// Returns the parent node.
    ///
    /// Example implementation:
    /// ```rust
    /// fn parent(&self) -> Option<SharedNode> {
    ///     match &self.parent {
    ///         Some(parent) => parent.upgrade(),
    ///         None => None
    ///     }
    /// }
    /// ```
    fn parent(&self) -> Option<SharedNode>;

    /// Add a child to the node. If the node does not support children, returns error ChildrenNotSupported.
    /// Adding the same child multiple times or to multiple parents is not supported and will result in undefined behavior.
    /// Arc<RwLock<Node>> does **NOT** mean that it's safe to add the same node multiple times.
    ///
    /// Default implementation uses [`Node::add_child_at`] and adds the child at the end.
    /// Assumes that if [`Node::children`] returns None, the node does not support children, and the if it returns Some, the node does support children.
    /// Also assumes that [`Node::children`] returns correct values - the length matches the actual number of children and so on.
    fn add_child(&mut self, _child: SharedNode) -> Result<(), ChildAddError> {
        if let Some(children) = self.children() {
            self.add_child_at(_child, children.len())
        } else {
            Err(ChildAddError::ChildrenNotSupported)
        }
    }

    /// Add a child to the node at the given index. If the node does not support children, returns error ChildrenNotSupported.
    /// Adding the same child multiple times or to multiple parents is not supported and will result in undefined behavior.
    /// Arc<RwLock<Node>> does **NOT** mean that it's safe to add the same node multiple times.
    ///
    /// Implementors can check [`Node::has_child`] to check if the child already exists. Default implementation thros [`ChildAddError::ChildrenNotSupported`].
    /// Adding a child that already exists should move that child to the new position.
    fn add_child_at(&mut self, _child: SharedNode, _index: usize) -> Result<(), ChildAddError> { Err(ChildAddError::ChildrenNotSupported) }

    /// Adds a child after the given child. If the node does not support children, returns error ChildrenNotSupported.
    /// Adding the same child multiple times or to multiple parents is not supported and will result in undefined behavior.
    /// Arc<RwLock<Node>> does **NOT** mean that it's safe to add the same node multiple times.
    fn add_child_after(&mut self, child: SharedNode, after: &SharedNode) -> Result<(), ChildAddError> {
        if let Some(_) = self.children() {
            if let Some(index) = self.has_child(after) {
                self.add_child_at(child, index + 1)
            } else {
                Err(ChildAddError::GenericError("Child not found".to_owned()))
            }
        } else {
            Err(ChildAddError::ChildrenNotSupported)
        }
    }

    /// Adds a child before the given child. If the node does not support children, returns error ChildrenNotSupported.
    /// Adding the same child multiple times or to multiple parents is not supported and will result in undefined behavior.
    /// Arc<RwLock<Node>> does **NOT** mean that it's safe to add the same node multiple times.
    fn add_child_before(&mut self, child: SharedNode, before: &SharedNode) -> Result<(), ChildAddError> {
        if let Some(_) = self.children() {
            if let Some(index) = self.has_child(before) {
                self.add_child_at(child, index)
            } else {
                Err(ChildAddError::GenericError("Child not found".to_owned()))
            }
        } else {
            Err(ChildAddError::ChildrenNotSupported)
        }
    }

    /// Removes a child from the node. If the node does not support children, returns error ChildrenNotSupported.
    /// Removing non-existent child is a no-op.
    fn remove_child(&mut self, _child: &SharedNode) -> Result<(), ChildAddError> { Err(ChildAddError::ChildrenNotSupported) }

    /// Returns the event handlers of the node. If the node has no event handlers, return None.
    /// Use [`EventHandlerDatabase`] to manage this, and return it's handlers property.
    /// Example implementation:
    /// ```rust
    /// fn event_handlers(&self) -> Option<InnerEventHandlerDataset> {
    ///     Some(self.events.handlers.clone())
    /// }
    /// ```
    ///
    /// Example struct:
    /// ```rust
    /// struct MyNode {
    ///    events: EventHandlerDatabase
    /// }
    /// ```
    fn event_handlers(&self) -> Option<InnerEventHandlerDataset> {
        None
    }

    /// Returns true if the node has the given child
    /// Returns false if there are no children (or if the node does not support children)
    fn has_child(&self, child: &SharedNode) -> Option<usize> {
        let mut i = 0;
        if let Some(children) = self.children() {
            for c in children {
                if Arc::ptr_eq(c, child) {
                    return Some(i);
                }
                i += 1;
            }
        }
        None
    }

    /// Called when the size of window changes on the root node. Layouts do implement this.
    /// Is an optional function instead of another trait because of missing support for trait upcasting
    // TODO: When rust supports trait upcasting, make this a trait
    fn resize(&mut self, _width: f32, _height: f32) {}
}

/// Runs event handlers for the given path
/// The target element should be the last one in path (event handlers are ran in reverse order)
pub(crate) fn run_event_handlers(path: Vec<SharedNode>, event: crate::events::NodeEvent) {
    for node in path.iter().rev() {
        let node = node.read().unwrap();
        if let Some(handlers) = node.event_handlers() {
            drop(node);
            for handler in handlers.lock().unwrap().values_mut() {
                handler.lock().unwrap()(&event);
            }
        }
    }
}

pub(crate) fn run_single_event_handlers(node: SharedNode, event: crate::events::NodeEvent) {
    let node = node.read().unwrap();
    if let Some(handlers) = node.event_handlers() {
        drop(node);
        for handler in handlers.lock().unwrap().values_mut() {
            handler.lock().unwrap()(&event);
        }
    }
}

/// Attempts to get path to the element at the target location. Assumes elements are always inside their parents.
pub(crate) fn get_element_at(node: &SharedNode, context: &RenderContext, location: Location) -> Option<Vec<SharedNode>> {
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

pub(crate) fn layout_recursively(node: &SharedNode, context: &mut RenderContext) -> taffy::node::Node {
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
                child.write().unwrap().set_parent(Some(Arc::downgrade(node)));
            }
            context.taffy.set_children(taffy_node, t_children.as_slice()).unwrap();
        }
    }

    taffy_node
}

pub(crate) fn render_recursively(node: &SharedNode, context: &mut RenderContext) {
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
    sself.write().unwrap().render_pre_children(context, layout);
    if let Some(children) = sself.read().unwrap().children() {
        for child in children {
            render_recursively(child, context);
        }
    }
    sself.write().unwrap().render_post_children(context, layout);
    context.canvas.restore();
}