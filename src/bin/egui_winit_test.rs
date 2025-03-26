use egui::{PlatformOutput, ViewportBuilder, ViewportId};
use egui_winit::State;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let mut app = AhoApp::default();
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Default)]
struct AhoApp {
    window: Option<Window>,
    egui_state: Option<egui_winit::State>,
}

impl ApplicationHandler for AhoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let viewport = ViewportBuilder::default().with_title("あほあほあぷり").with_inner_size((800.0, 600.0));

        let context = egui::Context::default();

        let window = egui_winit::create_window(&context, &event_loop, &viewport).expect("Failed to create window");

        self.window = Some(window);
        let egui_state = State::new(context, ViewportId::default(), self.window.as_ref().unwrap(), None, None, None);
        self.egui_state = Some(egui_state);


        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let egui_state = self.egui_state.as_mut().unwrap();
                let raw_input = egui_state.take_egui_input(self.window.as_ref().unwrap());
                let egui_ctx = egui_state.egui_ctx();
                egui_ctx.begin_pass(raw_input);

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.heading("あほ");
                    ui.horizontal(|ui| {
                        ui.label("あほ");
                        ui.label("ばか");
                        ui.label("まぬけ");
                    });
                    ui.add(egui::widgets::Button::new("あほボタン"));
                    ui.vertical(|ui| {
                        ui.label("あほあほスライダー");
                        ui.add(egui::widgets::Slider::new(&mut 0.5, 0.0..=1.0));
                    });
                    ui.image("https://www.rust-lang.org/logos/rust-logo-512x512.png");
                });

                let output = egui_ctx.end_pass();

                /*let platform_output = PlatformOutput {
                    pixels: output.pixels,
                    screen_rect: output.screen_rect,
                    screen_size: output.screen_size,
                    needs_repaint: output.needs_repaint,
                    events: output.events,
                }*/
                egui_state.handle_platform_output(self.window.as_ref().unwrap(), PlatformOutput::default());
            }
            _ => {}
        }
    }
}