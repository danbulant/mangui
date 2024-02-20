use rusalka_macro::make_component;
use std::default::Default;
use mangui::{femtovg::ImageFlags, nodes::{layout::Layout, Style}, nodes::image::Image, taffy::prelude::Size};

use rusalka::nodes::primitives::{Rectangle, RectangleAttributes, PartialRectangleAttributes};

make_component!(
    ComponentDemo,
    MainLogic {
        let _radius = attrs.radius;
        let imgpath = std::path::PathBuf::from("./demo/large/bx117324-97mHyfJGwpBq.jpg");
        let imgflags = ImageFlags::empty();
        let width = 230.;
        let height = 325.;
    }
    Attributes {
        radius: f32
    }
    Variables {
        test_: bool = false
    }
    Reactive {
        // dbg!($test_);
    }
    Component {
        @layout {
            @Rectangle {
                radius: if $test_ { attrs.radius } else { 0. },
                ..Default::default()
            }
            @image {
                style: Style {
                    layout: mangui::nodes::TaffyStyle {
                        min_size: Size {
                            width: mangui::taffy::style::Dimension::Points(width),
                            height: mangui::taffy::style::Dimension::Points(height)
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                image: mangui::nodes::image::ImageLoad::LoadFile(imgpath, imgflags),
                width,
                height,
                radius: 5.,
                events: Default::default(),
                parent: None
            }
            $|event| {
                match event.event {
                    mangui::events::InnerEvent::MouseDown(_) => {
                        $test_ = true;
                    },
                    mangui::events::InnerEvent::MouseUp(_) => {
                        $test_ = false;
                    },
                    _ => {}
                }
            }
            ..Default::default()
        }
    }
);