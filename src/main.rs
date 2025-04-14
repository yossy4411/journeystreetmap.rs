mod map;

use std::collections::HashMap;
use std::sync::Arc;
use macroquad::prelude::*;
use egui_macroquad::egui;
use tokio::runtime;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use crate::map::{JourneyMapViewerState};

#[macroquad::main("journeystreetmap")]
async fn main() {
    let mut state = JourneyMapViewerState::default();

    let mut images = Arc::new(Mutex::new(HashMap::new()));

    // 画像を読み込む
    let runtime = runtime::Builder::new_multi_thread().worker_threads(4).enable_all().build().unwrap();

    let images_clone = images.clone();

    let handle = runtime.spawn(async {
        map::load_images(images_clone).await.expect("Failed to load images");
    });


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
    camera.zoom = vec2(1.0 / screen_width(), -1.0 / screen_height());

    let mut cursor_in_ui = false;
    let mut clicked = false;

    loop {
        // eguiの定義
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("JourneyStreetMap Editor").show(egui_ctx, |ui| {

                if state.editing_mode() == map::EditingMode::View {
                    ui.label("編集モード: 表示");
                } else {
                    let mode_str = match state.editing_mode() {
                        map::EditingMode::Insert => "挿入",
                        map::EditingMode::Delete => "削除",
                        map::EditingMode::Select => "選択",
                        map::EditingMode::View => "表示",
                    };
                    let type_str = match state.editing_type() {
                        map::EditingType::Stroke => "線",
                        map::EditingType::Fill => "塗り",
                        map::EditingType::Poi => "場所",
                    };
                    ui.label(format!("編集モード: {}（{}）", mode_str, type_str));
                }


                if ui.button("ボタン牡丹ぼたん").clicked() {
                    println!("ボタンが押されたよ！");
                }
            });
            cursor_in_ui = egui_ctx.is_pointer_over_area();
        });

        // macroquadの描画処理
        clear_background(LIGHTGRAY);

        // マウスの処理
        if !cursor_in_ui && is_mouse_button_down(MouseButton::Left) {
            // マウスが押下されたとき
            clicked = true;
        }
        if is_mouse_button_released(MouseButton::Left) {
            // マウスが離されたとき
            clicked = false;
        }

        let delta = mouse_delta_position();
        if clicked {
            camera.target += delta / vec2(camera.zoom.x, -camera.zoom.y);
        }

        let mouse_position = mouse_position().into();
        // ホイールの処理
        if !cursor_in_ui {
            let before_zoom = camera.screen_to_world(mouse_position);

            let delta = mouse_wheel().1;
            if delta != 0.0 {
                let factor = 1.3f32.powf(delta);
                camera.zoom *= factor;
                let after_zoom = camera.screen_to_world(mouse_position);
                camera.target += before_zoom - after_zoom;
            }
        }

        // キーボードの処理
        // 編集モードの切り替え
        if is_key_pressed(KeyCode::I) {
            state.set_editing_mode(map::EditingMode::Insert);
        }
        if is_key_pressed(KeyCode::D) {
            state.set_editing_mode(map::EditingMode::Delete);
        }
        if is_key_pressed(KeyCode::S) {
            state.set_editing_mode(map::EditingMode::Select);
        }
        if is_key_pressed(KeyCode::V) {
            state.set_editing_mode(map::EditingMode::View);
        }

        if is_key_pressed(KeyCode::E) {
            // 編集の種別を切り替え
            state.toggle_editing_type();
        }

        set_camera(&camera);

        // カメラへ描画
        {
            let images = images.lock().await;
            for ((rx, rz), img) in images.iter() {
                let dest_x = rx * 512;
                let dest_y = rz * 512;
                draw_texture(*img, dest_x as f32, dest_y as f32, WHITE);
            }
        }

        set_default_camera();
        draw_text("Hello macroquad!", 20.0, 40.0, 30.0, DARKGRAY);


        // 描画更新
        egui_macroquad::draw();

        // next
        next_frame().await;
    }
}
