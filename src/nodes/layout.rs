use std::fmt::{Debug, Formatter};
use femtovg::Renderer;
use crate::nodes::{Node, NodeChildren, Overflow, RenderContext, Style};
use taffy::style::{Style as TaffyStyle};

#[derive(Clone, Default)]

pub struct Layout<T> {
    pub style: Style,
    pub children: NodeChildren<T>
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

impl<T: Renderer> Debug for Layout<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layout")
            .field("style", &self.style)
            .field("children", &self.children)
            .finish()
    }
}

impl<T: Renderer> Node<T> for Layout<T> {
    fn style(&self) -> &Style {
        &self.style
    }
    fn children(&self) -> Option<&NodeChildren<T>> {
        Some(&self.children)
    }
    // fn render_(&self, context: &mut RenderContext<T>, _layout: taffy::layout::Layout, render_children: &dyn Fn(&mut RenderContext<T>)) {
    //     render_children(context);
    // }

}