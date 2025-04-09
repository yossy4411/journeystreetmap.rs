mod map;

use crate::map::JourneyMapViewer;
use iced::Element;
use iced::widget::{row, column, button};

fn main() {
    iced::run("A cool counter", Application::update, Application::view).expect("Failed to run the application");
}

#[derive(Debug, Clone)]
enum Message {
    OnButtonClick,
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
    pub fn update(&mut self, _message: Message) {
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
            row![
                w,
                column![
                    "Hello World!",
                    button("ボタン牡丹ぼたん").on_press(Message::OnButtonClick)
                ],
            ].spacing(8),

        ].into()
    }
}