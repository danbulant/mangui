use rusalka_macro::make_component;
use std::default::Default;
use mangui::{SharedNode, nodes::Style, taffy::prelude::Size, femtovg::{Paint, Color}, nodes::layout::Layout};

use rusalka::nodes::primitives::{Rectangle, RectangleAttributes};

make_component!(
    ComponentDemo,
    Logic {
        let radius = 5.;
    }
    Component {
        @layout {
            @Rectangle {
                radius,
                ..Default::default()
            }
            ..Default::default()
        }
    }
);
