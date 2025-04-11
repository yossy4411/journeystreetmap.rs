use fastanvil::Region;
use fltk::prelude::{GroupExt, InputExt, MenuExt, WidgetBase};
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use macroquad::prelude::Texture2D;
use tiny_skia::Pixmap;


#[derive(Debug, Clone)]
/// 画像の状態を管理する構造体
struct ImageState {
    zoom: f32,
    zoom_factor: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl Default for ImageState {
    fn default() -> Self {
        ImageState {
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

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Default)]
/// 編集のモード
enum EditingMode {
    #[default]
    Insert,
    Delete,
    Select,
    View,
}

// 編集対象
#[derive(PartialEq, Hash, Copy, Clone, Debug, Default)]
enum EditingType {
    #[default]
    Stroke,  // 線（道路）
    Fill,    // 塗りつぶし（建物）
    Poi,    // ポイント（村、都市、交差点...）
}

#[derive(Debug, Default)]
pub struct JourneyMapViewerState {
    pub images: HashMap<(i32, i32), Texture2D>,  // Regionごとの画像データをキャッシュするためのHashMap
    image_state: ImageState,
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

