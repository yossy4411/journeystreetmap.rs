use fastanvil::Region;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

pub mod biome;
mod decoration;

#[derive(Deserialize)]
pub struct PositionData {
    pub blockstates: HashMap<String, BlockState>,
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
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties")]
    pub properties: Option<HashMap<String, String>>
}


pub struct JourneyMapReader<> {
    origin: String,
}

impl JourneyMapReader {
    pub fn new(origin: &str) -> JourneyMapReader {
        JourneyMapReader {
            origin: origin.to_string(),
        }
    }

    pub fn get_regions_list(&self) -> Vec<(i32, i32)> {
        let mut regions = Vec::new();
        let path = std::path::Path::new(&self.origin).join("overworld/cache");
        for entry in path.read_dir().expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            if filename.starts_with("r.") && filename.ends_with(".mca") {
                let splited: Vec<&str> = filename.split('.').collect();
                let x = splited[1].parse().unwrap();
                let z = splited[2].parse().unwrap();
                regions.push((x, z));
            }
        }
        regions
    }

    pub fn read_region(&mut self, x: i32, z: i32) -> Result<Region<File>, Box<dyn std::error::Error>> {
        let filename = self.origin.clone() + &format!("overworld/cache/r.{}.{}.mca", x, z);
        let stream = File::open(filename)?;
        let region = Region::from_stream(stream)?;
        Ok(region)
    }

    pub fn try_read_region(&mut self, x: i32, z: i32) -> Option<Region<File>> {
        let filename = self.origin.clone() + &format!("overworld/cache/r.{}.{}.mca", x, z);
        if !std::path::Path::new(&filename).exists() {
            return None;
        }
        let stream = File::open(filename);
        match stream {
            Ok(stream) => {
                if stream.metadata().unwrap().len() == 0 {
                    return None;   // 空っぽファイルはいらないよ
                }
                match Region::from_stream(stream) {
                    Ok(region) => {
                        Some(region)
                    }
                    Err(..) => {
                        None
                    }
                }
            },
            Err(_) => None
        }
    }

    pub fn get_chunk<T>(region: &mut Region<T>, x: usize, z: usize) -> Result<Option<Chunk>, Box<dyn std::error::Error>>
    where T: std::io::Read + std::io::Seek {
        let chunk = region.read_chunk(x, z)?.ok_or("Chunk not found")?;

        // let parsed: fastnbt::error::Result<Chunk> = fastnbt::from_bytes(chunk.unwrap().as_slice());
        let parsed: fastnbt::error::Result<Chunk> = fastnbt::from_bytes(&chunk);
        match parsed {
            Ok(chunk) => Ok(Some(chunk)),
            Err(e) => Err(e.into())
        }
    }
/*
    // Gets the chunk at the specified chunk coordinates
    pub fn get_chunk_at(&mut self, x: i32, z: i32) -> Result<Option<Chunk>, Box<dyn std::error::Error>> {
        let region_x = (x as f32 / 32.0).floor() as i32;
        let region_z = (z as f32 / 32.0).floor() as i32;
        let chunk_x = Self::positive_modulo(x,32);
        let chunk_z = Self::positive_modulo(z,32);  // Positive modulo
        self.read_region(region_x, region_z)?;
        self.get_chunk(chunk_x as usize, chunk_z as usize)
    }*/

    pub fn positive_modulo(x: i32, m: i32) -> i32 {
        (x % m + m) % m
    }
}
