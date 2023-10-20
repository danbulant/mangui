use femtovg::{Color, Paint, Path, Renderer};
use taffy::layout::Layout;
use crate::nodes::{Node, NodeChildren, RenderContext, Style};

#[derive(Clone, Default, Debug)]
pub struct Rectangle {
    pub style: Style,
    pub color: Color,
    pub radius: f32
}

impl Rectangle {
    pub(crate) fn new() -> Rectangle {
        Rectangle {
            style: Style::default(),
            color: Color::rgb(0, 0, 0),
            radius: 0.
        }
    }
}

impl<T: Renderer> Node<T> for Rectangle {
    fn style(&self) -> &Style {
        &self.style
    }
    fn children(&self) -> Option<&NodeChildren<T>> {
        None
    }
    fn render_pre_children(&self, context: &mut RenderContext<T>, layout: Layout) {
        if self.radius > 0. {
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
                &Paint::color(self.color)
            );
        } else {
            context.fill_rect(
                0,
                0,
                layout.size.width as u32,
                layout.size.height as u32,
                self.color
            );
        }
    }
    // fn render(&self, context: &mut RenderContext<T>, layout: Layout, _render_children: &dyn Fn(&mut RenderContext<T>)) {
    //     if self.radius > 0. {
    //         let mut path = Path::new();
    //         path.rounded_rect(
    //             0.,
    //             0.,
    //             layout.size.width,
    //             layout.size.height,
    //             self.radius
    //         );
    //         context.canvas.fill_path(
    //             &path,
    //             &Paint::color(self.color)
    //         );
    //     } else {
    //         context.canvas.clear_rect(
    //             0,
    //             0,
    //             layout.size.width as u32,
    //             layout.size.height as u32,
    //             self.color
    //         );
    //     }
    // }
}