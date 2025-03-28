mod map;

use fastanvil::Region;
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use softbuffer::{Context, Surface};
use crate::map::{JourneyMapViewer};
use std::collections::HashMap;
use std::fs::File;
use std::num::NonZeroU32;
use std::rc::Rc;
use fltk::prelude::{GroupExt, InputExt, MenuExt, WidgetBase, WidgetExt};
use iced::Element;
use iced::widget::{text, Canvas, Column};
use iced_tiny_skia::core::{Image, Widget};
use iced_tiny_skia::Renderer;
use rusttype::{point, Font, OutlineBuilder, Scale};
use tiny_skia::{Color, FillRule, Path, PathBuilder, Pixmap, Point, Rect, Stroke, Transform};

fn main() {
    iced::run("A cool counter", Application::update, Application::view).expect("Failed to run the application");
}

#[derive(Debug, Clone)]
enum Message {

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
    journey_map_viewer: JourneyMapViewer,
}

impl Default for Application {
    fn default() -> Self {
        let mut viewer = JourneyMapViewer::new();
        viewer.load_images().expect("Failed to load images");
        Self {
            journey_map_viewer: viewer,
        }
    }
}

impl Application {
    pub fn update(&mut self, message: Message) {
        // ここにアプリケーションの状態を更新する処理を書く
    }

    fn view(&self) -> Element<Message> {
        // Column::new().push(journey_map_viewer()).push(text!("Hello World!")).into()
        iced::widget::column![
            "JourneyMapのマップをアプリで表示する試み",
            Canvas::new(&self.journey_map_viewer),
        ].into()
    }
}