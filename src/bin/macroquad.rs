// カーソル位置を中心にズームするサンプル (ChatGPT生成)
use macroquad::prelude::*;

#[macroquad::main("Zoom on Cursor")]
async fn main() {
    let mut camera = Camera2D::default();

    loop {
        clear_background(WHITE);

        // 現在のマウス位置（スクリーン座標）
        let mouse_screen = mouse_position().into();

        // ズーム前のワールド座標でのマウス位置
        let before_zoom = camera.screen_to_world(mouse_screen);

        // マウスホイールでズーム倍率変更
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let zoom_factor = 1.1_f32.powf(scroll); // 1スクロールで1.1倍 or 1/1.1倍
            camera.zoom *= vec2(zoom_factor, zoom_factor);

            // ズーム後のマウス位置（ワールド座標）
            let after_zoom = camera.screen_to_world(mouse_screen);

            // ズームによってズレたぶんだけcamera.targetを調整
            camera.target += before_zoom - after_zoom;
        }

        // カメラ設定
        set_camera(&camera);

        // ワールドに何か描く（例：十字）
        draw_line(-1000.0, 0.0, 1000.0, 0.0, 2.0, RED);
        draw_line(0.0, -1000.0, 0.0, 1000.0, 2.0, BLUE);

        // カメラ解除（UIなどのため）
        set_default_camera();

        draw_text("スクロールでズーム（マウス中心）！", 20.0, 20.0, 24.0, BLACK);

        next_frame().await;
    }
}
