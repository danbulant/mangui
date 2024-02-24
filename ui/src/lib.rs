use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, Weak};

use cosmic_text::FontSystem;
use events::{Location, MouseValue, NodeEvent, MouseEvent};
use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color};
use glutin::surface::Surface;
use glutin::{context::PossiblyCurrentContext, display::Display};
use glutin_winit::DisplayBuilder;
use nodes::{get_element_at, run_event_handlers, run_single_event_handlers};
use raw_window_handle::HasRawWindowHandle;
use winit::event::{Event, WindowEvent, Modifiers, DeviceId};
use winit::event_loop::EventLoop;
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
use taffy::{style::AvailableSpace, TaffyTree};
use weak_table::PtrWeakKeyHashMap;
use crate::nodes::{update_taffynode_children, MeasureContext, Node, render_recursively, RenderContext, prepare_render_recursively};

pub mod nodes;
pub mod events;
pub use taffy;
pub use femtovg;
pub use cosmic_text;

pub type CurrentRenderer = OpenGl;
pub type SharedNode = Arc<RwLock<dyn Node>>;
type WeakNode = Weak<RwLock<dyn Node>>;
type NodePtr = Option<Vec<WeakNode>>;
type NodeLayoutMap = PtrWeakKeyHashMap<Weak<RwLock<dyn Node>>, taffy::tree::NodeId>;

lazy_static::lazy_static! {
    pub static ref FONT_SYSTEM: Mutex<FontSystem> = Mutex::new(FontSystem::new());
}

/// The entry point of the UI.
pub struct MainEntry {
    /// The root node of the UI
    pub root: SharedNode,
    /// Write an empty message to this receiver to schedule a frame.
    /// This is checked every 'frame' based on the monitor refresh rate.
    /// If there are no messages and no user input, no frame is scheduled.
    /// Currently, you don't need to use this after an event callback - a frame is scheduled after any event.
    /// The "render queue" is cleared on each frame so that sending multiple values to this channel will only schedule one frame.
    pub render: std::sync::mpsc::Receiver<()>,
}

