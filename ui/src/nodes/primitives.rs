use femtovg::{Color, Paint, Path};
use taffy::{Layout, Size};
use crate::{nodes::{Node, NodeChildren, RenderContext, Style}, events::handler::EventHandlerDatabase, WeakNode, SharedNode};
use crate::nodes::CanvasRenderer;

#[derive(Default, Debug)]
pub struct Rectangle {
    pub style: Style,
    pub events: EventHandlerDatabase,
    pub parent: Option<WeakNode>
}

impl Rectangle {
    pub fn new() -> Rectangle {
        Rectangle {
            style: Style::default(),
            events: EventHandlerDatabase::default(),
            parent: None
        }
    }
}

impl Node for Rectangle {
    fn style(&self) -> &Style {
        &self.style
    }
    fn children(&self) -> Option<&NodeChildren> {
        None
    }
    fn render_pre_children(&mut self, context: &mut RenderContext, layout: Layout) {
        draw_rect(layout.size, self.style.background.as_ref().unwrap_or(&Paint::color(Color::black())), self.style.border_radius, &mut context.canvas);
    }
    fn event_handlers(&self) -> Option<crate::events::handler::InnerEventHandlerDataset> {
        Some(self.events.handlers.clone())
    }
    fn set_parent(&mut self, parent: Option<WeakNode>) {
        self.parent = parent;
    }
    fn parent(&self) -> Option<SharedNode> {
        match &self.parent {
            Some(parent) => parent.upgrade(),
            None => None
        }
    }
}

pub fn draw_rect(size: Size<f32>, fill: &Paint, radius: f32, canvas: &mut CanvasRenderer) {
    let mut path = Path::new();
    path.rounded_rect(
        0.,
        0.,
        size.width,
        size.height,
        radius
    );
    canvas.fill_path(
        &path,
        fill
    );
}