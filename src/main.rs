mod map;

use macroquad::prelude::*;
use egui_macroquad::egui;
use crate::map::JourneyMapViewerState;

#[macroquad::main("journeystreetmap")]
async fn main() {
    let mut state = JourneyMapViewerState::default();

    // 画像を読み込む
    state.load_images().expect("Failed to load images");

    // macroquadの初期化
    egui_macroquad::cfg(|egui_ctx| {
        // ウィンドウの影の設定
        let style = egui_ctx.style();
        let mut new_style = style.as_ref().clone();
        new_style.visuals.window_shadow.extrusion = 10.0;
        egui_ctx.set_style(new_style);

        // フォントの設定
        let mut font_definitions = egui::FontDefinitions::default();
        font_definitions.font_data.insert(
            "Noto Sans JP".to_string(),
            egui::FontData::from_static(include_bytes!("../fonts/NotoSansJP-Regular.ttf"))
        );
        font_definitions.families.insert(
            egui::FontFamily::Proportional,
            vec!["Noto Sans JP".to_string()]
        );

        egui_ctx.set_fonts(font_definitions);
    });

    let mut camera = Camera2D::default();

    loop {
        // 定義
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("JourneyStreetMap Editor").show(egui_ctx, |ui| {
                ui.label("これは egui のウィンドウです！");
                if ui.button("ボタン牡丹ぼたん").clicked() {
                    println!("ボタンが押されたよ！");
                }
            });
        });

        // macroquadの描画処理
        clear_background(LIGHTGRAY);

        // マウスの処理
        if is_mouse_button_down(MouseButton::Left) {
            // マウスが押下されているとき
            state.dragging(mouse_delta_position());
        }

        camera.target = state.camera_position();
        camera.zoom = vec2(1.0, 1.0);

        set_camera(&camera);

        // カメラへ描画
        for ((rx, rz), img) in state.images() {
            let dest_x = rx * 512;
            let dest_y = rz * 512;
            draw_texture(*img, dest_x as f32, dest_y as f32, WHITE);
        }
        draw_text("Hello macroquad!", 20.0, 40.0, 30.0, DARKGRAY);


        // 描画更新
        egui_macroquad::draw();

        // next
        next_frame().await;
    }
}
