use femtovg::{Color, Paint, Path, Renderer};
use taffy::layout::Layout;
use crate::nodes::{Node, NodeChildren, RenderContext, Style};

#[derive(Clone, Default, Debug)]
pub struct Rectangle {
    pub style: Style,
    pub fill: Paint,
    pub radius: f32
}

impl Rectangle {
    pub fn new() -> Rectangle {
        Rectangle {
            style: Style::default(),
            fill: Paint::color(Color::rgb(0, 0, 0)),
            radius: 0.
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

    fn on_event(&mut self, event: &crate::events::NodeEvent) {}
}