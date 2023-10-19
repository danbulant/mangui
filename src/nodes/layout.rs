use femtovg::Renderer;
use crate::nodes::{Node, NodeChildren, Overflow, RenderContext, Style};
use taffy::style::{Style as TaffyStyle};

pub struct Layout<T> {
    style: Style,
    children: NodeChildren<T>
}

impl<T: Renderer> Layout<T> {
    pub(crate) fn new() -> Layout<T> {
        Layout {
            style: Style {
                layout: TaffyStyle::default(),
                overflow: Overflow::Visible
            },
            children: NodeChildren::new()
        }
    }
}

impl<T: Renderer> Node<T> for Layout<T> {
    fn style(&self) -> &Style {
        &self.style
    }
    fn children(&self) -> Option<&NodeChildren<T>> {
        Some(&self.children)
    }
    fn render(&self, context: &mut RenderContext<T>, _layout: taffy::layout::Layout, render_children: &dyn Fn(&mut RenderContext<T>)) {
        render_children(context);
    }
}