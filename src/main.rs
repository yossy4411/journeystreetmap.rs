use fastanvil::Region;
use journeystreetmap::journeymap::biome::RGB;
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use softbuffer::{Context, Surface};
use std::collections::HashMap;
use std::fs::File;
use std::num::NonZeroU32;
use std::rc::Rc;
use tiny_skia::{Color, Pixmap, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

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
    images: HashMap<(i32, i32), Pixmap>,  // Regionごとの画像データをキャッシュするためのHashMap
    canvas: Option<Pixmap>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    window: Option<Rc<Window>>,
    width: u32,
    height: u32,
}

impl Application {
    fn new() -> Self {
        Self {
            image_state: ImageState::new(),
            images: HashMap::new(),
            canvas: None,
            surface: None,
            window: None,
            width: 800,
            height: 800,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(& mut self, event_loop: &ActiveEventLoop) {
        let window_attr = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_title("JourneyMap Viewer");
        let window = event_loop
            .create_window(window_attr)
            .expect("Failed to create window");
        self.load_images().expect("Failed to load images");
        let window_rc = Rc::new(window);


        let canvas = Pixmap::new(self.width, self.height).unwrap();
        let context = Context::new(window_rc.clone()).unwrap();
        let surface = Surface::new(&context, window_rc.clone()).unwrap();
        self.window = Some(window_rc);
        self.surface = Some(surface);




        self.canvas = Some(canvas);
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
                    self.image_state.offset_y += dy;   // Y軸は上下逆
                    self.window.as_ref().unwrap().request_redraw();
                }
                self.image_state.last_mouse_x = position.x;
                self.image_state.last_mouse_y = position.y;
            }
            WindowEvent::RedrawRequested => {
                self.render().expect("Failed to render");

                let surface = self.surface.as_mut().unwrap();

                surface.resize(NonZeroU32::new(self.width).unwrap(), NonZeroU32::new(self.height).unwrap()).expect("Failed to resize");
                let mut buffer = surface.buffer_mut().unwrap();
                let data = self.canvas.as_ref().unwrap().data();
                for index in 0..(self.width * self.height) as usize {
                    buffer[index] =
                        data[index * 4 + 2] as u32
                            | (data[index * 4 + 1] as u32) << 8
                            | (data[index * 4 + 0] as u32) << 16;
                }
                buffer.present().unwrap();
            }
            WindowEvent::MouseWheel {
                delta,
                ..
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                        let factor = if y > 0.0 { 1.1 } else { 1.0 / 1.1 };
                        self.image_state.zoom *= factor;

                        self.image_state.offset_x = (self.image_state.offset_x - self.width as f32  / 2.0) * factor + self.width as f32  / 2.0;
                        self.image_state.offset_y = (self.image_state.offset_y - self.height as f32 / 2.0) * factor + self.height as f32 / 2.0;

                        self.window.as_mut().unwrap().request_redraw();
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
        let mut reader = JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
        let region_offset_x = 0;
        let region_offset_z = 0;

        let stopwatch = std::time::Instant::now();

        let mut threads = Vec::new();
        let regions = reader.get_regions_list();

        for (region_x, region_z) in regions {
            let region = reader.try_read_region(region_offset_x + region_x, region_offset_z + region_z);
            if let Some(mut region) = region {
                let thr = std::thread::spawn(move || {
                    ((region_x, region_z), Self::buffer_region(&mut region, region_offset_x, region_offset_z, region_x, region_z))
                });
                threads.push(thr);
            } else {
                println!("Region not found");
                continue;
            }


        }

        for thr in threads {
            let (key, content) = thr.join().unwrap();
            self.images.insert(key, content);
        }
        println!("Time taken: {:?}", stopwatch.elapsed());
        Ok(())
    }

    fn buffer_region(region: &mut Region<File>, region_offset_x: i32, region_offset_z: i32, region_x: i32, region_z: i32) -> Pixmap {
        let mut pixmap = Pixmap::new(512, 512).unwrap();
        let image_data = pixmap.pixels_mut();
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
                        if i < 512 * 512 {
                            let color = biome::get_color(&data.biome_name);
                            // Grid
                            let color: Color =
                                if pixel_x % 16 == 0 || pixel_y % 16 == 0 {
                                    color.blend(&RGB::new(255, 255, 255), 0.8).into()
                                } else {
                                    color.into()
                                };

                            image_data[i] = color.premultiply().to_color_u8()
                        }
                    }
                } else {
                    println!("Chunk load failed: {:?}", chunk_result.err());
                    continue;
                }
            }
        }
        pixmap
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pixmap = self.canvas.as_mut().ok_or("Canvas not found")?;
        let transform = Transform::from_scale(self.image_state.zoom, self.image_state.zoom)
            .post_translate(self.image_state.offset_x, self.image_state.offset_y);
        {
            // 黒でクリア
            pixmap.fill(Color::BLACK);

            let paint = tiny_skia::PixmapPaint::default();

            for ((rx, rz), img) in &self.images {
                let dest_x = rx * 512;
                let dest_y = rz * 512;
                pixmap.draw_pixmap(dest_x, dest_y, img.as_ref(), &paint, transform.clone(), None)
            }
        }


        Ok(())
    }
}