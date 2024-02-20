use std::{fmt::Debug, mem, path::PathBuf};
use femtovg::{Color, ErrorKind, ImageFlags, ImageId, Paint, Path};
use crate::{events::handler::EventHandlerDatabase, SharedNode, WeakNode};
use super::{Node, NodeChildren, Style};

#[derive(Debug)]
/// Status of the image - when rendering, image node attempts to load the image and sets this status accordingly.
/// Changes this if you want to change the image. If the previous status was loaded, free the image.
/// In case the loading fails, image load status changes to Error and the node doesn't render.
pub enum ImageLoad {
    LoadFile(PathBuf, ImageFlags),
    // LoadArray(&[u8]),
    LoadVec(Vec<u8>, ImageFlags),
    Loaded(ImageId),
    Error(ErrorKind)
}

#[derive(Debug)]
/// Image node.
/// Sadly doesn't implement `Default` because of the `ImageLoad` enum.
pub struct Image {
    pub style: Style,
    /// The image to be rendered.
    pub image: ImageLoad,
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

    fn render_pre_children(&mut self, context: &mut super::RenderContext, layout: taffy::prelude::Layout) {
        match &self.image {
            ImageLoad::LoadFile(_, _) => {
                let image = mem::replace(&mut self.image, ImageLoad::Error(ErrorKind::UnknownError));
                if let ImageLoad::LoadFile(path, flags) = image {
                    match context.canvas.load_image_file(path, flags) {
                        Ok(image) => {
                            self.image = ImageLoad::Loaded(image);
                        },
                        Err(e) => {
                            self.image = ImageLoad::Error(e);
                        }
                    }
                }
            },
            ImageLoad::LoadVec(data, flags) => {
                match context.canvas.load_image_mem(data, *flags) {
                    Ok(image) => {
                        self.image = ImageLoad::Loaded(image);
                    },
                    Err(e) => {
                        self.image = ImageLoad::Error(e);
                    }
                }
            },
            _ => {}
        }
        let mut path = Path::new();
        path.rounded_rect(
            0.,
            0.,
            layout.size.width,
            layout.size.height,
            self.radius
        );
        match &self.image {
            ImageLoad::Loaded(image) => {
                context.canvas.fill_path(
                    &path,
                    &Paint::image(*image, 0., 0., self.width, self.height, 0., 1.)
                );
            },
            ImageLoad::Error(_) => {
                context.canvas.fill_path(&path, &Paint::color(Color::rgb(255, 0, 0)))
            },
            _ => unreachable!("We just loaded the image before, so it's either loaded or errored out.")
        }
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