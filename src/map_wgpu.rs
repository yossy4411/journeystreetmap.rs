use iced::mouse::Cursor;
use iced::{border, Element, Length, Pixels, Rectangle, Size, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::Wrapping;
use iced_wgpu::core::layout::{Limits, Node};
use iced_wgpu::core::renderer::Style;
use iced_wgpu::core::{renderer, Color, Font, Layout, Text, Widget};
use iced_wgpu::core::text::Renderer;
use iced_wgpu::core::widget::Tree;
use iced_wgpu::graphics::geometry::{Frame, Path};
use iced_wgpu::graphics::geometry::frame::Backend;

pub struct MyGpuWidget;

impl MyGpuWidget {
    pub fn new() -> Self {
        MyGpuWidget {}
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
        let path = Path::new(|builder| {
            builder.line_to((0.0, 0.0).into());
            builder.line_to((100.0, 0.0).into());
            builder.line_to((50.0, 70.0).into());
            builder.close();
        });
        frame.fill(&path, Color::from_rgb(1.0, 0.0, 0.0)); // 赤色で塗りつぶす
        renderer.draw_geometry(frame.into_geometry());
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