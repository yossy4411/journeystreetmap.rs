use fastanvil::Region;
use journeystreetmap::journeymap;
use journeystreetmap::journeymap::biome::RGB;
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use pixels::{Pixels, SurfaceTexture};
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut app = Application::new();
    event_loop.unwrap().run_app(&mut app)?;

    Ok(())
}


// 画像の状態を管理する構造体
struct ImageState {
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl ImageState {
    fn new() -> Self {
        Self {
            zoom: 1.0,
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


struct Application {
    image_state: ImageState,
    images: HashMap<String, Vec<RGB>>,  // Regionごとの画像データをキャッシュするためのHashMap
    pixels: Option<Arc<Mutex<Pixels<'static>>>>, // 'staticにするとなんかへんかなって思ったからとりまApplicationと同じライフタイムにしてみた
    image_width: u32,
    image_height: u32,
    last_rendered: std::time::Instant,
}

impl Application {
    fn new() -> Self {
        Self {
            image_state: ImageState::new(),
            images: HashMap::new(),
            pixels: None,
            image_width: 800,
            image_height: 800,
            last_rendered: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(& mut self, event_loop: &ActiveEventLoop) {
        let window_attr = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(800,800))
            .with_title("JourneyMap Viewer");
        let window = event_loop
            .create_window(window_attr)
            .expect("Failed to create window");

        let window_size = window.inner_size();
        let pixels = Pixels::new(
            self.image_width,
            self.image_height,
            SurfaceTexture::new(window_size.width, window_size.height, window),
        ).expect("Pixels creation failed");

        self.load_images().expect("Failed to load images");

        self.pixels = Some(Arc::new(Mutex::new(pixels)));
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
                    self.image_state.offset_x -= dx;
                    self.image_state.offset_y -= dy;   // Y軸は上下逆
                    // self.request_render();
                    self.render().expect("Failed to render");
                }
                self.image_state.last_mouse_x = position.x;
                self.image_state.last_mouse_y = position.y;
            }
            WindowEvent::RedrawRequested => {
                self.render().expect("Failed to render");
            }
            WindowEvent::MouseWheel {
                delta,
                ..
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                        if y > 0.0 {
                            self.image_state.zoom = self.image_state.zoom * 1.1;
                        } else {
                            self.image_state.zoom = self.image_state.zoom / 1.1;
                        }
                        self.render().expect("Failed to render");
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }
}

impl Application {
    fn load_images(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = journeymap::JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
        let region_offset_x = 0;
        let region_offset_z = 0;

        let stopwatch = std::time::Instant::now();

        let mut threads = Vec::new();

        for region_x in -1..=3 {
            for region_z in -1..=3 {
                let mut region = reader.try_read_region(region_offset_x + region_x, region_offset_z + region_z);
                if region.is_none() {
                    println!("Region not found");
                    continue;
                }
                let thr = std::thread::spawn(move || {
                    (format!("r.{}.{}", region_x, region_z), Self::buffer_region(&mut region.unwrap(), region_offset_x, region_offset_z, region_x, region_z))
                });
                threads.push(thr);
            }
        }

        for thr in threads {
            let (key, content) = thr.join().unwrap();
            self.images.insert(key, content);
        }
        println!("Time taken: {:?}", stopwatch.elapsed());
        Ok(())
    }

    fn buffer_region(region: &mut Region<File>, region_offset_x: i32, region_offset_z: i32, region_x: i32, region_z: i32) -> Vec<RGB> {

        let mut image_data = vec![RGB::default(); 512 * 512];
        for i in 0..=31 {
            for j in 0..=31 {
                let chunk_result = JourneyMapReader::get_chunk(region, i, j);
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
        image_data
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.pixels.is_some() && self.last_rendered.elapsed().as_millis() > 1000 / 30 {
            self.last_rendered = std::time::Instant::now();
        } else {
            return Ok(());  // 30fpsになるように制御
        }
        {
            let mut binding = self.pixels.as_ref().unwrap().lock().unwrap();
            let frame = binding.frame_mut();

            // フレームをクリア
            frame.fill(0);

            // 映る範囲を計算
            let left = self.image_state.offset_x as i32;
            let top = self.image_state.offset_y as i32;
            let region_left_x = (left as f32 / 512.0).floor() as i32;
            let region_left_z = (top as f32 / 512.0).floor() as i32;
            let x_times = (512 + 800) as f32 / 512.0 / self.image_state.zoom;
            let y_times = (512 + 800) as f32 / 512.0 / self.image_state.zoom;
            for rx in 0..=x_times as i32 {
                for rz in 0..=y_times as i32 {
                    let region_x = region_left_x + rx;
                    let region_z = region_left_z + rz;
                    let key = format!("r.{}.{}", region_x, region_z);
                    if !self.images.contains_key(&key) {
                        continue;
                    }
                    let image_data = self.images.get(&key).unwrap();
                    for x in 0..512 {
                        for y in 0..512 {
                            if x < 0 || x >= 512 || y < 0 || y >= 512 {
                                continue;
                            }
                            let ori_idx = (y * 512 + x) as usize;  // もとの画像データのインデックス
                            let color = image_data[ori_idx];
                            let dest_x = x + rx * 512 - JourneyMapReader::positive_modulo(left, 512);
                            let dest_y = y + rz * 512 - JourneyMapReader::positive_modulo(top, 512);
                            let dest_x = (dest_x as f32 * self.image_state.zoom) as i32;
                            let dest_y = (dest_y as f32 * self.image_state.zoom) as i32;
                            if dest_x < 0 || dest_x >= self.image_width as i32 || dest_y < 0 || dest_y >= self.image_height as i32 {
                                continue;
                            }
                            let dest_idx = (dest_y * self.image_width as i32 + dest_x) as usize;  // 表示する画像データのインデックス
                            frame[dest_idx * 4] = color.r;
                            frame[dest_idx * 4 + 1] = color.g;
                            frame[dest_idx * 4 + 2] = color.b;
                            frame[dest_idx * 4 + 3] = 255;
                        }
                    }
                }
            }
            binding.render().unwrap();
        }



        Ok(())
    }
}