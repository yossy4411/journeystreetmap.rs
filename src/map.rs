use fastanvil::Region;
use fltk::prelude::{GroupExt, InputExt, MenuExt, WidgetBase, WidgetExt};
use iced::mouse::Cursor;
use iced::{Color, Length, Point, Rectangle, Size, Theme, Vector};
use iced_wgpu::core::image::Handle;
use iced_wgpu::core::layout::{Limits, Node};
use iced_wgpu::core::renderer::Style;
use iced_wgpu::core::widget::Tree;
use iced_wgpu::core::{Image, Layout, Widget};
use iced_wgpu::graphics::geometry::{stroke, Cache, Frame, Path, Renderer, Stroke};
use journeystreetmap::journeymap::{biome, JourneyMapReader};
use std::collections::HashMap;
use std::fs::File;
use tiny_skia::Pixmap;


#[derive(Debug, Clone)]
/// 画像の状態を管理する構造体
struct ImageState {
    zoom: f32,
    zoom_factor: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl Default for ImageState {
    fn default() -> Self {
        ImageState {
            zoom: 1.0,
            zoom_factor: 1.25,
            offset_x: 0.0,
            offset_y: 0.0,
            dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Default)]
/// 編集のモード
enum EditingMode {
    #[default]
    Insert,
    Delete,
    Select,
    View,
}

// 編集対象
#[derive(PartialEq, Hash, Copy, Clone, Debug, Default)]
enum EditingType {
    #[default]
    Stroke,  // 線（道路）
    Fill,    // 塗りつぶし（建物）
    Poi,    // ポイント（村、都市、交差点...）
}


#[derive(Debug, Default)]
pub struct JourneyMapViewer {
    images: HashMap<(i32, i32), Image>,  // Regionごとの画像データをキャッシュするためのHashMap
    image_layer_cache: Cache<iced_wgpu::Renderer>,
    fore_layer_cache: Cache<iced_wgpu::Renderer>,
    image_state: ImageState,
}

#[derive(Debug, Default)]
pub struct JourneyMapViewerState {
    image_state: ImageState,
    edit_mode: EditingMode,
    editing_type: EditingType,
    editable: bool,
    path: Vec<(f32, f32)>,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for JourneyMapViewer
where Renderer: iced_wgpu::graphics::geometry::Renderer
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(Size::new(512.0, 512.0)) // レイアウトノードのサイズ
    }

    /* fn update(&self, state: &mut Self::State, event: Event, _bounds: Rectangle, _: Cursor) -> (Status, Option<Message>) {
        match event {
            Event::Mouse(mouse) => {
                match mouse {
                    mouse::Event::ButtonPressed(button) => {
                        if button == mouse::Button::Right && state.edit_mode == EditingMode::Insert {
                            // 右クリックって、パス閉じるのか、それとも1個前のポイントに戻るのか。どっちがいいかな？
                            // → 1個前のポイントに戻るのがいいかな
                        } else if button == mouse::Button::Left {
                            state.image_state.dragging = true;
                            if state.edit_mode == EditingMode::Insert {
                                let x = (state.image_state.last_mouse_x - state.image_state.offset_x) / state.image_state.zoom;
                                let y = (state.image_state.last_mouse_y - state.image_state.offset_y) / state.image_state.zoom;
                                let x = x.round();  // ブロックの位置に丸める
                                let y = y.round();
                                if state.editing_type == EditingType::Poi {
                                    // todo 新しいウィンドウを開いてタグを入力する
                                    // winitで全部やるのは面倒すぎて死ぬ
                                    // 容量が小さくて使い勝手がいいやつ。
                                    // あと使い捨てになるし、ウィンドウで入力した情報が戻り値として返ってくるやつがいいネ
                                    // → GitHub Copilotによると、eguiが適してるって
                                    // 結果的にfltk-rsを使うことにしたyo

                                    let result = {
                                        let mut app = fltk::app::App::default();
                                        let mut wind = fltk::window::Window::new(100, 100, 400, 600, "地点の追加");
                                        let mut frame = fltk::frame::Frame::new(0, 0, 400, 200, "地点の追加");

                                        let mut flex = fltk::group::Flex::new(0, 300, 400, 300, "");
                                        let mut name = fltk::input::Input::new(0, 0, 200, 20, "名前");
                                        let mut desc = fltk::input::Input::new(0, 0, 200, 20, "説明");
                                        let mut category = fltk::menu::Choice::new(0, 0, 200, 20, "カテゴリ");
                                        // あくまでMinecraftのもののカテゴリです。
                                        category.add_choice("自然生成の村");
                                        category.add_choice("自然生成の村の構築物");
                                        category.add_choice("自然生成のその他地上構造物");
                                        category.add_choice("自然生成の地下構造物");
                                        category.add_choice("人工的な市区町村");
                                        category.add_choice("近代的な行政および国家機関");
                                        category.add_choice("公共交通");
                                        category.add_choice("公共施設");
                                        category.add_choice("娯楽施設・観光地");
                                        category.add_choice("地形");
                                        category.add_choice("歴史的建造物");
                                        category.add_choice("宗教的建造物");
                                        category.add_choice("教育施設");
                                        category.add_choice("商業施設");
                                        category.add_choice("産業施設");
                                        category.add_choice("住宅");
                                        category.add_choice("その他");
                                        let mut category2 = fltk::menu::Choice::new(0, 0, 200, 20, "サブカテゴリ");
                                        category2.add_choice("まずはカテゴリを選んでください");
                                        flex.end();
                                        let mut but = fltk::button::Button::new(160, 210, 80, 40, "追加");
                                        wind.end();
                                        wind.show();
                                        category.set_callback(move |cat| {
                                            category2.clear();
                                            let selections = match cat.value() {
                                                0 => {  // 自然生成の村
                                                    vec!["草原の村", "雪原の村", "砂漠の村", "サバンナの村", "タイガの村", "ジャングルの村", "湿地の村", "その他"]
                                                    // ※ ジャングルと湿地は自然生成されない
                                                }
                                                1 => {  // 自然生成の村の構築物
                                                    vec!["村", "村の広場", "村の家", "村の農場", "村の鍛冶屋", "村の神殿", "村の道", "その他"]
                                                }
                                                2 => {  // 自然生成のその他地上構造物
                                                    vec!["森の洋館", "ジャングルの神殿", "ピラミッド", "ピリジャーの前哨基地", "ウィッチの小屋",
                                                         "荒廃したポータル",
                                                         "海底神殿", "海底遺跡", "海底の廃墟", "難破船", "埋もれた宝",
                                                         "イグルー", "井戸",
                                                         "その他"]
                                                }
                                                // note: 旅路の遺跡って地下なのか地上なのか、微妙だよね
                                                3 => { // 自然生成の地下構造物
                                                    vec!["廃坑", "古代都市", "遺跡", "その他"]
                                                }
                                                4 => { // 人工的な市区町村
                                                    vec!["市役所", "区役所", "町役場", "村役場", "その他"]
                                                }
                                                5 => { // 近代的な行政および国家機関
                                                    vec!["国会議事堂", "首相官邸", "大統領官邸", "領事館", "その他"]
                                                }
                                                6 => { // 公共交通
                                                    vec!["鉄道駅", "地下鉄駅", "バス停", "路面電車停留所", "空港", "港", "その他"]
                                                }
                                                7 => { // 公共施設
                                                    vec!["図書館", "美術館", "博物館", "劇場", "コンサートホール", "体育館", "プール", "その他"]
                                                }
                                                8 => { // 娯楽施設・観光地
                                                    vec!["テーマパーク", "動物園", "水族館", "遊園地", "アート", "博物館", "美術館", "コンサートホール", "その他"]
                                                }
                                                9 => { // 地形
                                                    vec!["山", "丘", "谷", "川", "湖", "滝", "洞窟", "花畑", "氷河", "その他"]
                                                }
                                                10 => { // 歴史的建造物
                                                    vec!["その他"]
                                                    // いや冷静に考えてマイクラに歴史的建造物ってあるのか？
                                                }
                                                11 => { // 宗教的建造物
                                                    vec!["神社", "寺院", "教会", "モスク", "推しの宗教", "その他"]
                                                    // あくまで建築物のカテゴリなので、実際の宗教とは関係ない
                                                }
                                                12 => { // 教育施設
                                                    vec!["学校", "大学", "図書館", "育成所", "その他"]
                                                }
                                                13 => { // 商業施設
                                                    vec!["商店", "ショッピングモール", "市場 (交易所)", "その他"]
                                                }
                                                14 => { // 産業施設
                                                    vec!["工場", "倉庫", "採掘場", "農場", "牧場", "トラップ", "その他"]
                                                }
                                                15 => { // 住宅
                                                    vec!["アパート", "マンション", "一戸建て", "別荘", "その他"]
                                                }
                                                _ => { // その他 (or 間違った番号)
                                                    vec!["不明なもの", "未確認", "都市伝説", "その他"]
                                                    // 都市伝説とか怖すぎるわｗ
                                                }
                                            };
                                            for selection in selections {
                                                category2.add_choice(selection);
                                            }
                                        });
                                        but.set_callback(move |_| {
                                            // まじウィンドウ閉じないのどうにかしてほしい
                                            // Waylandのせいかも。X11なら閉れるんちゃう？
                                            wind.deactivate();
                                            app.quit();
                                        });
                                        app.run().unwrap();
                                        name.value()
                                    };
                                    state.editable = false;  // 誤審防止
                                    println!("Name: {}", result);

                                } else {
                                    state.path.push((x, y));
                                }
                            }
                        } else {
                            state.image_state.dragging = false;
                            return (Status::Ignored, None);
                        }
                    }
                    mouse::Event::ButtonReleased(button) => {
                        if button == mouse::Button::Left {
                            state.image_state.dragging = false;
                        }
                    }
                    mouse::Event::CursorMoved { position } => {
                        let dx = position.x - state.image_state.last_mouse_x;
                        let dy = position.y - state.image_state.last_mouse_y;
                        state.image_state.last_mouse_x = position.x;
                        state.image_state.last_mouse_y = position.y;

                        if state.image_state.dragging {
                            state.image_state.offset_x += dx;
                            state.image_state.offset_y += dy;   // Y軸は上下逆
                        } else {
                            return (Status::Ignored, None);
                        }

                    }
                    mouse::Event::WheelScrolled { delta } => {
                        match delta {
                            mouse::ScrollDelta::Lines { x:_, y } => {
                                let factor = if y > 0.0 { state.image_state.zoom_factor } else { 1.0 / state.image_state.zoom_factor };
                                state.image_state.zoom *= factor;

                                let zoom_origin_x = state.image_state.last_mouse_x;
                                let zoom_origin_y = state.image_state.last_mouse_y;
                                state.image_state.offset_x = (state.image_state.offset_x - zoom_origin_x) * factor + zoom_origin_x;
                                state.image_state.offset_y = (state.image_state.offset_y - zoom_origin_y) * factor + zoom_origin_y;
                            }
                            _ => {}
                        }
                    }
                    _ => { return (Status::Ignored, None) }
                };
            }
            _ => {}
        };
        self.image_layer_cache.clear();
        (Status::Captured, None)
    }*/

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        let timestamp = std::time::Instant::now();

        let mut f = Frame::new(renderer, layout.bounds().size());

        f.translate(Vector::new(self.image_state.offset_x, self.image_state.offset_y));
        f.scale(self.image_state.zoom);

        // 画像を最後に描画する（グリッドの下に行かないように）
        for ((rx, rz), img) in &self.images {
            let dest_x = rx * 512;
            let dest_y = rz * 512;
            f.draw_image(Rectangle::new(Point::new(dest_x as f32, dest_y as f32), (512.0, 512.0).into()), img.clone());
        }

        // グリッド（線）を先に描画する
        let stroke = Stroke {
            width: 10.0,
            style: stroke::Style::Solid(Color::from_rgba8(255, 0, 0, 1.0)),
            ..Default::default()
        };
        let path = Path::new(|builder| {
            builder.move_to(Point::new(20.0, 0.0));
            builder.line_to(Point::new(20.0, layout.bounds().height));
        });
        f.stroke(&path, stroke);

        println!("Rendering took {:?}", timestamp.elapsed());

        renderer.start_layer(layout.bounds());
        renderer.draw_geometry(f.into_geometry());
        renderer.end_layer();
    }
}

impl JourneyMapViewer {
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

    pub fn set_insert<Message>(&mut self, message: Message) {
    }
}