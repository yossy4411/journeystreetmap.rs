use std::collections::HashMap;
use std::fs::File;
use fastanvil::Region;
use iced::mouse::Cursor;
use iced::{border, Element, Length, Pixels, Point, Rectangle, Size, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::Image;
use iced::widget::text::Wrapping;
use iced_wgpu::core::layout::{Limits, Node};
use iced_wgpu::core::renderer::Style;
use iced_wgpu::core::{renderer, Color, Font, Layout, Text, Widget};
use iced_wgpu::core::image::Handle;
use iced_wgpu::core::text::Renderer;
use iced_wgpu::core::widget::Tree;
use iced_wgpu::graphics::geometry::{Frame, Path};
use iced_wgpu::graphics::geometry::frame::Backend;
use tiny_skia::Pixmap;
use journeystreetmap::journeymap::{biome, JourneyMapReader};

pub struct MyGpuWidget {images: HashMap<(i32, i32), Image>}

impl MyGpuWidget {
    pub fn new() -> Self {
        MyGpuWidget {
            images: HashMap::new(),
        }
    }
}

impl<Theme, Message, Renderer> Widget<Message, Theme, Renderer> for MyGpuWidget
where Renderer: iced_wgpu::graphics::geometry::Renderer + iced_wgpu::core::text::Renderer, {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill) // ウィジェットのサイズ
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(Size::new(512.0, 512.0)) // レイアウトノードのサイズ
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(10.0),
                ..renderer::Quad::default()
            },
            Color::from_rgb(0.5, 0.5, 0.5), // 色
        );
        let mut frame = renderer.new_frame(layout.bounds().size());
        let mut frame2 = renderer.new_frame(layout.bounds().size());
        // 画像を描画
        for ((rx, rz), img) in &self.images {
            let dest_x = rx * 512;
            let dest_y = rz * 512;
            frame.draw_image(Rectangle::new(Point::new(dest_x as f32, dest_y as f32), (512.0, 512.0).into()), img.clone());
        }

        // 画像の上に三角形を描画
        let path = Path::new(|builder| {
            builder.line_to((0.0, 0.0).into());
            builder.line_to((100.0, 0.0).into());
            builder.line_to((50.0, 70.0).into());
            builder.close();
        });
        frame2.fill(&path, Color::from_rgb(1.0, 0.0, 0.0)); // 赤色で塗りつぶす

        // 画像の上に三角形が描画されないので、レイヤーを分けることにする
        // これは一種のバグと思われる
        renderer.start_layer(layout.bounds());
        renderer.draw_geometry(frame.into_geometry());
        renderer.end_layer();
        renderer.start_layer(layout.bounds());
        renderer.draw_geometry(frame2.into_geometry());
        renderer.end_layer();

        let text = Text {
            content: "Hello, World!".to_string(),
            font: renderer.default_font(),
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Top,
            shaping: Default::default(),
            size: Pixels(15.0),
            bounds: Size::new(200.0, 50.0),
            line_height: Default::default(),
            wrapping: Wrapping::None,
        };
        renderer.fill_text(text, (30.0, 60.0).into(), Color::from_rgb(1.0, 0.7, 0.7), layout.bounds()); // テキストを描画

    }
}

impl MyGpuWidget {
    pub fn load_images(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = JourneyMapReader::new("/home/okayu/.local/share/ModrinthApp/profiles/Fabulously Optimized/journeymap/data/mp/160~251~235~246/");
        let region_offset_x = 0;
        let region_offset_z = 0;

        let stopwatch = std::time::Instant::now();

        let mut threads = Vec::new();
        let mut regions = // reader.get_regions_list();
            [(-1, -1), (0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

        for (i, (region_x, region_z)) in regions.into_iter().enumerate() {
            let region = reader.try_read_region(region_offset_x + region_x, region_offset_z + region_z);
            if let Some(mut region) = region {
                let thr = std::thread::spawn(move || {
                    ((region_x, region_z), Self::buffer_region(&mut region, region_offset_x, region_offset_z, region_x, region_z))
                });
                threads.push(thr);
            } else {
                println!("Region not found");
                continue;
            }
            if i > 20 {
                break;
            }
        }

        for thr in threads {
            let (key, content) = thr.join().unwrap();
            self.images.insert(key, content);
        }
        println!("Time taken: {:?}", stopwatch.elapsed());
        Ok(())
    }

    fn buffer_region(region: &mut Region<File>, region_offset_x: i32, region_offset_z: i32, region_x: i32, region_z: i32) -> Image {
        let mut pixmap = Pixmap::new(512, 512).unwrap();
        let image_data = pixmap.pixels_mut();
        for i in 0..=31 {
            for j in 0..=31 {
                let chunk_result = JourneyMapReader::get_chunk(region, i, j);
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
                                let color: tiny_skia::Color = color.into();
                                image_data[i] = color.premultiply().to_color_u8()
                            }
                        }
                    }
                }
            }
        }
        let handle = Handle::from_rgba(512, 512, pixmap.take());
        Image::new(handle)
    }
}