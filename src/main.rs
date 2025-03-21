use std::hint::black_box;

mod journeymap;

fn main() {
    let mut reader = journeymap::JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
    let region_x = -1;
    let region_z = -1;
    reader.read_region(region_x, region_z).expect("Failed to read region");

    let stopwatch = std::time::Instant::now();
    for i in 0..=31 {
        for j in 0..=31 {
            if let Ok(chunk) = reader.get_chunk(i, j) {
                if chunk.is_none() {
                    println!("Chunk not found");
                    continue;
                } let chunk = chunk.unwrap();
                // println!("Chunk pos: {}", chunk.pos);

                for (_, data) in chunk.sections {
                    // println!("{:?}", data.biome_name);
                    black_box(data);
                }
            } else {
                println!("Chunk load failed");
                continue;
            }
        }
    }
    println!("Time taken: {:?}", stopwatch.elapsed());
}
