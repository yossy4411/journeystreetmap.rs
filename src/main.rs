mod map;

use crate::map::JourneyMapViewerState;
use eframe::emath::vec2;
use eframe::Frame;
use egui::{Context, FontDefinitions, FontFamily, Pos2, Rect, ViewportBuilder};
use std::sync::Arc;

fn main() {
    let viewport = ViewportBuilder {
        title: Some("JourneyMap Viewer".to_string()),
        inner_size: Some(vec2(800.0, 600.0)),
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "JMViewer",
        options,
        Box::new(|cc: &eframe::CreationContext<'_>| {
            let mut fonts = FontDefinitions::default();

            fonts.font_data.insert(
                "NotoSansJP".to_owned(),
                Arc::new(egui::FontData::from_static(include_bytes!("../fonts/NotoSansJP-Regular.ttf"))),
            );

            fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "NotoSansJP".to_string());

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::<Application>::default())
        }),
    ).expect("Failed to run the application");
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

impl eframe::App for Application {

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("JourneyMapのマップをアプリで表示する試み");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("ここにマップが来る");
                        let painter = ui.painter();
                        // 画像を最後に描画する（グリッドの下に行かないように）
                        for ((rx, rz), img) in &self.journey_map_viewer_state.images {
                            let dest_x = rx * 512;
                            let dest_y = rz * 512;
                            let rect = Rect::from_min_max(Pos2::new(dest_x as f32, dest_y as f32), Pos2::new(dest_x as f32 + 512.0, dest_y as f32 + 512.0));
                            // painter.image(TextureId::Managed(0), rect, rect, Color32(img.color));
                            // なんだか、先に画像をテクスチャに落とし込んで、それをアドレスで指定して描画するらしい。すごくGPU感がある。
                        }

                        // todo: JourneyMapViewerの描画
                    });
                    ui.vertical(|ui| {
                        ui.add_space(10.0);
                        ui.label("Hello World!");
                        ui.add_space(10.0);
                        if ui.button("ボタン牡丹ぼたん").clicked() {
                            println!("Button clicked!");
                        }
                    });
                })
            })


        });
    }
}
