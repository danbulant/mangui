use std::fmt::Debug;
use crate::{events::handler::EventHandlerDatabase, SharedNode, WeakNode, FONT_SYSTEM};
use super::{text_render_cache::RENDER_CACHE, Node, NodeChildren, Style, MeasureContext, RenderContext};
use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use femtovg::{Color, Paint, Path};
use taffy::{AvailableSpace, Size};

#[derive(Debug, Default)]
pub struct Text {
    pub style: Style,
    pub text: String,
    pub events: EventHandlerDatabase,
    pub parent: Option<WeakNode>,
    pub metrics: Metrics,
    pub buffer: Option<Buffer>,
    pub paint: Paint
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
            self.buffer = Some(Buffer::new(&mut FONT_SYSTEM.lock().unwrap(), self.metrics));
        }
        let buf = self.buffer.as_mut().unwrap();
        let mut font = FONT_SYSTEM.lock().unwrap();
        buf.set_text(&mut font, &self.text, Attrs::new(), Shaping::Advanced);
    }

    fn render_pre_children(&mut self, context: &mut super::RenderContext, layout: taffy::prelude::Layout) {
        // this can crash, but it should crash earlier during measure -> see the comment there.
        let buf = self.buffer.as_mut().unwrap();
        let mut font = FONT_SYSTEM.lock().unwrap();
        buf.set_size(&mut font, layout.size.width, layout.size.height);
        buf.set_metrics(&mut font, self.metrics.scale(context.scale_factor));
        // fill_to_cmds requires FONT_SYSTEM lock.
        drop(font);
        let mut path = Path::new();
        path.rounded_rect(
            0.,
            0.,
            layout.size.width,
            layout.size.height,
            0.
        );
        context.canvas.fill_path(
            &path,
            &Paint::color(Color::rgb(255, 0, 0))
        );
        let cmds = RENDER_CACHE.lock().unwrap()
            .fill_to_cmds(&mut context.canvas, buf, (0.0, 0.0), context.scale_factor)
            .unwrap();
        context.canvas.draw_glyph_commands(cmds, &self.paint, 1.0);
    }

    fn measure(&mut self, _context: &mut MeasureContext, known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>) -> Size<f32> {
        let width_constraint = known_dimensions.width.unwrap_or(match available_space.width {
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => f32::INFINITY,
            AvailableSpace::Definite(width) => width,
        });
        // yes, this can crash if someone removes `buffer` during render from another thread.
        // though they're asking for it, so let them crash.
        let buf = self.buffer.as_mut().unwrap();
        buf.set_size(&mut FONT_SYSTEM.lock().unwrap(), width_constraint, f32::INFINITY);

        // Compute layout
        buf.shape_until_scroll(&mut FONT_SYSTEM.lock().unwrap(), false);

        // Determine measured size of text
        let (width, total_lines) = buf
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
        let height = total_lines as f32 * buf.metrics().line_height;
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