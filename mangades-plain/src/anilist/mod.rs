use bytes::Bytes;
use serde::Deserialize;
use mangui::femtovg::ImageFlags;
use mangui::nodes::image::ImageLoad;

#[derive(Deserialize, Debug)]
struct GraphqlResponse<T> {
    data: T
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollectionData {
    #[serde(rename = "MediaListCollection")]
    media_list_collection: MediaListCollection
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollection {
    lists: Vec<MediaList>
}

#[derive(Deserialize, Debug)]
struct MediaList {
    name: String,
    #[serde(rename = "isCustomList")]
    is_custom_list: bool,
    status: String,
    #[serde(rename = "isSplitCompletedList")]
    is_split_completed_list: bool,
    entries: Vec<MediaListEntry>,
}

#[derive(Deserialize, Debug)]
struct MediaListEntry {
    status: String,
    progress: i32,
    #[serde(rename = "progressVolumes")]
    progress_volumes: i32,
    repeat: i32,
    priority: i32,
    private: bool,
    notes: Option<String>,
    score: f32,
    media: MediaEntry,
}

#[derive(Deserialize, Debug)]
struct MediaEntry {
    id: i32,
    title: MediaTitle,
    status: String,
    chapters: Option<i32>,
    volumes: Option<i32>,
    #[serde(rename = "coverImage")]
    cover_image: CoverImage,
    #[serde(rename = "isAdult")]
    is_adult: bool,
    #[serde(rename = "isFavourite")]
    is_favourite: bool,
}

#[derive(Deserialize, Debug)]
struct MediaTitle {
    romaji: String,
    english: Option<String>,
    native: String,
    #[serde(rename = "userPreferred")]
    user_preferred: String,
}

#[derive(Deserialize, Debug)]
struct CoverImage {
    large: String,
    medium: String,
    color: Option<String>,
}

// pub fn load_demo() -> MediaListCollection {
//     // For demo purposes, load file in demo/list.json
//     let json = include_str!("../../demo/list.json");
//     let response: GraphqlResponse<MediaListCollectionData> = serde_json::from_str(json).unwrap();
//     response.data.media_list_collection
// }

pub async fn load_demo_async() -> MediaListCollection {
    let json = tokio::fs::read_to_string("demo/list.json").await.unwrap();
    let response: GraphqlResponse<MediaListCollectionData> = serde_json::from_str(&json).unwrap();
    response.data.media_list_collection
}

pub async fn load_demo_image(url: String) -> ImageLoad {
    let last_part = url.split('/').last().unwrap();
    let path = format!("demo/{}", last_part);
    let bytes = tokio::fs::read(path).await.unwrap();
    ImageLoad::LoadVec(bytes, ImageFlags::empty())
}

// pub async fn load_data(appref: Weak<MainWindow>) {
//     let data = load_demo();

//     let urls = data.lists.iter().flat_map(|list| {
//         list.entries.iter().map(|entry| {
//             entry.media.cover_image.medium.clone()
//         })
//     }).collect::<Vec<String>>();

//     let mut images = futures::future::join_all(urls.into_iter().map(|url| {
//         load_image(url)
//     })).await;
//     images.reverse();

//     slint::invoke_from_event_loop(move || {
//         let lists = Rc::new(VecModel::default());

//         for list in data.lists {
//             let entries: Rc<VecModel<AnilistItem>> = Rc::new(VecModel::default());
//             for entry in list.entries {
//                 let image = images.pop().unwrap().unwrap();
//                 let image = Image::from_rgba8(image);
//                 let item = AnilistItem {
//                     id: entry.media.id,
//                     title: entry.media.title.user_preferred.into(),
//                     image
//                 };
//                 entries.push(item);
//             }
//             let list = AnilistList {
//                 name: list.name.into(),
//                 items: ModelRc::from(entries)
//             };
//             lists.push(list);
//         }
//         let app = appref.upgrade().unwrap();
//         app.set_lists(ModelRc::from(lists));
//         app.set_loading(false);
//     }).expect("Load data into UI");
// }

async fn load_image(url: String) -> Result<Bytes, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    Ok(bytes)
}