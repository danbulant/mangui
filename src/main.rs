use std::num::NonZeroU32;
use std::ops::Deref;
use std::sync::{Arc, RwLock, Weak};

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
use taffy::style::Dimension;
use taffy::style_helpers::TaffyMaxContent;
use taffy::Taffy;
use weak_table::PtrWeakKeyHashMap;
use crate::nodes::{layout_recursively, Node, render_recursively, RenderContext, Style, TaffyStyle};
use crate::nodes::layout::Layout;

mod nodes;

type TNode<T> = dyn Node<T>;
type SharedTNode<T> = Arc<RwLock<TNode<T>>>;
type TaffyMap<T> = PtrWeakKeyHashMap<Weak<RwLock<TNode<T>>>, taffy::node::Node>;
type CurrentRenderer = OpenGl;

fn main() {
    let event_loop = EventLoop::new();
    let (buffer_context, gl_display, window, surface) = create_window(&event_loop);

    let renderer = unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s) as *const _) }
        .expect("Cannot create renderer");

    let canvas = Canvas::new(renderer).expect("Cannot create canvas");

    let mut taffy = Taffy::new();
    let mut taffy_map = TaffyMap::new();

    let mut root = Layout::<CurrentRenderer>::new();
    root.style.layout.display = taffy::style::Display::Flex;
    root.style.layout.flex_direction = taffy::style::FlexDirection::Row;
    root.children.push(Arc::new(RwLock::new(nodes::primitives::Rectangle {
        style: Style {
            overflow: nodes::Overflow::Visible,
            layout: TaffyStyle {
                min_size: Size {
                    width: Dimension::Points(100.),
                    height: Dimension::Points(100.)
                },
                ..Default::default()
            }
        },
        color: Color::rgb(255, 0, 0),
        radius: 10.
    })));
    root.children.push(Arc::new(RwLock::new(nodes::primitives::Rectangle {
        style: Style {
            overflow: nodes::Overflow::Visible,
            layout: TaffyStyle {
                min_size: Size {
                    width: Dimension::Points(50.),
                    height: Dimension::Points(100.)
                },
                ..Default::default()
            }
        },
        color: Color::rgb(0, 255, 0),
        radius: 0.
    })));
    let groot: Arc<RwLock<Layout<CurrentRenderer>>> = Arc::new(RwLock::new(root));
    {
        let clonned = groot.clone();
        let root = clonned.read().unwrap();
        let root_style = TNode::<OpenGl>::style(root.deref());
        let root_layout = root_style.layout.to_owned();
        let root_node = taffy.new_leaf(root_layout).unwrap();

        taffy_map.insert(groot.clone(), root_node);
    }

    let mut context = RenderContext {
        canvas,
        node_layout: taffy_map,
        taffy
    };

    // let mut width: u32 = 0;
    // let mut height: u32 = 0;
    let mut should_recompute = true;

    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            // WindowEvent::CursorMoved { position, .. } => {
            //     window.request_redraw();
            // }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                // width = size.width;
                // height = size.height;
                let width: NonZeroU32 = NonZeroU32::new(size.width).unwrap();
                let height: NonZeroU32 = NonZeroU32::new(size.height).unwrap();
                surface.resize(&buffer_context, width, height);
                let mut groot = groot.write().unwrap();
                // let scale_factor = window.scale_factor();
                groot.style.layout.size.width = Dimension::Points(size.width as f32);
                groot.style.layout.size.height = Dimension::Points(size.height as f32);
                drop(groot);
                window.request_redraw();
                should_recompute = true;
            },
            _ => {}
        },
        Event::RedrawRequested(_) => {
            let root: SharedTNode<CurrentRenderer> = groot.clone();
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
    // Make sure the canvas has the right size:
    let size = window.inner_size();
    context.canvas.reset();
    context.canvas.set_size(size.width, size.height, window.scale_factor() as f32);
    // context.canvas.scale(1., -1.); // layout is bottom to top, canvas is top to bottom, this might make it easier?
    context.canvas.clear_rect(0, 0, size.width, size.height, Color::black());

    // Do the render passes here
    render_recursively(root_node, context);

    // Tell renderer to execute all drawing commands
    context.canvas.flush();
    // Display what we've just rendered
    surface.swap_buffers(buffer_context).expect("Could not swap buffers");
}