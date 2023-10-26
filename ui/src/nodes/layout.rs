use std::fmt::{Debug, Formatter};
use crate::{nodes::{Node, NodeChildren, Style}, events::handler::EventHandlerDatabase};
use taffy::style::Dimension;

/// A simple layout node which contains children.
#[derive(Default)]
pub struct Layout {
    pub style: Style,
    pub children: NodeChildren,
    pub events: EventHandlerDatabase
}

impl Layout {
    pub fn new(children: NodeChildren) -> Layout {
        Layout {
            style: Style::default(),
            children,
            events: EventHandlerDatabase::default()
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

    fn event_handlers(&self) -> Option<crate::events::handler::InnerEventHandlerDataset> {
        Some(self.events.handlers.clone())
    }
}