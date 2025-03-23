use std::collections::HashMap;
use std::fmt::format;
use journeystreetmap::journeymap;
use journeystreetmap::journeymap::biome;
use journeystreetmap::journeymap::biome::RGB;
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut app = Application::default();
    event_loop.unwrap().run_app(&mut app)?;

    Ok(())
}

#[derive(Default)]
struct Application {
    window: Option<Window>,
    image_state: ImageState,
    images: HashMap<String, Vec<RGB>>,  // Regionごとの画像データをキャッシュするためのHashMap
}


// 画像の状態を管理する構造体
struct ImageState {
    zoom: u32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl ImageState {
    fn new() -> Self {
        Self {
            zoom: 1,
            offset_x: 0.0,
            offset_y: 0.0,
            dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
    }
}

impl Default for ImageState {
    fn default() -> Self {
        Self::new()
    }
}


impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attr = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(800,800))
            .with_title("JourneyMap Viewer");
        let window = event_loop
            .create_window(window_attr)
            .expect("Failed to create window");
        self.window = Some(window);
        self.load_images();
        self.render().expect("Failed to render");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::MouseInput {
                state,
                button,
                ..
            } => {
                if button == winit::event::MouseButton::Left {
                    self.image_state.dragging = state == winit::event::ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved {
                position,
                ..
            } => {
                let position = position.to_logical::<f32>(1.0);
                let dx = position.x - self.image_state.last_mouse_x;
                let dy = position.y - self.image_state.last_mouse_y;
                if self.image_state.dragging {
                    self.image_state.offset_x += dx;
                    self.image_state.offset_y -= dy;   // Y軸は上下逆
                    self.render().expect("Failed to render");
                }
                self.image_state.last_mouse_x = position.x;
                self.image_state.last_mouse_y = position.y;
            }
            _ => {}
        }
    }
}

impl Application {
    fn load_images(&mut self) {
        let mut reader = journeymap::JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
        let region_offset_x = -1;
        let region_offset_z = -1;

        let stopwatch = std::time::Instant::now();

        for region_x in 0..=1 {
            for region_z in 0..=1 {
                reader.read_region(region_offset_x + region_x, region_offset_z + region_z).expect("Failed to read region");
                let mut image_data = vec![RGB::default(); 512 * 512];
                for i in 0..=31 {
                    for j in 0..=31 {
                        let chunk_result = reader.get_chunk(i, j);
                        if let Ok(chunk) = chunk_result {
                            if chunk.is_none() {
                                println!("Chunk not found");
                                continue;
                            }
                            let chunk = chunk.unwrap();
                            for (pos, data) in chunk.sections {
                                let mut splited = pos.split(',');
                                let x: i32 = splited.next().unwrap().parse().unwrap();
                                let z: i32 = splited.next().unwrap().parse().unwrap();

                                // ブロック座標をリージョン内の相対座標に変換
                                let pixel_x = x - 512 * (region_offset_x + region_x);
                                let pixel_y = z - 512 * (region_offset_z + region_z);

                                // RGBA配列のインデックスを計算
                                let i = (pixel_y * 512 + pixel_x) as usize;

                                // iが画像内に入るなら色を設定
                                if i < image_data.len() {
                                    let color = biome::get_color(&data.biome_name);
                                    image_data[i] = color;

                                    // Grid
                                    image_data[i] =
                                        if pixel_x % 16 == 0 || pixel_y % 16 == 0 {
                                            color.blend(&RGB::new(255, 255, 255), 0.8)
                                        } else {
                                            color
                                        };
                                }
                            }
                        } else {
                            println!("Chunk load failed: {:?}", chunk_result.err());
                            continue;
                        }
                    }
                }
                self.images.insert(format!("r.{}.{}", region_x, region_z), image_data);

            }
        }
        println!("Time taken: {:?}", stopwatch.elapsed());
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let image_width = 800; // 1 chunk = 16 blocks, 32 chunks = 512 blocks (1 region)
        let image_height = 800;

        let win = self.window.as_mut().ok_or("window not initialized")?;
        let window_size = win.inner_size();

        let mut pixels = Pixels::new(image_width, image_height, SurfaceTexture::new(window_size.width, window_size.height, win))?;


        let frame = pixels.frame_mut();

        // フレームをクリア
        frame.fill(0);

        // 映る範囲を計算
        let left = self.image_state.offset_x as i32;
        let top = self.image_state.offset_y as i32;
        let region_x = left / 512;
        let region_z = top / 512;
        let key = format!("r.{}.{}", region_x, region_z);
        if !self.images.contains_key(&key) {
            return Ok(());
        }
        let image_data = self.images.get(&key).unwrap();

        let block_offset_x = left % 512;
        let block_offset_y = top % 512;
        for x in block_offset_x..512 {
            for y in block_offset_y..512 {
                if x < 0 || x >= 512 || y < 0 || y >= 512 {
                    continue;
                }
                let ori_idx = (y * 512 + x) as usize;  // もとの画像データのインデックス
                let color = image_data[ori_idx];
                let dest_x = x - block_offset_x;
                let dest_y = y - block_offset_y;
                if dest_x < 0 || dest_x >= image_width as i32 || dest_y < 0 || dest_y >= image_height as i32 {
                    continue;
                }
                let dest_idx = (dest_y * image_width as i32 + dest_x) as usize;  // 表示する画像データのインデックス
                frame[dest_idx * 4] = color.r;
                frame[dest_idx * 4 + 1] = color.g;
                frame[dest_idx * 4 + 2] = color.b;
                frame[dest_idx * 4 + 3] = 255;
            }
        }

        pixels.render().unwrap();
        Ok(())
    }
}