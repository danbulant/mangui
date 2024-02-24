use rusalka_macro::make_component;
use std::default::Default;
use mangui::{femtovg::{ImageFlags, Color, Paint}, cosmic_text::Metrics, nodes::{layout::Layout, Style}, nodes::text::Text, nodes::image::Image, taffy::prelude::Size};
use mangui::nodes::TaffyStyle;
use mangui::taffy::Display::Block;
use mangui::taffy::{FlexDirection, LengthPercentage, Rect};

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
                radius: if $test_ { attrs.radius } else { 0. }
            }
            @layout {
                @text {
                    text: String::from("Hello, World 🌎! And there's more text in here, as a single line"),
                    metrics: Metrics::new(20., 25.),
                    paint: Paint::color(Color::rgb(0, 255, 0)),
                    style: Style {
                        layout: TaffyStyle {
                            padding: Rect {
                                top: LengthPercentage::Length(10.),
                                left: LengthPercentage::Length(10.),
                                right: LengthPercentage::Length(10.),
                                bottom: LengthPercentage::Length(10.)
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }
                @image {
                    style: Style {
                        layout: TaffyStyle {
                            min_size: Size {
                                width: mangui::taffy::style::Dimension::Length(width),
                                height: mangui::taffy::style::Dimension::Length(height)
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    image: mangui::nodes::image::ImageLoad::LoadFile(imgpath, imgflags),
                    radius: 5.,
                    ..Default::default()
                }

                style: Style {
                    layout: TaffyStyle {
                        display: Block,
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
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