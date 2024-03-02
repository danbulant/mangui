use std::fmt::Debug;
use crate::{events::handler::EventHandlerDatabase, SharedNode, WeakNode, FONT_SYSTEM};
use super::{text_render_cache::RENDER_CACHE, Node, NodeChildren, Style, MeasureContext, RenderContext};
use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use taffy::{AvailableSpace, Size};
use femtovg::{Color, Paint, Path};
use crate::nodes::primitives::draw_rect;
use crate::nodes::text_render_cache::TextConfig;

#[derive(Debug, Default)]
pub struct Text {
    pub style: Style,
    pub text: String,
    pub events: EventHandlerDatabase,
    pub parent: Option<WeakNode>,
    pub buffer: Option<Buffer>
}

impl Text {
    pub fn new(text: String, metrics: Metrics) -> Text {
        Text {
            text,
            ..Default::default()
        }
    }
    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    fn get_metrics(&self) -> Metrics {
        let fontSize = self.style.font_size.unwrap_or(16.);
        Metrics {
            font_size: fontSize,
            line_height: fontSize * self.style.line_height.unwrap_or(1.2)
        }
    }
}

impl Node for Text {
    fn style(&self) -> &Style {
        &self.style
    }

    fn children(&self) -> Option<&NodeChildren> {
        None
    }

    fn prepare_render(&mut self, _context: &mut RenderContext) {
        if let None = self.buffer {
            self.buffer = Some(Buffer::new(&mut FONT_SYSTEM.lock().unwrap(), self.get_metrics()));
        }
        let buf = self.buffer.as_mut().unwrap();
        let mut font = FONT_SYSTEM.lock().unwrap();
        buf.set_text(&mut font, &self.text, Attrs::new(), Shaping::Advanced);
    }

    fn render_pre_children(&mut self, context: &mut super::RenderContext, layout: taffy::prelude::Layout) {
        if let Some(background) = &self.style.background { draw_rect(layout.size, background, self.style.border_radius, &mut context.canvas); }
        let metrics = self.get_metrics();
        // this can crash, but it should crash earlier during measure -> see the comment there.
        let buf = self.buffer.as_mut().unwrap();
        let mut font = FONT_SYSTEM.lock().unwrap();
        let offset_size = (
            layout.padding.left + layout.padding.right + layout.border.left + layout.border.right,
            layout.padding.top + layout.padding.bottom + layout.border.top + layout.border.bottom
            );
        // the height * scale factor is an ugly hack to fix height of the text... not sure why it's wrong in the first place
        buf.set_size(&mut font, layout.content_size.width - offset_size.0, (layout.content_size.height * context.scale_factor) - offset_size.1);
        buf.set_metrics(&mut font, metrics.scale(context.scale_factor));
        // fill_to_cmds requires FONT_SYSTEM lock.
        drop(font);
        let position = (
                layout.padding.left + layout.border.left,
                layout.padding.top + layout.border.top
            );
        let cmds = RENDER_CACHE.lock().unwrap()
            .fill_to_cmds(&mut context.canvas, buf, position, context.scale_factor, TextConfig { hint: false, subpixel: false })
            .unwrap();
        context.canvas.draw_glyph_commands(cmds, self.style.text_fill.as_ref().unwrap_or(&Paint::color(Color::black())), context.scale_factor);
    }

    fn measure(&mut self, context: &mut MeasureContext, known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>) -> Size<f32> {
        let width_constraint = known_dimensions.width.unwrap_or(match available_space.width {
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => f32::INFINITY,
            AvailableSpace::Definite(width) => width,
        });
        let metrics = self.get_metrics();
        // yes, this can crash if someone removes `buffer` during render from another thread.
        // though they're asking for it, so let them crash.
        let buf = self.buffer.as_mut().unwrap();
        let mut font = FONT_SYSTEM.lock().unwrap();
        buf.set_size(&mut font, width_constraint, f32::INFINITY);
        buf.set_metrics(&mut font, metrics.scale(context.scale_factor));

        // Compute layout
        buf.shape_until_scroll(&mut font, true);
        drop(font);

        // Determine measured size of text
        let (width, total_lines) = buf
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
        // fixes text not rendering in some cases (??????)
        let height = (total_lines as f32 * buf.metrics().line_height + 1.0) / context.scale_factor;
        // fixes flickering of text on devices with non-integer scale factors due to loss of precision
        let width = width + 0.5;

        Size { width, height }
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