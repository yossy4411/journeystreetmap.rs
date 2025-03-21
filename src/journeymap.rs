use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;
use fastanvil::Region;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct PositionData {
    pub biome_name: String,
    pub top_y: i32,
}

#[derive(Deserialize)]
pub struct Chunk {
    pub pos: i64,
    #[serde(flatten, deserialize_with = "deserialize_sections")]
    pub sections: HashMap<(i32, i32), PositionData>
}

fn deserialize_sections<'de, D>(
    deserializer: D,
) -> Result<HashMap<(i32, i32), PositionData>, D::Error>
where
    D: Deserializer<'de>,
{
    // Temporary HashMap<String, PositionData>
    let raw_map = HashMap::<String, PositionData>::deserialize(deserializer)?;

    // New HashMap<(i32, i32), PositionData>
    let mut map = HashMap::new();

    for (key, value) in raw_map {
        // キーを解析して(i32, i32)に変換
        let mut parts = key.split(',');
        if let (Some(part1), Some(part2)) = (parts.next(), parts.next()) {
            let x = i32::from_str(part1).map_err(serde::de::Error::custom)?;
            let y = i32::from_str(part2).map_err(serde::de::Error::custom)?;
            map.insert((x, y), value);
        } else {
            return Err(serde::de::Error::custom("Invalid key format"));
        }
    }

    Ok(map)
}


pub struct JourneyMapReader<> {
    origin: String,
    region: Option<Region<File>>
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
        if self.region.is_none() {
            return Err("Region not loaded".into())
        }
        let region = self.region.as_mut().unwrap();
        let chunk = region.read_chunk(x, z)?;
        if chunk.is_none() {
            return Ok(None)
        }
        let parsed: fastnbt::error::Result<Chunk> = fastnbt::from_bytes(chunk.unwrap().as_slice());
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

impl Chunk {
    pub fn get_all_sorted(&self) -> Vec<(&(i32, i32), &PositionData)> {
        // sort by x, then y
        let mut vec: Vec<_> = self.sections.iter().collect();
        vec.sort_by(|a, b| a.0.0.cmp(&b.0.0).then(a.0.1.cmp(&b.0.1)));
        vec
    }
}