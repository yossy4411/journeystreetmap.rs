use std::time::Instant;

// ステータスを表示するための構造体
pub struct Status {
    title: String,
    total: u32,
    current: u32,
    stopwatch: Instant,
    is_first: bool,
}

impl Status {
    pub fn new(title: String, total: u32) -> Self {
        Self {
            title,
            total,
            current: 0,
            stopwatch: Instant::now(),
            is_first: true,
        }
    }

    pub fn update(&mut self) {
        self.current += 1;

        let elapsed = self.stopwatch.elapsed();
        if self.is_first {
            self.is_first = false;
        } else {
            // 1行上にカーソルを移動
            print!("\x1b[1A");

            // 行を消すために空の行を上書きする
            print!("\x1b[2K");
        }
        println!(
            "Progress {}: {}/{} - Elapsed time: {:?}",
            self.title, self.current, self.total, elapsed
        );
    }
    
    pub fn finish(&self) {
        println!("Finished {} {} regions in {:?}", self.title, self.total, self.stopwatch.elapsed());
    }
}