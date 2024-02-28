use std::sync::{Arc, mpsc, Mutex};
use mangui::nodes::layout::Layout;
use mangui::{MainEntry, SharedNode};
use mangui::femtovg::Paint;
use mangui::nodes::text::Text;
use mangui::nodes::{Style, TaffyStyle, ToShared};
use mangui::taffy::{AlignItems, FlexDirection, JustifyContent, LengthPercentage, Rect};
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
        
        let mainview_container = Layout::default()
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
        let i = 2;
        uno!(gap-1 flex p-5px mt-1 mb-2 ml-[i] overflow-hidden);
        let title = Text::new("Mangades".to_owned(), TEXT_LARGE)
            .style(Style {
                layout: TaffyStyle {
                    padding: Rect { left: LengthPercentage::Length(10.), right: LengthPercentage::Length(10.), top: LengthPercentage::Length(10.), bottom: LengthPercentage::Length(10.) },
                    ..Default::default()
                },
                text_fill: Some(Paint::color(*tokens::WHITE)),
                ..Default::default()
            })
            .to_shared();
        
        append(&mainview_container, &title);
        detach(&loading_container);
        append(&groot_clone, &mainview_container);
        
        tx.send(()).unwrap();
    });

    mangui::run_event_loop(MainEntry {
        root: groot.clone(),
        render: rx
    });
}
