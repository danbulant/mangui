use std::ops::{AddAssign, Add, SubAssign, Sub};

use taffy::{prelude::Size, style::Dimension, geometry::Point};
use winit::event::ElementState;
pub use winit::event::{TouchPhase, MouseScrollDelta, DeviceId, ModifiersState, VirtualKeyCode, ScanCode, MouseButton};

use crate::SharedNode;

#[derive(Clone, Debug)]
pub struct NodeEvent {
    /// Target node of event.
    pub target: SharedNode,
    /// Path to the target - target will be the last item in the path.
    pub path: Vec<SharedNode>,
    /// Actual event
    pub event: InnerEvent
}

/// Different event types that can be sent to a node.
#[derive(Clone, Debug, PartialEq)]
pub enum InnerEvent {
    Wheel {
        phase: TouchPhase,
        delta: MouseScrollDelta,
        mouse: MouseEvent
    },
    /// Mouse enter event is fired when the mouse enters the target node or any of its children, and bubbles
    MouseEnter(MouseEvent),
    /// Mouse over event is fired when the mouse enters the target node, but not its children, and does not bubble
    MouseOver(MouseEvent),
    /// Mouse leave event is fired when the mouse leaves the target node or any of its children, and bubbles
    MouseLeave(MouseEvent),
    /// Mouse out event is fired when the mouse leaves the target node, but not its children, and does not bubble
    MouseOut(MouseEvent),
    /// Mouse moved
    MouseMove(MouseEvent),
    /// Mouse button pressed
    MouseDown(MouseEvent),
    /// Mouse button released
    MouseUp(MouseEvent),
    /// Mouse button clicked - fired after clicking and releasing a button without changing the target element
    Click(MouseEvent),
    /// Mouse secondary button (usually right) clicked
    ContextMenu(MouseEvent),
    /// Mouse tertiary button (usually middle or scroll wheel) clicked
    AuxClick(MouseEvent),
    /// Focus event is fired only on the target node and does not bubble
    Focus,
    /// Blur event is fired only on the target node and does not bubble
    Blur,
    /// Same as [InnerEvent::Focus] but bubbles
    FocusIn,
    /// Same as [InnerEvent::Blur] but bubbles
    FocusOut,
    /// Key pressed
    KeyDown(KeyboardEvent),
    /// Key released
    KeyUp(KeyboardEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyboardEvent {
    /// Logical location ("it's effect") of the key
    pub key: Option<VirtualKeyCode>,
    /// Physical location of the key
    pub code: ScanCode,

    // altKey: bool,
    // ctrlKey: bool,
    // metaKey: bool,
    // shiftKey: bool,
    /// modifier keys pressed (alt, ctrl, shift or meta/logo/windows)
    pub modifiers: ModifiersState,

    // repeat: bool,
    // char_code: u32,
    // key_code: u32,
    // which: u32,
    /// DeviceId as passed by winit. An opaque, only useful when comparing with other events.
    pub device: DeviceId
}

#[derive(Clone, Debug, PartialEq)]
pub struct MouseEvent {
    /// The button which fired the event (if any)
    pub button: Option<MouseButton>,
    /// The buttons which are currently pressed as a bitmask.
    /// Use [MouseEvent::button_to_buttons] to convert a single button to a bitmask.
    pub buttons: u8,

    // altKey: bool,
    // ctrlKey: bool,
    // metaKey: bool,
    // shiftKey: bool,
    /// modifier keys pressed (alt, ctrl, shift or meta/logo/windows)
    pub modifiers: ModifiersState,

    /// The location of the mouse relative to window
    pub client: Location,

    /// The location of the mouse relative to last event
    pub movement: Location,

    /// The location of the mouse relative to the target node (not the current node!)
    /// For the first event, this will be 0, 0
    pub offset: Location,

    /// DeviceId as passed by winit. An opaque, only useful when comparing with other events.
    pub device: DeviceId
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Location {
    pub x: f32,
    pub y: f32
}

impl AddAssign for Location {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Location {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Location {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<(f64, f64)> for Location {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x: x as f32, y: y as f32 }
    }
}

impl From<Point<f32>> for Location {
    fn from(point: Point<f32>) -> Self {
        Self { x: point.x, y: point.y }
    }
}

impl Add for Location {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl SubAssign for Location {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub for Location {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl Into<(f32, f32)> for Location {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl Into<Size<Dimension>> for Location {
    fn into(self) -> Size<Dimension> {
        Size {
            width: Dimension::Points(self.x as f32),
            height: Dimension::Points(self.y as f32)
        }
    }
}

impl MouseEvent {
    /// Returns `true` if the shift key is pressed.
    pub fn shift(&self) -> bool {
        self.modifiers.intersects(ModifiersState::SHIFT)
    }
    /// Returns `true` if the control key is pressed.
    pub fn ctrl(&self) -> bool {
        self.modifiers.intersects(ModifiersState::CTRL)
    }
    /// Returns `true` if the alt key is pressed.
    pub fn alt(&self) -> bool {
        self.modifiers.intersects(ModifiersState::ALT)
    }
    /// Returns `true` if the logo key is pressed.
    pub fn logo(&self) -> bool {
        self.modifiers.intersects(ModifiersState::LOGO)
    }

    pub fn button_to_buttons(button: MouseButton) -> u8 {
        match button {
            MouseButton::Left => 1,
            MouseButton::Right => 2,
            MouseButton::Middle => 4,
            MouseButton::Other(n) => 1 << n
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub(crate) struct MouseValue {
    pub last_location: Location,
    pub buttons: u8
}

impl MouseValue {
    pub(crate) fn update_buttons(&mut self, button: MouseButton, state: ElementState) {
        let buttons = MouseEvent::button_to_buttons(button);
        match state {
            ElementState::Pressed => self.buttons |= buttons,
            ElementState::Released => self.buttons &= !buttons
        }
    }
}