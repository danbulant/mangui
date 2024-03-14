use std::sync::{Arc, mpsc, Mutex};
use mangui::nodes::layout::Layout;
use mangui::{MainEntry, SharedNode};
use mangui::dpi::PhysicalPosition;
use mangui::events::InnerEvent;
use mangui::femtovg::{ImageFlags, Paint};
use mangui::nodes::text::Text;
use mangui::nodes::{Style, TaffyStyle, ToShared};
use mangui::nodes::image::{Image, ImageLoad};
use mangui::taffy::{AlignItems, FlexDirection, JustifyContent, LengthPercentage, LengthPercentageAuto, Point, Rect};
use uno_gen::uno;
use crate::anilist::load_demo_async;
use crate::tokens::TEXT_LARGE;
use crate::utils::{append, detach};

mod anilist;
mod utils;
mod tokens;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(tx);
    let root = Layout::default();
    let groot: SharedNode = Arc::new(Mutex::new(root));
    
    let loading_container = Layout::default()
        .style(Style {
            layout: TaffyStyle {
                flex_grow: 1.,
                justify_content: JustifyContent::Center.into(),
                align_items: AlignItems::Center.into(),
                ..Default::default()
            },
            background: Some(Paint::color(*tokens::BACKGROUND)),
            ..Default::default()
        })
        .to_shared();
    let loading_text = Text::new("Loading...".to_owned(), TEXT_LARGE)
        .style(Style {
            text_fill: Some(Paint::color(*tokens::WHITE)),
            ..Default::default()
        })
        .to_shared();
    append(&groot, &{ loading_container.clone() });
    append(&loading_container, &{ loading_text.clone() });

    let groot_clone = groot.clone();
    tokio::spawn(async move {
        let data = load_demo_async().await;

        let mut mainview_container = Layout::default()
            .style(Style {
                layout: TaffyStyle {
                    flex_grow: 1.,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                background: Some(Paint::color(*tokens::BACKGROUND)),
                ..Default::default()
            })
            .to_arcmutex();
        mainview_container.lock().unwrap().events.add_handler(Box::new({
            let mainview_container = mainview_container.clone();
            move |event| {
                if let InnerEvent::Wheel { delta, .. } = event.event {
                    let delta = match delta {
                        mangui::events::MouseScrollDelta::LineDelta(_, y) => y * 30f32,
                        mangui::events::MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y as f32,
                    };
                    let mut layout = mainview_container.lock().unwrap();
                    // layout.style.layout.scroll.y -= delta * 10.;
                    layout.style.scroll_y -= delta;
                    // cap scroll_y to 0
                    if layout.style.scroll_y < 0. {
                        layout.style.scroll_y = 0.;
                    }
                    println!("scroll_y: {}", layout.style.scroll_y);
                }
            }
        }));
        let i = LengthPercentageAuto::Length(5.);
        let title = Text::new("Mangades".to_owned(), TEXT_LARGE)
            .style(Style {
                text_fill: Some(Paint::color(*tokens::WHITE)),
                ..uno!(p-10)
            })
            .to_shared();
        append(&{ mainview_container.clone() }, &title);
        
        for list in data.lists {
            let list_container = Layout::default()
                .style(Style {
                    layout: TaffyStyle {
                        flex_grow: 1.,
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    background: Some(Paint::color(*tokens::BACKGROUND)),
                    ..Default::default()
                })
                .to_shared();
            let list_title = Text::new(list.name, TEXT_LARGE)
                .style(Style {
                    text_fill: Some(Paint::color(*tokens::WHITE)),
                    ..uno!(p-10)
                })
                .to_shared();
            append(&{ mainview_container.clone() }, &list_container);
            append(&list_container, &list_title);

            for entry in list.entries {
                let entry_container = Layout::default()
                    .style(Style {
                        layout: TaffyStyle {
                            flex_grow: 1.,
                            flex_direction: FlexDirection::Row,
                            ..Default::default()
                        },
                        background: Some(Paint::color(*tokens::BACKGROUND)),
                        ..Default::default()
                    })
                    .to_shared();
                // image loading disabled for speed
                // let addr = entry.media.cover_image.large;
                // // use only last two parts from url, which is a folder and a file (either medium/something.jpg or large/something.jpg)
                // let addr = addr.split('/').collect::<Vec<&str>>().into_iter().rev().take(2).collect::<Vec<&str>>().into_iter().rev().collect::<Vec<&str>>().join("/");
                // let addr = addr.replace("medium", "large").replace("small", "medium");
                // let addr = format!("demo/{}", addr);
                // dbg!(&addr);
                // let image = Image::new(
                //     ImageLoad::LoadFile(addr.parse().unwrap(),
                //     ImageFlags::empty())
                // )
                //     .style(Style {
                //         ..Default::default()
                //     })
                //     .to_shared();
                let title = Text::new(entry.media.title.user_preferred, TEXT_LARGE)
                    .style(Style {
                        text_fill: Some(Paint::color(*tokens::WHITE)),
                        ..uno!(p-10)
                    })
                    .to_shared();
                append(&list_container, &entry_container);
                // append(&entry_container, &image);
                append(&entry_container, &title);
            }
        }

        detach(&loading_container);
        append(&groot_clone, &{ mainview_container.clone() });

        tx.send(()).unwrap();
    });

    mangui::run_event_loop(MainEntry {
        root: groot.clone(),
        render: rx
    }).unwrap();
}
