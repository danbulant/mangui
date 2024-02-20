use std::fmt::Debug;
use femtovg::{ImageId, Paint, Path};
use crate::{events::handler::EventHandlerDatabase, SharedNode, WeakNode};
use super::{Node, NodeChildren, Style};

#[derive(Debug)]
/// Simple image node.
/// This is basically just a wrapper around Rectangle node with a fill of type Paint::image.
/// Use that if you need more options.
pub struct Image {
    pub style: Style,
    /// The image to be rendered. You are responsible for freeing the image data.
    pub image: ImageId,
    /// Image width - note that you also have to set the style accordingly for it to render correctly, this is more about scaling the image
    pub width: f32,
    /// Image height - note that you also have to set the style accordingly for it to render correctly, this is more about scaling the image
    pub height: f32,
    /// Border radius
    pub radius: f32,
    pub events: EventHandlerDatabase,
    pub parent: Option<WeakNode>
}

impl Node for Image {
    fn style(&self) -> &Style {
        &self.style
    }

    fn children(&self) -> Option<&NodeChildren> {
        None
    }

    fn render_pre_children(&self, context: &mut super::RenderContext, layout: taffy::prelude::Layout) {
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
            &Paint::image(self.image, 0., 0., self.width, self.height, 0., 1.)
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