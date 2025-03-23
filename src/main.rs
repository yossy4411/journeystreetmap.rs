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
    window: Option<Window>
}


// 画像の状態を管理する構造体
struct ImageState {
    width: u32,
    height: u32,
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl ImageState {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
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
        self.render().expect("Failed to render");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

impl Application {
    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = journeymap::JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
        let region_offset_x = -1;
        let region_offset_z = -1;

        let image_width = 1024; // 1 chunk = 16 blocks, 32 chunks = 512 blocks (1 region)
        let image_height = 1024;
        let window_size = self.window.as_ref().ok_or("window not initialized")?.inner_size();

        let win = self.window.as_mut().ok_or("window not initialized")?;
        let mut pixels = Pixels::new(image_width, image_height, SurfaceTexture::new(window_size.width, window_size.height, win))?;

        let stopwatch = std::time::Instant::now();
        let mut image_data = vec![RGB::default(); (image_width * image_height) as usize];

        for region_x in 0..=1 {
            for region_z in 0..=1 {
                reader.read_region(region_offset_x + region_x, region_offset_z + region_z).expect("Failed to read region");

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
                                let block_x = x - 512 * region_offset_x;
                                let block_z = z - 512 * region_offset_z;

                                // Z座標を反転させ、X-Z平面にマッピング
                                let pixel_x = block_x;
                                let pixel_y = block_z;

                                // RGBA配列のインデックスを計算
                                let i = (pixel_y * image_width as i32 + pixel_x) as usize;

                                // 範囲チェック
                                if i < image_data.len() {
                                    let color = biome::get_color(&data.biome_name);
                                    image_data[i] = color;

                                    // Grid
                                    image_data[i] =
                                        if block_x % 16 == 0 || block_z % 16 == 0 {
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
            }
        }
        println!("Time taken: {:?}", stopwatch.elapsed());

        let frame = pixels.frame_mut();

        // フレームをクリア
        frame.fill(0);

        // 画像データをフレームにコピー
        for (i, pixel) in image_data.iter().enumerate() {
            let i = i;
            frame[i * 4] = pixel.r;
            frame[i * 4 + 1] = pixel.g;
            frame[i * 4 + 2] = pixel.b;
            frame[i * 4 + 3] = 255;
        }

        pixels.render().unwrap();
        println!("Load and Render Taken: {:?}", stopwatch.elapsed());
        Ok(())
    }
}