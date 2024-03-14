pub mod layout;
pub mod empty;
pub mod primitives;
pub mod image;
pub mod text;
pub mod text_render_cache;

use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};
use femtovg::{Canvas, Color, Paint};
use crate::events::Location;
use crate::events::handler::InnerEventHandlerDataset;
use crate::{NodeLayoutMap, NodePtr, CurrentRenderer, SharedNode, WeakNode};

pub use taffy::style::Style as TaffyStyle;
use taffy::{Layout, Overflow, Point, Size, TaffyTree};

pub type CanvasRenderer = Canvas<CurrentRenderer>;

pub struct RenderContext {
    pub canvas: CanvasRenderer,
    pub node_layout: NodeLayoutMap,
    pub taffy: TaffyTree<WeakNode>,
    pub mouse: NodePtr,
    pub keyboard_focus: NodePtr,
    pub scale_factor: f32,
    pub window_size: Size<f32>
}

pub struct MeasureContext<'a> {
    pub canvas: &'a mut CanvasRenderer,
    pub scale_factor: f32
}

impl RenderContext {
    /// Fills a rectangle area with the specified color, using the current transform of the canvas.
    /// Rotation WILL break this, this is mostly for simple scaling and translation.
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        let transform = self.canvas.transform();
        let x = transform[0] * x as f32 + transform[2] * y as f32 + transform[4];
        let y = transform[1] * x + transform[3] * y as f32 + transform[5];
        let width = transform[0] * width as f32 + transform[2] * height as f32;
        let height = transform[1] * width + transform[3] * height as f32;
        self.canvas.clear_rect(x as u32, y as u32, width as u32, height as u32, color);
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub enum Cursor {
    #[default]
    Default
}

#[derive(Clone, Default, Debug)]
/// Transform is handled by UI lib - components shouldn't need to read this.
pub struct Transform {
    /// Translation in x and y direction (in pixels; scaled by parents)
    pub position: Point<f32>,
    /// Scale in x and y direction
    pub scale: Size<f32>,
    /// Rotation in radians
    pub rotation: f32
}

/// Styles for the node. Note that the styles aren't inherited (yet?)
#[derive(Clone, Default, Debug)]
pub struct Style {
    pub layout: TaffyStyle,
    pub cursor: Cursor,
    pub background: Option<Paint>,
    /// defaults to black
    pub text_fill: Option<Paint>,
    /// font size in pixels. Default is 16
    pub font_size: Option<f32>,
    /// multiplier of line height in relation to font size. Default is 1.2
    pub line_height: Option<f32>,
    /// border radius in pixels
    pub border_radius: f32,
    /// Various transformation (position, scale and rotation)
    pub transform: Option<Transform>,
    /// sets scroll offset for x-axis
    /// 0.0 is the default value
    /// you cannot scroll outside the layout - render function will clip the value in that case
    pub scroll_x: f32,
    /// sets scroll offset for y-axis
    /// 0.0 is the default value
    /// you cannot scroll outside the layout - render function will clip the value in that case
    pub scroll_y: f32,
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
///
/// # Function call order
///
/// Read-only functions are called in any order (style, children, event_handlers, measure).
///
/// `resize` is called independently of the rendering process, but not concurrently with it. The render process is single-threaded.
///
/// During rendering, the order is following:
///
/// - parents are changed ([`Node::set_parent`]) to new state according to read-only functions
/// - [`Node::prepare_render`] is called on each node
/// - [`Node::style`] is read
/// - [`Node::measure`] is called on some nodes (depends on taffy); can be called multiple times
/// - nodes are rendered, i.e. on each node, starting from the root node, the following is called:
///    - [`Node::render_pre_children`] is called
///    - children are rendered
///    - [`Node::render_post_children`] is called
pub trait Node: Debug + Send {
    /// Return style.
    ///insert
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
    /// Top left corner is after margin and similar, but before padding and border.
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
    /// Note that this doesn't set the parent of the child. You need to do that manually.
    ///
    /// Implementors can check [`Node::has_child`] to check if the child already exists. Default implementation throws [`ChildAddError::ChildrenNotSupported`].
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

    /// Called on each redraw. Use this to prepare for rendering. Called before any layouting or rendering happens.
    /// Order between nodes is not guaranteed.
    fn prepare_render(&mut self, _context: &mut RenderContext) {}

