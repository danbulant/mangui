use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use crate::{nodes::{Node, NodeChildren, Style}, events::handler::EventHandlerDatabase, WeakNode, SharedNode};
use taffy::style::Dimension;

/// A simple layout node which contains children.
#[derive(Default)]
pub struct Layout {
    pub style: Style,
    pub children: NodeChildren,
    pub events: EventHandlerDatabase,
    pub parent: Option<WeakNode>
}

impl Layout {
    pub fn new(children: NodeChildren) -> Layout {
        Layout {
            style: Style::default(),
            children,
            events: EventHandlerDatabase::default(),
            parent: None
        }
    }
    pub fn empty() -> Layout {
        Layout {
            style: Style::default(),
            children: NodeChildren::default(),
            events: EventHandlerDatabase::default(),
            parent: None
        }
    }
    
    pub fn style(mut self, style: Style) -> Layout {
        self.style = style;
        self
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
        self.style.layout.size.width = Dimension::Length(width);
        self.style.layout.size.height = Dimension::Length(height);
    }

    fn add_child_at(&mut self, child: crate::SharedNode, index: usize) -> Result<(), super::ChildAddError> {
        let mut index = index;
        if let Some(i) = self.has_child(&child) {
            self.children.remove(i);
            if i < index {
                index -= 1;
            }
        }
        self.children.insert(index, child);
        Ok(())
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
    fn remove_child(&mut self, child: &SharedNode) -> Result<(), super::ChildAddError> {
        if let Some(i) = self.has_child(child) {
            self.children.remove(i);
            Ok(())
        } else {
            Ok(())
        }
    }
}