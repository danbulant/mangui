use std::fmt::{Debug, Formatter};
use femtovg::Renderer;
use crate::nodes::{Node, NodeChildren, Overflow, Style};
use taffy::style::{Style as TaffyStyle, Dimension};

#[derive(Clone, Default)]

pub struct Layout {
    pub style: Style,
    pub children: NodeChildren
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
            style: Style {
                layout: TaffyStyle::default(),
                overflow: Overflow::Visible
            },
            children: NodeChildren::new()
        }
    }
}

impl Debug for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layout")
            .field("style", &self.style)
            .field("children", &self.children)
            .finish()
    }
}

impl Node for Layout {
    fn style(&self) -> &Style {
        &self.style
    }
    fn children(&self) -> Option<&NodeChildren> {
        Some(&self.children)
    }
    fn resize(&mut self, width: f32, height: f32) {
        self.style.layout.size.width = Dimension::Points(width);
        self.style.layout.size.height = Dimension::Points(height);
    }
}