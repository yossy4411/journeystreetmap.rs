mod map;

use macroquad::prelude::*;
use egui_macroquad::egui;
use crate::map::{JourneyMapViewerState};

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

    let mut cursor_in_ui = false;

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
            println!("cursors: {}", cursor_in_ui);
        });

        // macroquadの描画処理
        clear_background(LIGHTGRAY);

        let screen_size = vec2(screen_width(), screen_height());

        // マウスの処理
        if !cursor_in_ui && is_mouse_button_down(MouseButton::Left) {
            // マウスが押下されたとき
            state.clicked();
        }
        if is_mouse_button_released(MouseButton::Left) {
            // マウスが離されたとき
            state.released();
        }
        state.dragging(mouse_delta_position(), screen_size);

        // ホイールの処理
        if !cursor_in_ui {
            state.scrolling(mouse_wheel().1);
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

        camera.target = state.camera_position();
        camera.zoom = state.camera_zoom(screen_size);

        set_camera(&camera);

        // カメラへ描画
        for ((rx, rz), img) in state.images() {
            let dest_x = rx * 512;
            let dest_y = rz * 512;
            draw_texture(*img, dest_x as f32, dest_y as f32, WHITE);
        }

        set_default_camera();
        draw_text("Hello macroquad!", 20.0, 40.0, 30.0, DARKGRAY);


        // 描画更新
        egui_macroquad::draw();

        // next
        next_frame().await;
    }
}
