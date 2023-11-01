use rusalka_macro::make_component;
use std::default::Default;
use mangui::{SharedNode, nodes::Style, taffy::prelude::Size, femtovg::{Paint, Color}, nodes::layout::Layout};

use rusalka::nodes::primitives::{Rectangle, RectangleAttributes};

make_component!(
    ComponentDemo,
    MainLogic {
        let _radius = attrs.radius;
    }
    Attributes {
        radius: f32
    }
    Variables {
        test_: bool = false
    }
    Reactive {
        dbg!($test_);
    }
    Component {
        @layout {
            @Rectangle {
                radius: if $test_ { attrs.radius } else { 0. },
                ..Default::default()
            }
            $|event| {
                match event.event {
                    mangui::events::InnerEvent::MouseDown(_) => {
                        $test_ = true;
                        println!("Mouse down");
                    },
                    mangui::events::InnerEvent::MouseUp(_) => {
                        $test_ = false;
                        println!("Mouse up");
                    },
                    _ => {}
                }
            }
            ..Default::default()
        }
    }
);