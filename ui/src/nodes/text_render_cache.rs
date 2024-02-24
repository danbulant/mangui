use cosmic_text::{Buffer, CacheKey, SubpixelBin};
use femtovg::{
    Atlas, DrawCommand, ErrorKind, GlyphDrawCommands, ImageFlags, ImageId,
    ImageSource, Quad
};
use std::{collections::HashMap, sync::Mutex};
use femtovg::imgref::{Img, ImgRef};
use femtovg::rgb::RGBA8;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::{Format, Vector};

use crate::FONT_SYSTEM;

use super::CanvasRenderer;


const GLYPH_PADDING: u32 = 1;
const GLYPH_MARGIN: u32 = 1;
const TEXTURE_SIZE: usize = 512;

pub struct FontTexture {
    atlas: Atlas,
    image_id: ImageId
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TextConfig {
    pub hint: bool,
    pub subpixel: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct RenderedGlyph {
    texture_index: usize,
    width: u32,
    height: u32,
    offset_x: i32,
    offset_y: i32,
    atlas_x: u32,
    atlas_y: u32,
    color_glyph: bool,
}

#[derive(Default)]
pub struct RenderCache {
    scale_context: ScaleContext,
    rendered_glyphs: HashMap<CacheKey, Option<RenderedGlyph>>,
    glyph_textures: Vec<FontTexture>,
}


lazy_static::lazy_static! {
    pub static ref RENDER_CACHE: Mutex<RenderCache> = Mutex::new(RenderCache::default());
}

impl RenderCache {
    /// Generates draw commands from cosmic text buffer.
    /// Note that this requires a lock on FONT_SYSTEM.
    pub(crate) fn fill_to_cmds(
        &mut self,
        canvas: &mut CanvasRenderer,
        buffer: &Buffer,
        position: (f32, f32),
        scale: f32,
        config: TextConfig
    ) -> Result<GlyphDrawCommands, ErrorKind> {
        let mut alpha_cmd_map = HashMap::new();
        let mut color_cmd_map = HashMap::new();

        let lines = buffer.layout_runs().filter(|run| run.line_w != 0.0).count();
        let total_height = lines as f32 * buffer.metrics().line_height;
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical(position, scale);
                let mut cache_key = physical_glyph.cache_key;
                // perform cache lookup for rendered glyph
                let Some(rendered) = self.rendered_glyphs.entry(cache_key).or_insert_with(|| {
                    // ...or insert it

                    // do the actual rasterization
                    let font = FONT_SYSTEM.lock().unwrap()
                        .get_font(cache_key.font_id)
                        .expect("Shaped a nonexistent font. What?");
                    let mut scaler = self
                        .scale_context
                        .builder(font.as_swash())
                        .size(f32::from_bits(cache_key.font_size_bits))
                        .hint(config.hint)
                        .build();
                    let offset = Vector::new(cache_key.x_bin.as_float(), cache_key.y_bin.as_float());
                    let rendered = Render::new(&[
                        Source::ColorOutline(0),
                        Source::ColorBitmap(StrikeWith::BestFit),
                        Source::Outline,
                    ])
                        .format(if config.subpixel { Format::Subpixel } else { Format::Alpha })
                        .offset(offset)
                        .render(&mut scaler, cache_key.glyph_id);

                    // upload it to the GPU
                    rendered.map(|rendered| {
                        // pick an atlas texture for our glyph
                        let content_w = rendered.placement.width as usize;
                        let content_h = rendered.placement.height as usize;
                        let alloc_w = rendered.placement.width + (GLYPH_MARGIN + GLYPH_PADDING) * 2;
                        let alloc_h = rendered.placement.height + (GLYPH_MARGIN + GLYPH_PADDING) * 2;
                        let used_w = rendered.placement.width + GLYPH_PADDING * 2;
                        let used_h = rendered.placement.height + GLYPH_PADDING * 2;
                        let mut found = None;
                        for (texture_index, glyph_atlas) in self.glyph_textures.iter_mut().enumerate() {
                            if let Some((x, y)) = glyph_atlas.atlas.add_rect(alloc_w as usize, alloc_h as usize) {
                                found = Some((texture_index, x, y));
                                break;
                            }
                        }

                        let (texture_index, atlas_alloc_x, atlas_alloc_y) =
                            found.unwrap_or_else(|| {
                                // if no atlas could fit the texture, make a new atlas tyvm
                                // TODO error handling
                                let mut atlas = Atlas::new(TEXTURE_SIZE, TEXTURE_SIZE);
                                let image_id = canvas
                                    .create_image(
                                        Img::new(
                                            vec![
                                                RGBA8::new(0, 0, 0, 0);
                                                TEXTURE_SIZE * TEXTURE_SIZE
                                            ],
                                            TEXTURE_SIZE,
                                            TEXTURE_SIZE,
                                        )
                                            .as_ref(),
                                        ImageFlags::empty(),
                                    )
                                    .unwrap();
                                let texture_index = self.glyph_textures.len();
                                let (x, y) =
                                    atlas.add_rect(alloc_w as usize, alloc_h as usize).unwrap();
                                self.glyph_textures.push(FontTexture { atlas, image_id });
                                (texture_index, x, y)
                            });

                        let atlas_used_x = atlas_alloc_x as u32 + GLYPH_MARGIN;
                        let atlas_used_y = atlas_alloc_y as u32 + GLYPH_MARGIN;
                        let atlas_content_x = atlas_alloc_x as u32 + GLYPH_MARGIN + GLYPH_PADDING;
                        let atlas_content_y = atlas_alloc_y as u32 + GLYPH_MARGIN + GLYPH_PADDING;

                        let mut src_buf = Vec::with_capacity(content_w * content_h);
                        match rendered.content {
                            Content::Mask => {
                                for chunk in rendered.data.chunks_exact(1) {
                                    src_buf.push(RGBA8::new(chunk[0], 0, 0, 0));
                                }
                            }
                            Content::Color => {
                                for chunk in rendered.data.chunks_exact(4) {
                                    src_buf.push(RGBA8::new(chunk[0], chunk[1], chunk[2], chunk[3]));
                                }
                            }
                            Content::SubpixelMask => unreachable!(),
                        }
                        canvas
                            .update_image::<ImageSource>(
                                self.glyph_textures[texture_index].image_id,
                                ImgRef::new(&src_buf, content_w, content_h).into(),
                                atlas_content_x as usize,
                                atlas_content_y as usize,
                            )
                            .unwrap();

                        RenderedGlyph {
                            texture_index,
                            width: used_w,
                            height: used_h,
                            offset_x: rendered.placement.left,
                            offset_y: rendered.placement.top,
                            atlas_x: atlas_used_x,
                            atlas_y: atlas_used_y,
                            color_glyph: matches!(rendered.content, Content::Color),
                        }
                    })
                }) else { continue };

                let cmd_map = if rendered.color_glyph {
                    &mut color_cmd_map
                } else {
                    &mut alpha_cmd_map
                };

                let cmd = cmd_map.entry(rendered.texture_index).or_insert_with(|| DrawCommand {
                    image_id: self.glyph_textures[rendered.texture_index].image_id,
                    quads: Vec::new(),
                });

                let mut q = Quad::default();
                let it = 1.0 / TEXTURE_SIZE as f32;
                q.x0 = (physical_glyph.x + rendered.offset_x - GLYPH_PADDING as i32) as f32;
                q.y0 = (physical_glyph.y - rendered.offset_y - GLYPH_PADDING as i32) as f32 + run.line_y;
                q.x1 = q.x0 + rendered.width as f32;
                q.y1 = q.y0 + rendered.height as f32;

                q.s0 = rendered.atlas_x as f32 * it;
                q.t0 = rendered.atlas_y as f32 * it;
                q.s1 = (rendered.atlas_x + rendered.width) as f32 * it;
                q.t1 = (rendered.atlas_y + rendered.height) as f32 * it;

                cmd.quads.push(q);
            }
        }

        Ok(GlyphDrawCommands {
            alpha_glyphs: alpha_cmd_map.into_values().collect(),
            color_glyphs: color_cmd_map.into_values().collect(),
        })
    }
}