/// Starts the event loop.
///
/// The event loop only returns when the window is closed, and all the resources regarding the window are freed.
/// Note that the DOM tree may not be destroyed if you hold a reference to it, and the DOM tree can be used again, although it's discouraged -
/// your app should exit at this point and only do cleanup.
pub fn run_event_loop(entry: MainEntry) -> () {
    let event_loop = EventLoop::new().unwrap();
    let (buffer_context, gl_display, window, surface) = create_window(&event_loop);

    if let Err(res) = surface.set_swap_interval(&buffer_context, glutin::surface::SwapInterval::Wait(NonZeroU32::new(1).unwrap())) {
        dbg!("Could not set swap interval (vsync)", res);
    }

    let renderer = unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s) as *const _) }
        .expect("Cannot create renderer");

    let canvas = Canvas::new(renderer).expect("Cannot create canvas");

    let mut taffy = TaffyTree::new();
    let mut taffy_map = NodeLayoutMap::new();
    {
        let cloned = entry.root.clone();
        let root = cloned.read().unwrap();
        let root_style = root.deref().style();
        let root_layout = root_style.layout.to_owned();
        let taffy_root_node = taffy.new_leaf(root_layout).unwrap();

        taffy_map.insert(entry.root.clone(), taffy_root_node);
    }

    let size = window.inner_size();
    let mut context = RenderContext {
        canvas,
        node_layout: taffy_map,
        taffy,
        mouse: None,
        keyboard_focus: None,
        scale_factor: window.scale_factor() as f32,
        window_size: Size { width: size.width as f32, height: size.height as f32 }
    };
    let root = entry.root.clone();

    let mut should_recompute = true;

    let mut modifiers = Modifiers::default();
    let focus_path: Option<Vec<WeakNode>> = None;
    let mut mouse_values: HashMap<DeviceId, MouseValue> = HashMap::new();

    event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::MouseWheel { device_id: _, delta: _, phase: _, .. } => {},
            WindowEvent::CursorMoved { device_id, position, .. } => {
                let mouse_value = mouse_values.get(&device_id);
                let (movement, location, mouse_value) = match mouse_value {
                    Some(mouse_value) => {
                        let location = (position.x, position.y).into();
                        let movement = location - mouse_value.last_location;
                        (movement, location, MouseValue {
                            last_location: location,
                            buttons: mouse_value.buttons
                        })
                    },
                    None => {
                        let location = (position.x, position.y).into();
                        let movement = Location::new(0., 0.);
                        let value = MouseValue {
                            last_location: location,
                            buttons: 0
                        };
                        (movement, location, value)
                    }
                };
                let buttons = mouse_value.buttons;
                mouse_values.insert(device_id, mouse_value);

                let path = get_element_at(&root, &context, location);

                if let Some(path) = path {
                    let target_layout = context.node_layout.get(path.last().unwrap());
                    let target_layout = match target_layout {
                        Some(target_layout) => target_layout,
                        None => { return; }
                    };
                    let target_layout = context.taffy.layout(target_layout.to_owned()).unwrap();
                    let event = NodeEvent {
                        target: path.last().unwrap().clone(),
                        path: path.clone(),
                        event: events::InnerEvent::MouseMove(MouseEvent {
                            button: None,
                            buttons,
                            client: location,
                            movement,
                            device: device_id,
                            modifiers,
                            offset: location - target_layout.location.into()
                        })
                    };

                    run_event_handlers(path, event);
                    window.request_redraw();
                }
            },
            WindowEvent::DroppedFile(_path) => {},
            WindowEvent::HoveredFile(_path) => {},
            WindowEvent::HoveredFileCancelled => {},
            WindowEvent::Focused(focused) => {
                match &focus_path {
                    Some(path) => {
                        let strong_focus_path: Option<Vec<SharedNode>> = convert_vec_option_to_option_vec(path.iter().map(|weak| weak.upgrade()).collect());
                        if matches!(strong_focus_path, None) { return; }
                        let strong_focus_path = strong_focus_path.unwrap();
                        if strong_focus_path.len() == 0 { return; }

                        let focus_event = NodeEvent {
                            target: strong_focus_path.last().unwrap().clone(),
                            path: strong_focus_path.clone(),
                            event: if focused { events::InnerEvent::Focus } else { events::InnerEvent::Blur }
                        };
                        // strong_focus_path.last().unwrap().write().unwrap().on_event(&focus_event);
                        run_single_event_handlers(strong_focus_path.last().unwrap().clone(), focus_event);

                        let focus_event = NodeEvent {
                            target: strong_focus_path.last().unwrap().clone(),
                            path: strong_focus_path.clone(),
                            event: if focused { events::InnerEvent::FocusIn } else { events::InnerEvent::FocusOut }
                        };

                        run_event_handlers(strong_focus_path, focus_event);
                        window.request_redraw();
                    },
                    None => {}
                };
            },
            WindowEvent::ModifiersChanged(new_modifiers) => { modifiers = new_modifiers; },
            WindowEvent::KeyboardInput { device_id: _, event: _, is_synthetic: _ } => {},
            WindowEvent::MouseInput { device_id, state, button, .. } => {
                let mouse_value = mouse_values.get(&device_id);
                let mut mouse_value = match mouse_value {
                    Some(mouse_value) => mouse_value.clone(),
                    None => { return; } // Mouse move should be fired first
                };
                mouse_value.update_buttons(button, state);

                let location = mouse_value.last_location;
                let path = get_element_at(&root, &context, location);

                match path {
                    Some(path) => {
                        let target_layout = context.node_layout.get(path.last().unwrap());
                        let target_layout = match target_layout {
                            Some(target_layout) => target_layout,
                            None => { return; }
                        };
                        let target_layout = context.taffy.layout(target_layout.to_owned()).unwrap();
                        let mevent = MouseEvent {
                            button: Some(button),
                            buttons: mouse_value.buttons,
                            client: location,
                            movement: Location::new(0., 0.),
                            device: device_id,
                            modifiers,
                            offset: location - target_layout.location.into()
                        };
                        let event = NodeEvent {
                            target: path.last().unwrap().clone(),
                            path: path.clone(),
                            event: match state {
                                winit::event::ElementState::Pressed => events::InnerEvent::MouseDown(mevent),
                                winit::event::ElementState::Released => events::InnerEvent::MouseUp(mevent)
                            }
                        };

                        window.request_redraw();
                        run_event_handlers(path, event);
                    },
                    None => {}
                }
            },
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::Resized(size) => {
                let width: NonZeroU32 = NonZeroU32::new(size.width).unwrap();
                let height: NonZeroU32 = NonZeroU32::new(size.height).unwrap();
                surface.resize(&buffer_context, width, height);
                let mut groot = entry.root.write().unwrap();
                // let scale_factor = window.scale_factor();
                groot.resize(size.width as f32, size.height as f32);
                drop(groot);
                window.request_redraw();
                context.scale_factor = window.scale_factor() as f32;
                should_recompute = true;
            },
            WindowEvent::RedrawRequested => {
                if should_recompute {
                    update_taffynode_children(&root, &mut context);
                    let src_nodes = context.node_layout.values().map(|v| v.to_owned()).collect::<Vec<_>>();
                    context.node_layout.remove_expired();
                    let dst_nodes = context.node_layout.values().map(|v| v.to_owned()).collect::<Vec<_>>();
                    for src_node in src_nodes {
                        if !dst_nodes.contains(&src_node) {
                            context.taffy.remove(src_node).unwrap();
                            dbg!("Removed node", src_node);
                        }
                    }
                    prepare_render_recursively(&root, &mut context);
                    for (node, taffy_node) in context.node_layout.iter() {
                        let node = node.read().unwrap();
                        let node_style = node.style();
                        context.taffy.set_style(*taffy_node, node_style.layout.to_owned()).unwrap();
                    }
                    let size = window.inner_size();
                    let size = Size { width: AvailableSpace::Definite(size.width as f32), height: AvailableSpace::Definite(size.height as f32) };
                    let RenderContext { taffy, node_layout, canvas, scale_factor, .. } = &mut context;
                    let mut measure_context = MeasureContext { canvas, scale_factor: *scale_factor };
                    taffy.compute_layout_with_measure(
                        *node_layout.get(&root).unwrap(),
                        size,
                        |known_dimensions, available_space, _node_id, node_context| {
                            match node_context {
                                Some(node) => {
                                    match node.upgrade() {
                                        Some(node) => {
                                            node.write().unwrap().measure(&mut measure_context, known_dimensions, available_space)
                                        },
                                        None => Size::ZERO
                                    }
                                },
                                None => Size::ZERO
                            }
                        },
                    ).unwrap();
                    should_recompute = false;
                    // Additional optimizations could be done here
                    // - When setting styles, check that the styles aren't the same (taffy doesn't do that and instead always mark it as dirty)
                    // - taffy seems to always recompute (maybe internally checks dirtyness, I didn't look into it that much)
                    // - the weakmap dance (src_nodes, dst_nodes) could be avoided by changing the weakmap used
                    //   (weakmap removes keys when you attempt to read them, we could change it so that we could iterate on them and remove them in one go)
                    // could perhaps be a significant boost regarding memory usage (and performance) during large layout changes
                    // dbg!("recomputed");
                }
                // Clear the render queue
                while let Ok(_) = entry.render.try_recv() {}
                render(&buffer_context, &surface, &window, &mut context, &root);
            }
            _ => {}
        },
        Event::NewEvents(_) => {
            // if let Some(monitor) = window.current_monitor() {
            //     if let Some(refresh_rate) = monitor.refresh_rate_millihertz() {
                    // dbg!(refresh_rate);
                    // some leeway before vsync
                    // target.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(1000 / refresh_rate as u64 - 100/refresh_rate as u64)));
                    if let Ok(_) = entry.render.try_recv() {
                        window.request_redraw();
                    }
            //     }
            // }
        },
        // In the future, window should be created after resuming from suspend (for android support)
        _ => {}
    }).unwrap();
}

/// I have no idea if there's a better way to do this in rust...
/// Found via ChatGPT (the only piece of code by chatgpt itself in this whole project as of now)
fn convert_vec_option_to_option_vec<T>(vec: Vec<Option<T>>) -> Option<Vec<T>> {
    vec.into_iter().collect::<Option<Vec<T>>>()
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
    context: &mut RenderContext,
    root_node: &SharedNode
) {
    let size = window.inner_size();
    context.canvas.reset();
    context.canvas.set_size(size.width, size.height, window.scale_factor() as f32);
    context.canvas.clear_rect(0, 0, size.width, size.height, Color::black());

    render_recursively(root_node, context);

    context.canvas.flush();
    window.pre_present_notify();
    surface.swap_buffers(buffer_context).expect("Could not swap buffers");
}