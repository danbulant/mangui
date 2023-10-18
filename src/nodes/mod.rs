use std::sync::Arc;
use femtovg::{Canvas, Color, Renderer};
use femtovg::renderer::OpenGl;
use taffy::geometry::Size;
use taffy::layout::Layout;
use taffy::prelude::Dimension;
use taffy::style::Style as TaffyStyle;
use taffy::Taffy;
use crate::{GNode, TaffyMap};

#[derive(Copy, Clone, Default)]
pub enum Overflow {
    #[default]
    /// Content is not clipped and may be rendered outside the element's box
    Visible,
    /// Clips the content at the border of the element
    Hidden,
    // tbd :)
    // Scroll,
    // Auto
}

pub struct Style {
    pub(crate) layout: TaffyStyle,
    pub overflow: Overflow
}

pub trait Node<T: Renderer> {
    fn style(&self) -> &Style;
    fn children(&self) -> Option<&Vec<Arc<dyn Node<T>>>>;
    fn render(&self, canvas: &mut Canvas<T>, layout: &Layout, render_children: &dyn Fn(&mut Canvas<T>) -> ());
}

pub fn render_recursively(selfref: &Arc<GNode>, canvas: &mut Canvas<OpenGl>, taffy_map: &TaffyMap, taffy: &Taffy) {
    let node = taffy_map.get(selfref).unwrap();
    let layout = taffy.layout(*node).unwrap();
    let sself = selfref.clone();
    canvas.save_with(move |mut canvas| {
        canvas.translate(layout.location.x, layout.location.y);
        sself.render(&mut canvas, &layout, & (|canvas| {
            if let Some(children) = sself.children() {
                for child in children {
                    render_recursively(child, canvas, taffy_map, taffy);
                }
            }
        }));
    });
}

pub struct RedBoxDemo {
    style: Style
}

impl RedBoxDemo {
    pub(crate) fn new() -> RedBoxDemo {
        RedBoxDemo {
            style: Style {
                layout: TaffyStyle {
                    size: Size {
                        width: Dimension::Points(30.),
                        height: Dimension::Points(30.)
                    },
                    ..TaffyStyle::default()
                },
                overflow: Overflow::Visible
            }
        }
    }
}

impl<T: Renderer> Node<T> for RedBoxDemo {
    fn children(&self) -> Option<&Vec<Arc<dyn Node<T>>>> {
        None
    }
    fn style(&self) -> &Style {
        &self.style
    }
    fn render(&self, canvas: &mut Canvas<T>, layout: &Layout, _render_children: &dyn Fn(&mut Canvas<T>)) {
        canvas.clear_rect(
            0,
            0,
            30,
            30,
            Color::rgbf(1., 0., 0.),
        );
    }
}