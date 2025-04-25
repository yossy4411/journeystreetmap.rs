use bevy::math::Vec2;
use bevy::render::render_resource::Extent3d;
use fastanvil::asyncio::Region;
use journeystreetmap::journeymap::biome::RGB;
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;
use bevy::prelude::Resource;
use tokio::fs::File;
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
/// 画像の状態を管理する構造体
struct MouseHandling {
    zoom: f32,
    zoom_factor: f32,
    translation: Vec2,
    last_mouse_pos: Vec2,
}

impl Default for MouseHandling {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            zoom_factor: 1.3,
            translation: Vec2::ZERO,
            last_mouse_pos: Vec2::ZERO,
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

#[derive(Debug, Default, Resource)]
pub struct JourneyMapViewerState {
    edit_mode: EditingMode,
    editing_type: EditingType,
    editable: bool,
    path: Vec<(f32, f32)>,
    mouse_handling: MouseHandling,
}

pub const EXTENT_SIZE: Extent3d = Extent3d {
    width: 512,
    height: 512,
    depth_or_array_layers: 1,
};

pub async fn load_images(images: Arc<Mutex<Vec<((i32, i32), Box<[u8;512*512*4]>)>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
    let region_offset_x = 0;
    let region_offset_z = 0;

    let stopwatch = std::time::Instant::now();

    let mut threads = JoinSet::new();
    let regions = // reader.get_regions_list().await;
    vec![(0,0)];

    for (region_x, region_z) in regions.into_iter() {
        let region = reader.try_read_region(region_offset_x + region_x, region_offset_z + region_z).await;
        if region.is_some() {
            threads.spawn(async move {
                buffer_region(region.unwrap(), region_offset_x, region_offset_z, region_x, region_z).await
            });
            if let Some(Ok(obj)) = threads.try_join_next() {
                images.lock().as_mut().unwrap().push(obj);
            }
        } else {
            println!("Region not found");
            continue;
        }
    }

    while let Some(result) = threads.join_next().await {
        if let Ok(obj) = result {
            images.lock().as_mut().unwrap().push(obj);  // 画像を保存
        }
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
    /// クリック
    pub fn clicked(&mut self, pos: Vec2) {
        self.mouse_handling.last_mouse_pos = pos;
    }

    /// マウスのドラッグの処理
    pub fn dragging(&mut self, pos: Vec2) -> Vec2 {
        let mut d = (pos - self.mouse_handling.last_mouse_pos) / self.mouse_handling.zoom;
        self.mouse_handling.last_mouse_pos = pos;
        d.x = -d.x;  // xを反転する
        d
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