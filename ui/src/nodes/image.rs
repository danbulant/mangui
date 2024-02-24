use std::{fmt::Debug, mem, path::PathBuf};
use std::sync::Mutex;
use femtovg::{Color, ErrorKind, ImageFlags, ImageId, Paint, Path};
use taffy::{AvailableSpace, Size};
use crate::{events::handler::EventHandlerDatabase, SharedNode, WeakNode};
use super::{MeasureContext, Node, NodeChildren, RenderContext, Style};

#[derive(Debug, Default)]
/// Status of the image - when rendering, image node attempts to load the image and sets this status accordingly.
/// Changes this if you want to change the image. If the previous status was loaded, free the image.
/// In case the loading fails, image load status changes to Error and the node doesn't render.
pub enum ImageLoad {
    LoadFile(PathBuf, ImageFlags),
    // LoadArray(&[u8]),
    LoadVec(Vec<u8>, ImageFlags),
    Loaded(ImageHandle),
    Error(ErrorKind),
    #[default]
    Empty
}

#[derive(Debug)]
pub struct ImageHandle {
    image: ImageId
}

impl ImageHandle {
    fn new(image: ImageId) -> ImageHandle {
        ImageHandle {
            image
        }
    }
}

impl Drop for ImageHandle {
    fn drop(&mut self) {
        IMAGES_TO_UNLOAD.lock().unwrap().push(self.image);
    }
}

#[derive(Debug, Default)]
/// Image node.
/// Sadly doesn't implement `Default` because of the `ImageLoad` enum.
pub struct Image {
    pub style: Style,
    /// The image to be rendered.
    pub image: ImageLoad,
    /// Border radius
    pub radius: f32,
    pub events: EventHandlerDatabase,
    pub parent: Option<WeakNode>
}

lazy_static::lazy_static! {
    pub static ref IMAGES_TO_UNLOAD: Mutex<Vec<ImageId>> = Mutex::new(Vec::new());
}

impl Node for Image {
    fn style(&self) -> &Style {
        &self.style
    }

    fn children(&self) -> Option<&NodeChildren> {
        None
    }

    fn prepare_render(&mut self, context: &mut RenderContext) {
        match &self.image {
            ImageLoad::LoadFile(_, _) => {
                let image = mem::replace(&mut self.image, ImageLoad::Error(ErrorKind::UnknownError));
                if let ImageLoad::LoadFile(path, flags) = image {
                    match context.canvas.load_image_file(path, flags) {
                        Ok(image) => {
                            self.image = ImageLoad::Loaded(ImageHandle::new(image));
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
                        self.image = ImageLoad::Loaded(ImageHandle::new(image));
                    },
                    Err(e) => {
                        self.image = ImageLoad::Error(e);
                    }
                }
            },
            _ => {}
        }
    }

    fn render_pre_children(&mut self, context: &mut super::RenderContext, layout: taffy::prelude::Layout) {
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
                    &Paint::image(image.image, 0., 0., layout.size.width, layout.size.height, 0., 1.)
                );
            },
            ImageLoad::Error(_) => {
                context.canvas.fill_path(&path, &Paint::color(Color::rgb(255, 0, 0)))
            },
            _ => {
                // this shouldn't happen as the image should be loaded earlier during the render pass,
                // but someone can still change the image in another thread
            }
        }
    }

    fn measure(&mut self, context: &mut MeasureContext, known_dimensions: Size<Option<f32>>, _available_space: Size<AvailableSpace>) -> Size<f32> {
        match &self.image {
            ImageLoad::Loaded(image) => {
                match context.canvas.image_size(image.image) {
                    Ok((img_width, img_height)) => {
                        let img_width = img_width as f32;
                        let img_height = img_height as f32;
                        match (known_dimensions.width, known_dimensions.height) {
                            (Some(width), Some(height)) => Size { width, height },
                            (Some(width), None) => Size { width, height: (width / img_width) * img_height },
                            (None, Some(height)) => Size { width: (height / img_height) * img_width, height },
                            (None, None) => Size { width: img_width, height: img_height },
                        }
                    },
                    _ => Size::ZERO
                }
            },
            _ => Size::ZERO
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