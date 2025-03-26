use fastanvil::Region;
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use softbuffer::{Context, Surface};
use std::collections::HashMap;
use std::fs::File;
use std::num::NonZeroU32;
use std::rc::Rc;
use rusttype::{point, Font, OutlineBuilder, Scale};
use tiny_skia::{Color, Path, PathBuilder, Pixmap, Point, Rect, Stroke, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
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
    zoom_factor: f32,
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
            zoom_factor: 1.25,
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

// 編集モード
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum EditingMode {
    Insert,
    Delete,
    Select,
    View,
}

// 編集対象
#[derive(PartialEq, Hash, Copy, Clone, Debug)]
enum EditingType {
    Stroke,  // 線（道路）
    Fill,    // 塗りつぶし（建物）
    Poi,    // ポイント（村、都市、交差点...）
}

// 編集したものを保存するenum
#[derive(Debug)]
enum EditResult {
    StrokePath(Path),
    FillPath(Path),
    PoiPoint(Point),
}


struct Application {
    image_state: ImageState,
    images: HashMap<(i32, i32), Pixmap>,  // Regionごとの画像データをキャッシュするためのHashMap
    canvas: Option<Pixmap>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    window: Option<Rc<Window>>,
    width: u32,
    height: u32,
    edit_mode: EditingMode,
    editable: bool,
    font: Font<'static>,
    editing_type: EditingType,
    path: PathBuilder,
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
            edit_mode: EditingMode::View,
            editable: false,
            font: Font::try_from_bytes(include_bytes!("../fonts/NotoSansJP-Regular.ttf") as &[u8]).unwrap(),
            editing_type: EditingType::Stroke,
            path: PathBuilder::new(),
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
                if self.editable {
                    if state == ElementState::Pressed {
                        if button == MouseButton::Right && self.edit_mode == EditingMode::Insert {
                            // 右クリックって、パス閉じるのか、それとも1個前のポイントに戻るのか。どっちがいいかな？
                            // → 1個前のポイントに戻るのがいいかな
                            if let Some(path) = self.path.clone().finish() {
                                self.path = PathBuilder::new();
                                let poi = path.points();
                                if let Some(poi) = poi.first() {
                                    self.path.move_to(poi.x, poi.y);
                                }
                                for i in 1..poi.len() - 1 {
                                    self.path.line_to(poi[i].x, poi[i].y);
                                }
                                self.window.as_ref().unwrap().request_redraw();
                            }
                        } else if button == MouseButton::Left {
                            if self.edit_mode == EditingMode::Insert {
                                let x = (self.image_state.last_mouse_x - self.image_state.offset_x) / self.image_state.zoom;
                                let y = (self.image_state.last_mouse_y - self.image_state.offset_y) / self.image_state.zoom;
                                let x = x.round();  // ブロックの位置に丸める
                                let y = y.round();
                                if self.editing_type == EditingType::Poi {
                                    // todo 新しいウィンドウを開いてタグを入力する
                                    // winitで全部やるのは面倒すぎて死ぬ
                                    // 容量が小さくて使い勝手がいいやつ。
                                    // あと使い捨てになるし、ウィンドウで入力した情報が戻り値として返ってくるやつがいいネ
                                    // → GitHub Copilotによると、eguiが適してるって

                                } else {
                                    if self.path.len() > 0 {
                                        self.path.line_to(x, y);
                                    } else {
                                        self.path.move_to(x, y);
                                    }
                                    self.window.as_ref().unwrap().request_redraw();
                                }
                            }
                        }
                    }
                }
                self.image_state.dragging = !self.editable && button == MouseButton::Left && state == ElementState::Pressed;
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
            WindowEvent::MouseWheel {
                delta,
                ..
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                        let factor = if y > 0.0 { self.image_state.zoom_factor } else { 1.0 / self.image_state.zoom_factor };
                        self.image_state.zoom *= factor;

                        let zoom_origin_x = self.image_state.last_mouse_x;
                        let zoom_origin_y = self.image_state.last_mouse_y;
                        self.image_state.offset_x = (self.image_state.offset_x - zoom_origin_x) * factor + zoom_origin_x;
                        self.image_state.offset_y = (self.image_state.offset_y - zoom_origin_y) * factor + zoom_origin_y;
                        self.window.as_mut().unwrap().request_redraw();
                    }
                    _ => {}
                }
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
            WindowEvent::KeyboardInput {
                event,
                ..
            } => {
                match event.logical_key {
                    Key::Character(s) => {
                        if event.state == ElementState::Pressed {
                            match s.to_uppercase().as_str() {
                                "I" => {
                                    self.edit_mode = EditingMode::Insert;
                                    println!("Insert mode");
                                }
                                "D" => {
                                    self.edit_mode = EditingMode::Delete;
                                    println!("Delete mode");
                                }
                                "S" => {
                                    self.edit_mode = EditingMode::Select;
                                    println!("Select mode");
                                }
                                "V" => {
                                    self.edit_mode = EditingMode::View;
                                    println!("View mode");
                                }
                                "E" => {
                                    // 編集対象を周期的に切り替える
                                    self.editing_type = match self.editing_type {
                                        EditingType::Stroke => EditingType::Fill,
                                        EditingType::Fill => EditingType::Poi,
                                        EditingType::Poi => EditingType::Stroke,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    Key::Named(name) => {
                        match name {
                            NamedKey::Shift => {
                                self.editable = event.state == ElementState::Pressed;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                self.window.as_ref().unwrap().request_redraw();
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
                match chunk_result {
                    Err(..) => {
                        continue;
                    }
                    Ok(chunk) => {
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
                                let color: Color = color.into();
                                image_data[i] = color.premultiply().to_color_u8()
                            }
                        }
                    }
                }
            }
        }
        pixmap
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pixmap = self.canvas.as_mut().ok_or("Canvas not found")?;
        let transform = Transform::from_scale(self.image_state.zoom, self.image_state.zoom)
            .post_translate(self.image_state.offset_x, self.image_state.offset_y);
        // 黒でクリア
        pixmap.fill(Color::BLACK);

        let pixmap_paint = tiny_skia::PixmapPaint::default();
        let rect_paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 255, 255, 255)),
            ..Default::default()
        };
        let grid_paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 255, 255, 100)),
            ..Default::default()
        };

        for ((rx, rz), img) in &self.images {
            let dest_x = rx * 512;
            let dest_y = rz * 512;
            pixmap.draw_pixmap(dest_x, dest_y, img.as_ref(), &pixmap_paint, transform.clone(), None);
            // でかいほうのグリッド
            let stroke = Stroke {
                width: 0.2,
                ..Default::default()
            };

            let path = tiny_skia::PathBuilder::from_rect(Rect::from_xywh(dest_x as f32, dest_y as f32, 512.0, 512.0).unwrap());
            pixmap.stroke_path(&path, &rect_paint, &stroke, transform.clone(), None);

            // 小さいほうのグリッド
            for i in 0..=32 {
                let x = dest_x as f32 + i as f32 * 16.0;
                let y = dest_y as f32 + i as f32 * 16.0;
                let mut path = tiny_skia::PathBuilder::new();
                path.move_to(x, dest_y as f32);
                path.line_to(x, dest_y as f32 + 512.0);
                pixmap.stroke_path(&path.finish().unwrap(), &grid_paint, &stroke, transform.clone(), None);
                let mut path = tiny_skia::PathBuilder::new();
                path.move_to(dest_x as f32, y);
                path.line_to(dest_x as f32 + 512.0, y);
                pixmap.stroke_path(&path.finish().unwrap(), &grid_paint, &stroke, transform.clone(), None);
            }
        }

        // テキストの描画
        let mut mode: String = match self.edit_mode {
            EditingMode::Insert => "Insert",
            EditingMode::Delete => "Delete",
            EditingMode::Select => "Select",
            EditingMode::View => "View",
        }.to_string();

        let mut color = Color::WHITE;
        if self.edit_mode != EditingMode::View {
            mode += match self.editing_type {
                EditingType::Stroke => " Stroke",
                EditingType::Fill => " Fill",
                EditingType::Poi => " Poi",
            };
            if !self.editable {
                color.apply_opacity(0.5);
            }

            match self.editing_type {
                EditingType::Fill => {
                    let mut path = self.path.clone();
                    path.close();
                    if let Some(path) = path.finish() {
                        let paint = tiny_skia::Paint {
                            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 100, 100, 100)),  // red
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &paint, &Stroke{width:2.0, ..Default::default()}, transform.clone(), None);
                        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, transform.clone(), None);
                    }
                }
                EditingType::Stroke => {
                    if let Some(path) = self.path.clone().finish() {
                        let paint = tiny_skia::Paint {
                            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(100, 100, 255, 100)),  // blue
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &paint, &Stroke { width: 2.0, ..Default::default() }, transform.clone(), None);
                        let paint = tiny_skia::Paint {
                            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 255, 255, 50)),  // white
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &paint, &Stroke { width: 1.0, ..Default::default() }, transform.clone(), None);
                    }
                }
                _ => {}
            }
        }
        Self::draw_text(pixmap, &self.font, Scale::uniform(16.0), rusttype::point(6.0, self.height as f32 - 20.0), &mode, color);


        Ok(())
    }


    fn draw_text(pixmap: &mut Pixmap, font: &Font, scale: Scale, start: rusttype::Point<f32>, text: &str, color: Color) {
        // Paintの設定
        let paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(color), // 緑色のテキスト
            ..Default::default()
        };

        // ベースラインの位置を計算
        let v_metrics = font.v_metrics(scale);
        let offset = start.y + v_metrics.ascent;

        // グリフのレイアウトを計算
        let glyphs: Vec<_> = font.layout(text, scale, point(start.x, offset)).collect();

        for glyph in glyphs {
            let mut path = GriffPathBuilder::new();
            let pos = glyph.position();
            if glyph.build_outline(&mut path) {
                let path = path.unwrap();
                let bounds = path.bounds();
                let path = path.transform(Transform::from_translate(pos.x, pos.y - bounds.height())).unwrap();
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
    }
}

struct GriffPathBuilder {
    path_builder: PathBuilder,
}

// OutlineBuilderトレイトを実装するための実装。単純にラッピングしてるだけっていうねｗ
impl OutlineBuilder for GriffPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path_builder.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path_builder.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path_builder.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path_builder.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.path_builder.close();
    }
}

impl GriffPathBuilder {
    fn new() -> Self {
        Self {
            path_builder: PathBuilder::new(),
        }
    }

    fn unwrap(self) -> Path {
        self.path_builder.finish().unwrap()
    }
}