use fastanvil::Region;
use serde::{Deserialize};
use std::collections::HashMap;
use std::fs::File;

#[derive(Deserialize)]
pub struct PositionData {
    pub blockstates: HashMap<i32, BlockState>,
    pub biome_name: String,
    pub top_y: i32,
}

#[derive(Deserialize)]
pub struct Chunk {
    pub pos: i64,
    #[serde(flatten)]
    pub sections: HashMap<String, PositionData>
}

#[derive(Deserialize)]
pub struct BlockState {
    pub name: String,
    pub properties: HashMap<String, String>
}


pub struct JourneyMapReader<> {
    origin: String,
    region: Option<Region<File>>,
}

impl JourneyMapReader {
    pub fn new(origin: &str) -> JourneyMapReader {
        JourneyMapReader {
            origin: origin.to_string(),
            region: None
        }
    }

    pub fn read_region(&mut self, x: i32, z: i32) -> Result<(), Box<dyn std::error::Error>> {
        let filename = self.origin.clone() + &format!("overworld/cache/r.{}.{}.mca", x, z);
        let stream = File::open(filename)?;
        self.region = Some(Region::from_stream(stream)?);
        Ok(())
    }

    pub fn get_chunk(&mut self, x: usize, z: usize) -> Result<Option<Chunk>, Box<dyn std::error::Error>> {

        let region = self.region.as_mut().ok_or("Region not loaded")?;
        let chunk = region.read_chunk(x, z)?.ok_or("Chunk not found")?;

        // let parsed: fastnbt::error::Result<Chunk> = fastnbt::from_bytes(chunk.unwrap().as_slice());
        let parsed: fastnbt::error::Result<Chunk> = fastnbt::from_bytes(&chunk);
        match parsed {
            Ok(chunk) => Ok(Some(chunk)),
            Err(e) => Err(e.into())
        }
    }

    // Gets the chunk at the specified chunk coordinates
    pub fn get_chunk_at(&mut self, x: i32, z: i32) -> Result<Option<Chunk>, Box<dyn std::error::Error>> {
        let region_x = (x as f32 / 32.0).floor() as i32;
        let region_z = (z as f32 / 32.0).floor() as i32;
        let chunk_x = Self::positive_modulo(x,32);
        let chunk_z = Self::positive_modulo(z,32);  // Positive modulo
        self.read_region(region_x, region_z)?;
        self.get_chunk(chunk_x as usize, chunk_z as usize)
    }

    pub fn positive_modulo(x: i32, m: i32) -> i32 {
        (x % m + m) % m
    }
}
