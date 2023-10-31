use rusalka_macro::make_component;
use std::default::Default;
use mangui::{SharedNode, nodes::Style, taffy::prelude::Size, femtovg::{Paint, Color}, nodes::layout::Layout};

use rusalka::nodes::primitives::{Rectangle, RectangleAttributes};

make_component!(
    ComponentDemo,
    MainLogic {
        let radius = attrs.radius;
    }
    Attributes {
        radius: f32
    }
    Variables {
        test: bool = false
    }
    Component {
        @layout {
            @Rectangle {
                radius: if $test { radius } else { 0. },
                ..Default::default()
            }
            $|event| {
                match event.event {
                    mangui::events::InnerEvent::MouseDown(_) => {
                        $test = true;
                    },
                    mangui::events::InnerEvent::MouseUp(_) => {
                        $test = false;
                    },
                    _ => {}
                }
            }
            ..Default::default()
        }
    }
);
