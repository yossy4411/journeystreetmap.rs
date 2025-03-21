mod journeymap;

fn main() {
    let mut reader = journeymap::JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
    reader.read_region(-1, -1).expect("Failed to read region");

    let stopwatch = std::time::Instant::now();
    for i in 0..=31 {
        for j in 0..=31 {
            if let Ok(chunk) = reader.get_chunk(i, j) {
                if chunk.is_none() {
                    println!("Chunk not found");
                    continue;
                } let chunk = chunk.unwrap();
                // println!("Chunk pos: {}", chunk.pos);
                let mut vec = chunk.get_all_sorted();
                for ((x, y), data) in vec {
                    // println!("{}, {}: {:?}", x, y, data.biome_name);
                }
            } else {
                println!("Chunk load failed");
                continue;
            }
        }
    }
    println!("Time taken: {:?}", stopwatch.elapsed());
}
