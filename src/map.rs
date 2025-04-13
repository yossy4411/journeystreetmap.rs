use fastanvil::Region;
use fltk::prelude::{GroupExt, InputExt, MenuExt, WidgetBase};
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use macroquad::math::{vec2, Vec2};
use macroquad::prelude::Texture2D;
use tiny_skia::Pixmap;


#[derive(Debug, Clone)]
/// 画像の状態を管理する構造体
struct MouseHandling {
    zoom: f32,
    zoom_factor: f32,
    position: Vec2,
    pressed: bool,
}

impl Default for MouseHandling {
    fn default() -> Self {
        MouseHandling {
            zoom: 0.2,
            zoom_factor: 1.25,
            position: Vec2::new(0.0, 0.0),
            pressed: false,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Default)]
/// 編集のモード
pub enum EditingMode {
    #[default]
    Insert,
    Delete,
    Select,
    View,
}

// 編集対象
#[derive(PartialEq, Hash, Copy, Clone, Debug, Default)]
pub enum EditingType {
    #[default]
    Stroke,  // 線（道路）
    Fill,    // 塗りつぶし（建物）
    Poi,    // ポイント（村、都市、交差点...）
}

#[derive(Debug, Default)]
pub struct JourneyMapViewerState {
    images: HashMap<(i32, i32), Texture2D>,  // Regionごとの画像データをキャッシュするためのHashMap
    mouse_handling: MouseHandling,
    edit_mode: EditingMode,
    editing_type: EditingType,
    editable: bool,
    path: Vec<(f32, f32)>,
}

impl JourneyMapViewerState {
    pub fn load_images(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
        let region_offset_x = 0;
        let region_offset_z = 0;

        let stopwatch = std::time::Instant::now();

        let mut threads = Vec::new();
        let regions = // reader.get_regions_list();
            [(-1, -1), (0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

        for (i, (region_x, region_z)) in regions.into_iter().enumerate() {
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
            if i > 20 {
                break;
            }
        }

        for thr in threads {
            let (key, content) = thr.join().unwrap();
            let texture = Texture2D::from_rgba8(512, 512, &content);
            self.images.insert(key, texture);
        }
        println!("Time taken: {:?}", stopwatch.elapsed());
        Ok(())
    }

    fn buffer_region(region: &mut Region<File>, region_offset_x: i32, region_offset_z: i32, region_x: i32, region_z: i32) -> Vec<u8> {
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
                                let color: tiny_skia::Color = color.into();
                                image_data[i] = color.premultiply().to_color_u8()
                            }
                        }
                    }
                }
            }
        }
        pixmap.data().to_vec()
    }
}

impl JourneyMapViewerState {
    /// クリックされた瞬間の処理
    pub fn clicked(&mut self) {
        self.mouse_handling.pressed = true;
        // todo Pathを追加する処理などなど
    }

    /// クリックが離された瞬間の処理
    pub fn released(&mut self) {
        self.mouse_handling.pressed = false;
    }

    /// マウスのドラッグの処理
    pub fn dragging(&mut self, delta: Vec2, screen_size: Vec2) {
        if !self.mouse_handling.pressed {
            return;
        }
        self.mouse_handling.position += delta / self.mouse_handling.zoom * screen_size;
    }

    /// 画像の参照を返す
    pub fn images(&self) -> &HashMap<(i32, i32), Texture2D> {
        &self.images
    }

    pub fn camera_position(&self) -> Vec2 {
        vec2(self.mouse_handling.position.x, self.mouse_handling.position.y)
    }

    pub fn camera_zoom(&self, screen_size: Vec2) -> Vec2 {
        vec2(self.mouse_handling.zoom, -self.mouse_handling.zoom) / screen_size
    }

    pub fn editing_type(&self) -> EditingType {
        self.editing_type
    }

    pub fn editing_mode(&self) -> EditingMode {
        self.edit_mode
    }

    /// 編集モードを切り替える（周期的に）
    pub fn toggle_editing_type(&mut self) {
        self.editing_type = match self.editing_type {
            EditingType::Stroke => EditingType::Fill,
            EditingType::Fill => EditingType::Poi,
            EditingType::Poi => EditingType::Stroke,
        };
    }

    /// 編集モードを切り替える
    pub fn set_editing_mode(&mut self, editing_mode: EditingMode) {
        self.edit_mode = editing_mode;
    }
}