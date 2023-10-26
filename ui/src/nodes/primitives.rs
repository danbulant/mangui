use femtovg::{Color, Paint, Path};
use taffy::layout::Layout;
use crate::{nodes::{Node, NodeChildren, RenderContext, Style}, events::handler::EventHandlerDatabase};

#[derive(Default, Debug)]
pub struct Rectangle {
    pub style: Style,
    pub fill: Paint,
    pub radius: f32,
    pub events: EventHandlerDatabase
}

impl Rectangle {
    pub fn new() -> Rectangle {
        Rectangle {
            style: Style::default(),
            fill: Paint::color(Color::rgb(0, 0, 0)),
            radius: 0.,
            events: EventHandlerDatabase::default()
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
    fn render_pre_children(&self, context: &mut RenderContext, layout: Layout) {
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
}