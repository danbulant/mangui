use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::sync::{Arc, RwLock, Weak};

use events::{Location, MouseValue, NodeEvent, MouseEvent};
use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color};
use glutin::surface::Surface;
use glutin::{context::PossiblyCurrentContext, display::Display};
use glutin_winit::DisplayBuilder;
use nodes::get_element_at;
use raw_window_handle::HasRawWindowHandle;
use winit::event::{Event, WindowEvent, ModifiersState, DeviceId};
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
pub mod events;
pub use taffy;
pub use femtovg;

pub type CurrentRenderer = OpenGl;
pub type SharedNode = Arc<RwLock<dyn Node>>;
type WeakNode = Weak<RwLock<dyn Node>>;
type NodePtr = Option<Vec<WeakNode>>;
type NodeLayoutMap = PtrWeakKeyHashMap<Weak<RwLock<dyn Node>>, taffy::node::Node>;

pub fn run_event_loop(root_node: SharedNode) -> ! {
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

    let mut modifiers = ModifiersState::default();
    let focus_path: Option<Vec<WeakNode>> = None;
    let mut mouse_values: HashMap<DeviceId, MouseValue> = HashMap::new();

    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::MouseWheel { device_id, delta, phase, .. } => {},
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

                    for node in path.iter().rev() {
                        node.write().unwrap().on_event(&event);
                    }
                }
            },
            WindowEvent::DroppedFile(path) => {},
            WindowEvent::HoveredFile(path) => {},
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
                        strong_focus_path.last().unwrap().write().unwrap().on_event(&focus_event);

                        let focus_event = NodeEvent {
                            target: strong_focus_path.last().unwrap().clone(),
                            path: strong_focus_path.clone(),
                            event: if focused { events::InnerEvent::FocusIn } else { events::InnerEvent::FocusOut }
                        };

                        for node in strong_focus_path.iter().rev() {
                            node.write().unwrap().on_event(&focus_event);
                        }
                    },
                    None => {}
                };
            },
            WindowEvent::ModifiersChanged(new_modifiers) => { modifiers = new_modifiers; },
            WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {},
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
                        let mevent = MouseEvent {
                            button: Some(button),
                            buttons: mouse_value.buttons,
                            client: location,
                            movement: Location::new(0., 0.),
                            device: device_id,
                            modifiers,
                            offset: Location::new(0., 0.)
                        };
                        let event = NodeEvent {
                            target: path.last().unwrap().clone(),
                            path: path.clone(),
                            event: match state {
                                winit::event::ElementState::Pressed => events::InnerEvent::MouseDown(mevent),
                                winit::event::ElementState::Released => events::InnerEvent::MouseUp(mevent)
                            }
                        };

                        for node in path.iter().rev() {
                            node.write().unwrap().on_event(&event);
                        }
                    },
                    None => {}
                }
            },
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
        // In the future, window should be created after resuming from suspend (for android support)
        _ => {}
    })
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
    surface.swap_buffers(buffer_context).expect("Could not swap buffers");
}