use rusalka_macro::make_component;
use std::default::Default;
use mangui::{SharedNode, nodes::Style, taffy::prelude::Size, femtovg::{Paint, Color}};

use rusalka::nodes::primitives::{Rectangle, RectangleAttributes};

make_component!(
    ComponentDemo,
    Logic {
        let test = false;
    }
    Component {
        @Rectangle {
            // style: Style {
            //     layout: TaffyStyle {
            //         min_size: Size {
            //             width: Dimension::Points(50.),
            //             height: Dimension::Points(100.)
            //         },
            //         ..Default::default()
            //     },
            //     ..Default::default()
            // },
            // fill: Paint::color(Color::rgb(0, 0, 255)),
            // radius: 5.,
            // ..Default::default()
        }
    }
);
