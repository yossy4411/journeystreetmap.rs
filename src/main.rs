use std::env;
use fastanvil::asyncio::Region;
use pmtiles2::{Compression, TileType};
use tokio::fs::File;
use tokio::task::JoinSet;
use journeystreetmap::journeymap::biome::RGB;
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use journeystreetmap::log::Status;

extern crate env_logger as logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    logger::init();

    info!("[1/4] JourneyMap Map Data to Bitmap (RAW) conversion");

    let mut reader = JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
    info!("JourneyMapReader initialized");
    info!("Start adding threads...");
    let region_offset_x = 0;
    let region_offset_z = 0;

    let mut threads = JoinSet::new();
    let regions = reader.get_regions_list().await;
    let mut images = Vec::new();

    let mut status = Status::new("Add thread pool".to_string(), regions.len() as u32);

    for (region_x, region_z) in regions.into_iter() {
        let region = reader.try_read_region(region_offset_x + region_x, region_offset_z + region_z).await;
        if region.is_some() {
            threads.spawn(async move {
                buffer_region(region.unwrap(), region_offset_x, region_offset_z, region_x, region_z).await
            });
        } else {
            println!("Region not found");
            continue;
        }
        status.update();
    }
    status.finish();
    info!("Start processing regions...");

    let mut status = Status::new("Processing regions".to_string(), threads.len() as u32);
    while let Some(result) = threads.join_next().await {
        if let Ok(obj) = result {
            images.push(obj);  // 画像を保存
        }
        status.update();
    }
    status.finish();
    info!("Finished processing regions.");
    info!("Loaded {} regions", images.len());

    info!("[2/4] Convert Bitmap (RAW) to WEBP");

    let mut status = Status::new("Convert to WEBP".to_string(), images.len() as u32);

    let mut webp_images = Vec::new();
    for (pos, image) in images.iter() {
        let encoder = webp::Encoder::from_rgba(image.as_slice(), 512, 512);
        let webp_image = encoder.encode_lossless();
        webp_images.push((pos, webp_image));
        status.update();
    }
    status.finish();
    info!("Finished converting WEBP images.");

    info!("[3/4] Pack WEBP images into PMTiles");
    let pt = pmtiles2::PMTiles::new(TileType::WebP, Compression::GZip);
    // todo! implement this
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