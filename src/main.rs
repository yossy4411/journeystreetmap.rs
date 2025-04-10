mod map;

use crate::map::{JourneyMapViewer, JourneyMapViewerState};
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
    journey_map_viewer_state: JourneyMapViewerState
}

impl Default for Application {
    fn default() -> Self {
        let mut jm_state = JourneyMapViewerState::default();
        jm_state.load_images().expect("Failed to load images from JourneyMapViewerState");
        Self {
            journey_map_viewer_state: jm_state,
        }
    }
}

impl Application {
    pub fn update(&mut self, _message: Message) {
        // ここにアプリケーションの状態を更新する処理を書く
    }

    fn view(&mut self) -> Element<Message> {
        // Column::new().push(journey_map_viewer()).push(text!("Hello World!")).into()
/*        let mut jm = JourneyMapViewer::default();
        jm.load_images().expect("Failed to load images");*/
        let mut w = JourneyMapViewer::new(&mut self.journey_map_viewer_state);
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