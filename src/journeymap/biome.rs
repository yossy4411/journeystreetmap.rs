#[derive(Debug, Clone, Copy, Default)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub fn get_color(biome_name: &str) -> RGB {
    match biome_name {
        // Overworld - Ocean
        "minecraft:ocean" => RGB::new(0, 0, 160),
        "minecraft:deep_ocean" => RGB::new(0, 0, 112),
        "minecraft:warm_ocean" => RGB::new(0, 128, 255),
        "minecraft:lukewarm_ocean" => RGB::new(0, 100, 200),
        "minecraft:cold_ocean" => RGB::new(0, 50, 150),
        "minecraft:deep_cold_ocean" => RGB::new(0, 30, 100),
        "minecraft:deep_frozen_ocean" => RGB::new(0, 20, 80),
        "minecraft:deep_lukewarm_ocean" => RGB::new(0, 80, 160),
        "minecraft:frozen_ocean" => RGB::new(100, 150, 255),

        // Overworld - Rivers & Beaches
        "minecraft:river" => RGB::new(50, 100, 255),
        "minecraft:frozen_river" => RGB::new(150, 200, 255),
        "minecraft:beach" => RGB::new(240, 230, 140),
        "minecraft:snowy_beach" => RGB::new(230, 250, 250),
        "minecraft:stony_shore" => RGB::new(160, 160, 160),

        // Overworld - Land (Grass Adjusted)
        "minecraft:plains" => RGB::new(80, 200, 80),
        "minecraft:sunflower_plains" => RGB::new(90, 210, 90),
        "minecraft:forest" => RGB::new(30, 130, 30),
        "minecraft:flower_forest" => RGB::new(50, 140, 50),
        "minecraft:birch_forest" => RGB::new(50, 160, 60),
        "minecraft:dark_forest" => RGB::new(20, 80, 20),
        "minecraft:old_growth_birch_forest" => RGB::new(60, 180, 70),
        "minecraft:old_growth_pine_taiga" => RGB::new(50, 150, 50),
        "minecraft:old_growth_spruce_taiga" => RGB::new(40, 140, 40),
        "minecraft:savanna" => RGB::new(180, 160, 60),
        "minecraft:savanna_plateau" => RGB::new(160, 140, 50),
        "minecraft:taiga" => RGB::new(50, 100, 50),
        "minecraft:snowy_taiga" => RGB::new(200, 255, 255),
        "minecraft:snowy_plains" => RGB::new(240, 240, 255),
        "minecraft:jungle" => RGB::new(30, 150, 70),
        "minecraft:sparse_jungle" => RGB::new(40, 140, 60),
        "minecraft:bamboo_jungle" => RGB::new(50, 180, 80),

        // Overworld - Hills & Mountains
        "minecraft:windswept_hills" => RGB::new(100, 100, 100),
        "minecraft:windswept_gravelly_hills" => RGB::new(90, 90, 90),
        "minecraft:windswept_forest" => RGB::new(70, 110, 70),
        "minecraft:windswept_savanna" => RGB::new(170, 150, 50),
        "minecraft:meadow" => RGB::new(100, 180, 100),
        "minecraft:cherry_grove" => RGB::new(255, 160, 190),
        "minecraft:stony_peaks" => RGB::new(120, 120, 120),
        "minecraft:snowy_slopes" => RGB::new(220, 220, 255),
        "minecraft:jagged_peaks" => RGB::new(210, 210, 255),
        "minecraft:frozen_peaks" => RGB::new(200, 200, 255),

        // Overworld - Swamps
        "minecraft:swamp" => RGB::new(40, 90, 40),
        "minecraft:mangrove_swamp" => RGB::new(70, 100, 50),

        // Overworld - Badlands & Deserts
        "minecraft:desert" => RGB::new(250, 210, 100),
        "minecraft:badlands" => RGB::new(200, 130, 80),
        "minecraft:eroded_badlands" => RGB::new(180, 110, 70),
        "minecraft:wooded_badlands" => RGB::new(170, 120, 80),

        // Overworld - Snow & Ice
        "minecraft:ice_spikes" => RGB::new(160, 230, 255),

        // Nether
        "minecraft:nether_wastes" => RGB::new(110, 30, 30),
        "minecraft:crimson_forest" => RGB::new(170, 20, 20),
        "minecraft:warped_forest" => RGB::new(20, 140, 160),
        "minecraft:soul_sand_valley" => RGB::new(140, 110, 80),
        "minecraft:basalt_deltas" => RGB::new(70, 70, 70),

        // End
        "minecraft:the_end" => RGB::new(190, 190, 190),
        "minecraft:end_highlands" => RGB::new(170, 170, 170),
        "minecraft:end_midlands" => RGB::new(160, 160, 160),
        "minecraft:small_end_islands" => RGB::new(150, 150, 150),
        "minecraft:end_barrens" => RGB::new(130, 130, 130),

        // Cave Biomes
        "minecraft:dripstone_caves" => RGB::new(130, 90, 70),
        "minecraft:lush_caves" => RGB::new(50, 160, 80),

        // Other
        _ => RGB::new(255, 0, 255), // 未知のバイオームはMagentaで表示
    }
}