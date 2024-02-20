use femtovg::{Color, Paint, Path};
use taffy::layout::Layout;
use crate::{nodes::{Node, NodeChildren, RenderContext, Style}, events::handler::EventHandlerDatabase, WeakNode, SharedNode};

#[derive(Default, Debug)]
pub struct Rectangle {
    pub style: Style,
    pub fill: Paint,
    pub radius: f32,
    pub events: EventHandlerDatabase,
    pub parent: Option<WeakNode>
}

impl Rectangle {
    pub fn new() -> Rectangle {
        Rectangle {
            style: Style::default(),
            fill: Paint::color(Color::rgb(0, 0, 0)),
            radius: 0.,
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
        let mut path = Path::new();
        path.rounded_rect(
            0.,
            0.,
            layout.size.width,
            layout.size.height,
            self.radius
        );
        context.canvas.fill_path(
            &path,
            &self.fill
        );
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