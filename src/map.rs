use std::io::Write;
use iced::mouse::Cursor;
use iced::{color, Element, Length, Point, Rectangle, Size};
use iced_tiny_skia::core::layout::{Limits, Node};
use iced_tiny_skia::core::renderer::Style;
use iced_tiny_skia::core::widget::Tree;
use iced_tiny_skia::core::{renderer, Layout, Widget};
use iced_tiny_skia::graphics::Viewport;
use iced::Renderer;
use tiny_skia::{Color, Mask, MaskType, Pixmap, Transform};
use crate::EditingMode::View;

enum Message {

}

#[derive(Debug, Default)]
pub struct JourneyMapViewer {

}

pub fn journey_map_viewer() -> JourneyMapViewer {
    JourneyMapViewer::default()
}


impl<Message, Theme> Widget<Message, Theme, Renderer> for JourneyMapViewer {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(Size::new(0.0, 0.0))
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        let width = viewport.width as u32;
        let height = viewport.height as u32;

        let mut pixmap = Pixmap::new(width, height).unwrap();
        pixmap.fill(Color::WHITE);

        let mut path = tiny_skia::PathBuilder::new();
        // 三角形
        path.move_to(100.0, 100.0);
        path.line_to(200.0, 100.0);
        path.line_to(150.0, 200.0);
        path.close();
        let path = path.finish().unwrap();

        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(255, 0, 0, 255);
        let stroke = tiny_skia::Stroke {width: 3.0, ..Default::default()};
        pixmap.stroke_path(&path, &paint, &stroke, Transform::default(), None);


        let pixmap_ref = pixmap.as_ref();

        let mut mask = Mask::new(width, height).unwrap();
        mask.invert();
        mask.save_png("triangleMask.png").unwrap();
        pixmap.save_png("triangle.png").unwrap();
        let mut pixmap_mut = pixmap.as_mut();
        let size = iced::Size::new(width, height);
        let viewport = Viewport::with_physical_size(size, 1.0);
        renderer.draw::<String>(
            &mut pixmap_mut,
            &mut mask,
            &viewport,
            &[Rectangle::new(Point::default(), viewport.logical_size())],
            iced::Color::from_rgba8(255, 0, 255, 1.0),
            &[]
        );

    }
}


impl<Message, Theme> From<JourneyMapViewer>
for Element<'_, Message, Theme, Renderer>
{

    fn from(viewer: JourneyMapViewer) -> Self {
        Self::new(viewer)
    }
}