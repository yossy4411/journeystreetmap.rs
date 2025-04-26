use fastanvil::asyncio::Region;
use tokio::fs::File;
use tokio::task::JoinSet;
use journeystreetmap::journeymap::biome::RGB;
use journeystreetmap::journeymap::{biome, JourneyMapReader};

#[tokio::main]
async fn main() {
    let mut reader = JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
    let region_offset_x = 0;
    let region_offset_z = 0;

    let stopwatch = std::time::Instant::now();

    let mut threads = JoinSet::new();
    let regions = reader.get_regions_list().await;
    let mut images = Vec::new();

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
    }

    while let Some(result) = threads.join_next().await {
        if let Ok(obj) = result {
            images.push(obj);  // 画像を保存
        }
    }
    println!("Time taken: {:?}", stopwatch.elapsed());
    
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