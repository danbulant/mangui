use std::num::NonZeroU32;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color, Renderer};
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
use taffy::Taffy;
use weak_table::PtrWeakKeyHashMap;
use crate::nodes::{Node, render_recursively};

mod nodes;

type GNode = dyn Node<OpenGl>;
type TaffyMap = PtrWeakKeyHashMap<Weak<GNode>, taffy::node::Node>;

fn main() {
    let event_loop = EventLoop::new();
    let (context, gl_display, window, surface) = create_window(&event_loop);

    let renderer = unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s) as *const _) }
        .expect("Cannot create renderer");

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

    let mut taffy = Taffy::new();
    let mut taffy_map = TaffyMap::new();

    let root: Arc<GNode> = Arc::new(nodes::RedBoxDemo::new());
    let root_style = <dyn Node<OpenGl>>::style(root.deref());
    let root_node = taffy.new_leaf(root_style.layout.to_owned()).unwrap();

    taffy_map.insert(root.clone(), root_node);

    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            // WindowEvent::CursorMoved { position, .. } => {
            //     window.request_redraw();
            // }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                let width: NonZeroU32 = NonZeroU32::new(size.width).unwrap();
                let height: NonZeroU32 = NonZeroU32::new(size.height).unwrap();
                surface.resize(&context, width, height);
                window.request_redraw();
            },
            _ => {}
        },
        Event::RedrawRequested(_) => {
            render(&context, &surface, &window, &mut canvas, &root, &taffy_map, &taffy);
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
    context: &PossiblyCurrentContext,
    surface: &Surface<WindowSurface>,
    window: &Window,
    canvas: &mut Canvas<OpenGl>,
    root_node: &Arc<GNode>,
    taffy_map: &TaffyMap,
    taffy: &Taffy
) {
    // Make sure the canvas has the right size:
    let size = window.inner_size();
    canvas.set_size(size.width, size.height, window.scale_factor() as f32);
    canvas.scale(1., -1.); // layout is bottom to top, canvas is top to bottom, this might make it easier?
    canvas.clear_rect(0, 0, size.width, size.height, Color::black());

    // Do the render passes here
    render_recursively(root_node, canvas, taffy_map, taffy);

    // Tell renderer to execute all drawing commands
    canvas.flush();
    // Display what we've just rendered
    surface.swap_buffers(context).expect("Could not swap buffers");
}