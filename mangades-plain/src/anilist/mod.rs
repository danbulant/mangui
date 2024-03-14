use bytes::Bytes;
use serde::Deserialize;
use mangui::femtovg::ImageFlags;
use mangui::nodes::image::ImageLoad;

#[derive(Deserialize, Debug)]
struct GraphqlResponse<T> {
    pub(crate) data: T
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollectionData {
    #[serde(rename = "MediaListCollection")]
    pub(crate) media_list_collection: MediaListCollection
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollection {
    pub(crate) lists: Vec<MediaList>
}

#[derive(Deserialize, Debug)]
pub(crate) struct MediaList {
    pub(crate) name: String,
    #[serde(rename = "isCustomList")]
    pub(crate) is_custom_list: bool,
    pub(crate) status: String,
    #[serde(rename = "isSplitCompletedList")]
    pub(crate) is_split_completed_list: bool,
    pub(crate) entries: Vec<MediaListEntry>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MediaListEntry {
    pub(crate) status: String,
    pub(crate) progress: i32,
    #[serde(rename = "progressVolumes")]
    pub(crate) progress_volumes: i32,
    pub(crate) repeat: i32,
    pub(crate) priority: i32,
    pub(crate) private: bool,
    pub(crate) notes: Option<String>,
    pub(crate) score: f32,
    pub(crate) media: MediaEntry,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MediaEntry {
    pub(crate) id: i32,
    pub(crate) title: MediaTitle,
    pub(crate) status: String,
    pub(crate) chapters: Option<i32>,
    pub(crate) volumes: Option<i32>,
    #[serde(rename = "coverImage")]
    pub(crate) cover_image: CoverImage,
    #[serde(rename = "isAdult")]
    pub(crate) is_adult: bool,
    #[serde(rename = "isFavourite")]
    pub(crate) is_favourite: bool,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MediaTitle {
    pub(crate) romaji: String,
    pub(crate) english: Option<String>,
    pub(crate) native: String,
    #[serde(rename = "userPreferred")]
    pub(crate) user_preferred: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct CoverImage {
    pub(crate) large: String,
    pub(crate) medium: String,
    pub(crate) color: Option<String>,
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