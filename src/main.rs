mod map;
mod map_wgpu;

use iced::widget::button;
use iced::Element;
use tiny_skia::{Path, Point};
use crate::map::JourneyMapViewer;

fn main() {
    iced::run("A cool counter", Application::update, Application::view).expect("Failed to run the application");
}

#[derive(Debug, Clone)]
enum Message {
    OnButtonClick,
}

// 編集したものを保存するenum
#[derive(Debug)]
enum EditResult {
    StrokePath(Path),
    FillPath(Path),
    PoiPoint(Point),
}

#[derive(Debug, Clone)]
/// 画像の状態を管理する構造体
struct ImageState {
    zoom: f32,
    zoom_factor: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl Default for ImageState {
    fn default() -> Self {
        ImageState {
            zoom: 1.0,
            zoom_factor: 1.25,
            offset_x: 0.0,
            offset_y: 0.0,
            dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
    }
}

struct Application {
    // journey_map_viewer: Arc<JourneyMapViewer>,
}

impl Default for Application {
    fn default() -> Self {
/*        let mut viewer = JourneyMapViewer::default();
        viewer.load_images().expect("Failed to load images");
        Self {
            journey_map_viewer: Arc::new(viewer),
        }*/
        Self{}
    }
}

impl Application {
    pub fn update(&mut self, message: Message) {
        // ここにアプリケーションの状態を更新する処理を書く
    }

    fn view(&self) -> Element<Message> {
        // Column::new().push(journey_map_viewer()).push(text!("Hello World!")).into()
/*        let mut jm = JourneyMapViewer::default();
        jm.load_images().expect("Failed to load images");*/
        let mut w = JourneyMapViewer::default();
        w.load_images().expect("Failed to load images");
        iced::widget::column![
            "JourneyMapのマップをアプリで表示する試み",
            // Canvas::new(jm),
            Element::new(w),
            button("aaaa").on_press(Message::OnButtonClick)
        ].into()
    }
}