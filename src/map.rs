use iced::widget::canvas::Geometry;
use iced::widget::canvas;
use iced::Theme;
use iced_tiny_skia::core::layout::{Limits, Node};
use iced_tiny_skia::core::mouse::Cursor;
use iced_tiny_skia::core::widget::Tree;
use iced_tiny_skia::core::{border, renderer, Color, Element, Layout, Length, Point, Rectangle, Size, Widget};
use iced_tiny_skia::Renderer;
use tiny_skia::{PathBuilder, Pixmap, Stroke, Transform};

enum Message {

}

#[derive(Debug, Default)]
pub struct JourneyMapViewer {
    cache: iced::widget::canvas::Cache,
}

pub fn journey_map_viewer() -> JourneyMapViewer {
    JourneyMapViewer::default()
}



impl<Message> canvas::Program<Message> for JourneyMapViewer
where Renderer: renderer::Renderer
{
    type State = ();
    fn draw(&self, state: &Self::State, renderer: &iced::Renderer, theme: &Theme, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry<iced::Renderer>> {

        let geom = self.cache.draw(renderer, bounds.size(), |a| {
            let mut pixmap = Pixmap::new(200, 200).unwrap();
            pixmap.fill(tiny_skia::Color::WHITE);
            let mut path = PathBuilder::new();
            // 三角形
            path.move_to(100.0, 40.0);
            path.line_to(150.0, 180.0);
            path.line_to(50.0, 180.0);
            path.close();
            let path = path.finish().unwrap();

            let mut paint = tiny_skia::Paint::default();
            paint.set_color(tiny_skia::Color::from_rgba8(0, 0, 0, 255));
            pixmap.stroke_path(&path, &paint, &Stroke{width: 6.0, ..Default::default()}, Transform::identity(), None);
            let image = iced_tiny_skia::core::image::Handle::from_rgba(
                200,
                200,
                pixmap.data().to_vec(),
            );
            a.fill_rectangle(bounds.position(), bounds.size(), Color::WHITE);
            a.draw_image(Rectangle::with_size(Size::new(200.0, 200.0)), iced_tiny_skia::core::Image::new(image));
        });

        vec![geom]
    }
}


pub struct Circle {
    radius: f32,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

pub fn circle(radius: f32) -> Circle {
    Circle::new(radius)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Circle
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &Limits,
    ) -> Node {
        Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(self.radius),
                ..renderer::Quad::default()
            },
            Color::BLACK,
        );
    }
}

impl<Message, Theme, Renderer> From<Circle>
for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(circle: Circle) -> Self {
        Self::new(circle)
    }
}