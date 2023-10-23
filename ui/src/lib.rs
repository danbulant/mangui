use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::sync::{Arc, RwLock, Weak, RwLockReadGuard};

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color};
use glutin::surface::Surface;
use glutin::{context::PossiblyCurrentContext, display::Display};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::{dpi::PhysicalSize, window::Window};

use glutin::{
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use taffy::geometry::Size;
use taffy::style_helpers::TaffyMaxContent;
use taffy::Taffy;
use weak_table::PtrWeakKeyHashMap;
use crate::nodes::{layout_recursively, Node, render_recursively, RenderContext};

pub mod nodes;
pub use taffy;
pub use femtovg;

pub type CurrentRenderer = OpenGl;
pub type TNode<T> = dyn Node<T>;
pub type SharedTNode<T> = Arc<RwLock<TNode<T>>>;
type WeakTNode<T> = Weak<RwLock<TNode<T>>>;
type TNodePtr<T> = Option<Vec<WeakTNode<T>>>;
type NodeLayoutMap<T> = PtrWeakKeyHashMap<Weak<RwLock<TNode<T>>>, taffy::node::Node>;

pub fn run_event_loop(root_node: SharedTNode<CurrentRenderer>) -> ! {
    let event_loop = EventLoop::new();
    let (buffer_context, gl_display, window, surface) = create_window(&event_loop);

    let renderer = unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s) as *const _) }
        .expect("Cannot create renderer");

    let canvas = Canvas::new(renderer).expect("Cannot create canvas");

    let mut taffy = Taffy::new();
    let mut taffy_map = NodeLayoutMap::new();
    {
        let clonned = root_node.clone();
        let root = clonned.read().unwrap();
        let root_style = root.deref().style();
        let root_layout = root_style.layout.to_owned();
        let taffy_root_node = taffy.new_leaf(root_layout).unwrap();

        taffy_map.insert(root_node.clone(), taffy_root_node);
    }

    let mut context = RenderContext {
        canvas,
        node_layout: taffy_map,
        taffy,
        mouse: None,
        keyboard_focus: None
    };
    let root = root_node.clone();

    let mut should_recompute = true;

    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            // WindowEvent::CursorMoved { position, .. } => {
            //     window.request_redraw();
            // }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                let width: NonZeroU32 = NonZeroU32::new(size.width).unwrap();
                let height: NonZeroU32 = NonZeroU32::new(size.height).unwrap();
                surface.resize(&buffer_context, width, height);
                let mut groot = root_node.write().unwrap();
                // let scale_factor = window.scale_factor();
                groot.resize(size.width as f32, size.height as f32);
                drop(groot);
                window.request_redraw();
                should_recompute = true;
            },
            _ => {}
        },
        Event::RedrawRequested(_) => {
            if should_recompute {
                layout_recursively(&root, &mut context);
                let src_nodes = context.node_layout.values().map(|v| v.to_owned()).collect::<Vec<_>>();
                context.node_layout.remove_expired();
                let dst_nodes = context.node_layout.values().map(|v| v.to_owned()).collect::<Vec<_>>();
                for src_node in src_nodes {
                    if !dst_nodes.contains(&src_node) {
                        context.taffy.remove(src_node).unwrap();
                        dbg!("Removed node", src_node);
                    }
                }
                for (node, taffy_node) in context.node_layout.iter() {
                    let node = node.read().unwrap();
                    let node_style = node.style();
                    context.taffy.set_style(*taffy_node, node_style.layout.to_owned()).unwrap();
                }
                context.taffy.compute_layout(*context.node_layout.get(&root).unwrap(), Size::MAX_CONTENT).unwrap();
                should_recompute = false;
                // Additional optimizations could be done here
                // - When setting styles, check that the styles aren't the same (taffy doesn't do that and instead always mark it as dirty)
                // - taffy seems to always recompute (maybe internally checks dirtyness, I didn't look into it that much)
                // - the weakmap dance (src_nodes, dst_nodes) could be avoided by changing the weakmap used
                //   (weakmap removes keys when you attempt to read them, we could change it so that we could iterate on them and remove them in one go)
                // could perhaps be a significant boost regarding memory usage (and performance) during large layout changes
                // dbg!("recomputed");
            }
            // dbg!(&root);
            render(&buffer_context, &surface, &window, &mut context, &root);
        },
        _ => {}
    })
}

fn create_window(event_loop: &EventLoop<()>) -> (PossiblyCurrentContext, Display, Window, Surface<WindowSurface>) {
    let window_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1000., 600.))
        .with_title("Mangui test");

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(event_loop, template, |mut configs| configs.next().unwrap())
        .unwrap();

    let window = window.unwrap();

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));

    let mut not_current_gl_context =
        Some(unsafe { gl_display.create_context(&gl_config, &context_attributes).unwrap() });

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(1000).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );

    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };

    (
        not_current_gl_context.take().unwrap().make_current(&surface).unwrap(),
        gl_display,
        window,
        surface,
    )
}

fn render(
    buffer_context: &PossiblyCurrentContext,
    surface: &Surface<WindowSurface>,
    window: &Window,
    context: &mut RenderContext<CurrentRenderer>,
    root_node: &SharedTNode<CurrentRenderer>
) {
    let size = window.inner_size();
    context.canvas.reset();
    context.canvas.set_size(size.width, size.height, window.scale_factor() as f32);
    context.canvas.clear_rect(0, 0, size.width, size.height, Color::black());

    render_recursively(root_node, context);

    context.canvas.flush();
    surface.swap_buffers(buffer_context).expect("Could not swap buffers");
}