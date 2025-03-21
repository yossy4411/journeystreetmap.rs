mod journeymap;

fn main() {
    let mut reader = journeymap::JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
    let chunk_x = 0;
    let chunk_z = 0;
    let chunk = reader.get_chunk_at(chunk_x, chunk_z).expect("Failed to read region");

    if chunk.is_none() {
        println!("Chunk not found");
        return;
    } let chunk = chunk.unwrap();

    println!("Chunk pos: {}", chunk.pos);


    for i in 0..=chunk_x {
        for j in 0..=chunk_z {
            let chunk = reader.get_chunk_at(i, j).expect("Failed to read region");
            if chunk.is_none() {
                println!("Chunk not found");
                return;
            } let chunk = chunk.unwrap();
            println!("Chunk pos: {}", chunk.pos);
            let mut vec = chunk.get_all_sorted();
            for ((x, y), data) in vec {
                println!("{}, {}: {:?}", x, y, data.biome_name);
            }
        }
    }
}