    /// Called before rendering the node to measure it's size.
    /// The calling of this method is managed by taffy, and as such:
    /// - It may be called multiple times (with same or different arguments) during the same render pass
    /// - It may not be called at all
    /// - order between nodes is not guaranteed
    ///
    /// If you need to change self during layouting, use [`Node::prepare_render`] to do so.
    /// You're getting &mut self here to support things like cosmic text that require changing text data to measure it.
    fn measure(&mut self, _context: &mut MeasureContext, _known_dimensions: Size<Option<f32>>, _available_space: Size<taffy::AvailableSpace>) -> Size<f32> {
        Size::ZERO
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

pub trait ToShared {
    fn to_shared(self) -> SharedNode;
    fn to_arcmutex(self) -> Arc<Mutex<Self>> where Self: Sized {
        Arc::new(Mutex::new(self))
    }
}

impl<T: Node + 'static> ToShared for T {
    fn to_shared(self) -> SharedNode {
        Arc::new(Mutex::new(self))
    }
}

/// Runs event handlers for the given path
/// The target element should be the last one in path (event handlers are ran in reverse order)
pub(crate) fn run_event_handlers(path: Vec<SharedNode>, event: crate::events::NodeEvent) {
    for node in path.iter().rev() {
        let node = node.lock().unwrap();
        if let Some(handlers) = node.event_handlers() {
            drop(node);
            for handler in handlers.lock().unwrap().values_mut() {
                handler.lock().unwrap()(&event);
            }
        }
    }
}

pub(crate) fn run_single_event_handlers(node: SharedNode, event: crate::events::NodeEvent) {
    let node = node.lock().unwrap();
    if let Some(handlers) = node.event_handlers() {
        drop(node);
        for handler in handlers.lock().unwrap().values_mut() {
            handler.lock().unwrap()(&event);
        }
    }
}

/// Attempts to get path to the element at the target location. Assumes elements are always inside their parents.
pub(crate) fn get_element_at(node: &SharedNode, context: &RenderContext, location: Location) -> Option<Vec<SharedNode>> {
    let node_borrowed = node.lock().unwrap();
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

pub(crate) fn update_taffynode_children(node: &SharedNode, context: &mut RenderContext) -> taffy::tree::NodeId {
    let taffy_node = context.node_layout.get(node);
    let taffy_node = match taffy_node {
        Some(taffy_node) => taffy_node,
        None => {
            let taffy_node = context.taffy.new_leaf_with_context(
                node.lock().unwrap().style().layout.to_owned(),
                Arc::downgrade(node)
            ).unwrap();
            context.node_layout.insert(node.clone(), taffy_node);
            context.node_layout.get(node).unwrap()
        }
    };

    let taffy_node = taffy_node.to_owned();

    match node.lock().unwrap().children() {
        None => {},
        Some(children) => {
            let mut t_children = Vec::with_capacity(children.len());
            for child in children {
                t_children.push(update_taffynode_children(child, context).to_owned());
                child.lock().unwrap().set_parent(Some(Arc::downgrade(node)));
            }
            context.taffy.set_children(taffy_node, t_children.as_slice()).unwrap();
        }
    }

    taffy_node
}

pub(crate) fn render_recursively(node: &SharedNode, context: &mut RenderContext) {
    let read_node = node.lock().unwrap();
    let styles = read_node.style();
    let taffy_node = context.node_layout.get(node).unwrap();
    let layout = *context.taffy.layout(*taffy_node).unwrap();
    let sself = node.clone();
    context.canvas.save();
    let offset = styles.transform.as_ref().map(|t| (t.position.x, t.position.y)).unwrap_or((0., 0.));
    let scroll_offset = (styles.scroll_x, styles.scroll_y);
    let content_size = layout.content_size;
    let visible_size = layout.size;
    let scroll_offset = (scroll_offset.0.min(content_size.width - visible_size.width).max(0.), scroll_offset.1.min(content_size.height - visible_size.height).max(0.));
    context.canvas.translate(
        layout.location.x + offset.0 - scroll_offset.0,
        layout.location.y + offset.1 - scroll_offset.1
    );
    if let Some(transform) = &styles.transform {
        context.canvas.scale(transform.scale.width, transform.scale.height);
        context.canvas.rotate(transform.rotation);
    }
    let clip_width = matches!(styles.layout.overflow.x, Overflow::Hidden | Overflow::Clip | Overflow::Scroll);
    let clip_height = matches!(styles.layout.overflow.y, Overflow::Hidden | Overflow::Clip | Overflow::Scroll);
    if clip_width || clip_height {
        context.canvas.scissor(
            0.,
            0.,
            if clip_width { layout.size.width } else { f32::INFINITY },
            if clip_height { layout.size.height } else { f32::INFINITY },
        );
    }
    drop(read_node);
    let mut locked = sself.lock().unwrap();
    locked.render_pre_children(context, layout);
    if let Some(children) = locked.children() {
        for child in children {
            render_recursively(child, context);
        }
    }
    locked.render_post_children(context, layout);
    context.canvas.restore();
}

pub(crate) fn prepare_render_recursively(node: &SharedNode, context: &mut RenderContext) {
    let mut write_node = node.lock().unwrap();
    write_node.prepare_render(context);
    if let Some(children) = write_node.children() {
        for child in children {
            prepare_render_recursively(child, context);
        }
    }
}