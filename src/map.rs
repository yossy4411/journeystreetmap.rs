use iced_tiny_skia::core::layout::{Limits, Node};
use iced_tiny_skia::core::mouse::Cursor;
use iced_tiny_skia::core::renderer::Style;
use iced_tiny_skia::core::widget::Tree;
use iced_tiny_skia::core::{border, renderer, Background, Color, Element, Layout, Length, Point, Rectangle, Size, Widget};
use iced_tiny_skia::graphics::Viewport;
use iced_tiny_skia::Renderer;
use tiny_skia::{Mask, PathBuilder, Pixmap, Stroke, Transform};

enum Message {

}

#[derive(Debug, Default)]
pub struct JourneyMapViewer {
    cache: iced::widget::canvas::Cache,
}

pub fn journey_map_viewer() -> JourneyMapViewer {
    JourneyMapViewer::default()
}


impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for JourneyMapViewer
where Renderer: renderer::Renderer
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(Size::new(200.0, 200.0))
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        let mut pixmap = Pixmap::new(200, 200).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);
        self.cache.draw(renderer, viewport.size(), |a| {

        });
        let mut path = PathBuilder::new();
        // 三角形
        path.move_to(100.0, 0.0);
        path.line_to(200.0, 200.0);
        path.line_to(0.0, 200.0);
        path.close();
        let path = path.finish().unwrap();

        let mut paint = tiny_skia::Paint::default();
        paint.set_color(tiny_skia::Color::from_rgba8(0, 0, 0, 255));
        pixmap.stroke_path(&path, &paint, &Stroke{width: 6.0, ..Default::default()}, Transform::identity(), None);
        renderer.draw::<String>(
            &mut pixmap.as_mut(),
            &mut Mask::new(200, 200).unwrap(),
            &Viewport::with_physical_size(Size::new(200, 200), 1.0),
            &[layout.bounds()],
            Color::BLACK,
            &[]
        )

    }
}


impl<Message, Theme> From<JourneyMapViewer>
for Element<'_, Message, Theme, Renderer>
where Renderer: renderer::Renderer,
{
    fn from(viewer: JourneyMapViewer) -> Self {
        Self::new(viewer)
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