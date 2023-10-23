use std::fmt::{Debug, Formatter};
use femtovg::Renderer;
use crate::nodes::{Node, NodeChildren, Overflow, RenderContext, Style};
use taffy::style::{Style as TaffyStyle, Dimension};

#[derive(Clone, Default)]

pub struct Layout<T> {
    pub style: Style,
    pub children: NodeChildren<T>
}

impl<T: Renderer> Layout<T> {
    pub fn new() -> Layout<T> {
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
    fn resize(&mut self, width: f32, height: f32) {
        self.style.layout.size.width = Dimension::Points(width);
        self.style.layout.size.height = Dimension::Points(height);
    }
}