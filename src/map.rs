use fastanvil::asyncio::Region;
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::fs::File;
use macroquad::math::{Vec2};
use macroquad::prelude::Texture2D;
use tokio::sync::Mutex;
use journeystreetmap::journeymap::biome::RGB;

#[derive(Debug, Clone)]
/// 画像の状態を管理する構造体
struct MouseHandling {
    zoom: f32,
    zoom_factor: f32,
    position: Vec2,
    pressed: bool,
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
    edit_mode: EditingMode,
    editing_type: EditingType,
    editable: bool,
    path: Vec<(f32, f32)>,
}

pub async fn load_images(images: Arc<Mutex<Vec<((i32, i32), Box<[u8;512*512*4]>)>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
    let region_offset_x = 0;
    let region_offset_z = 0;

    let stopwatch = std::time::Instant::now();

    let mut threads = Vec::new();
    let regions = // reader.get_regions_list();
        [(-1, -1), (0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

    for (i, (region_x, region_z)) in regions.into_iter().enumerate() {
        let region = reader.try_read_region(region_offset_x + region_x, region_offset_z + region_z).await;
        if region.is_some() {
            let thr = tokio::spawn(async move {
                buffer_region(region.unwrap(), region_offset_x, region_offset_z, region_x, region_z).await
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
        let obj = thr.await.unwrap();
        images.lock().await.push(obj);  // 画像を保存
    }
    println!("Time taken: {:?}", stopwatch.elapsed());
    Ok(())
}

async fn buffer_region(mut region: Region<File>, region_offset_x: i32, region_offset_z: i32, region_x: i32, region_z: i32) -> ((i32, i32), Box<[u8;512 * 512 * 4]>) {
    let mut image_data = Box::new([RGB::default(); 512 * 512]);
    for i in 0..=31 {
        for j in 0..=31 {
            let chunk_result = JourneyMapReader::get_chunk(&mut region, i, j).await;
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
                            image_data[i] = color;
                        }
                    }
                }
            }
        }
    }
    let mut colors = Box::new([0_u8; 512 * 512 * 4]);  // RGBA8
    for i in 0..512 * 512 {
        let color = image_data[i];
        colors[i * 4] = color.r;
        colors[i * 4 + 1] = color.g;
        colors[i * 4 + 2] = color.b;
        colors[i * 4 + 3] = 255; // Alpha
    }
    ((region_x, region_z), colors)
}

impl JourneyMapViewerState {
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