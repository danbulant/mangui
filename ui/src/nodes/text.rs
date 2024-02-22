use std::fmt::Debug;
use crate::{events::handler::EventHandlerDatabase, SharedNode, WeakNode, FONT_SYSTEM};
use super::{text_render_cache::RENDER_CACHE, Node, NodeChildren, Style};
use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use femtovg::Paint;

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

    fn render_pre_children(&mut self, context: &mut super::RenderContext, layout: taffy::prelude::Layout) {
        if let None = self.buffer {
            self.buffer = Some(Buffer::new(&mut FONT_SYSTEM.lock().unwrap(), self.metrics));
        }
        let buf = self.buffer.as_mut().unwrap();
        let mut font = FONT_SYSTEM.lock().unwrap();
        buf.set_text(&mut font, &self.text, Attrs::new(), Shaping::Advanced);
        buf.set_size(&mut font, layout.size.width, layout.size.height);
        buf.set_metrics(&mut font, self.metrics.scale(context.scale_factor));
        drop(font);
        let cmds = RENDER_CACHE.lock().unwrap()
            .fill_to_cmds(&mut context.canvas, buf, (0.0, 0.0), context.scale_factor)
            .unwrap();
        context.canvas.draw_glyph_commands(cmds, &self.paint, 1.0);
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