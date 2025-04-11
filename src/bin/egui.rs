use eframe::{egui, AppCreator};
use egui::*;

// アプリケーションの状態管理
struct Application {
    // 地震データ
    waveform_data: Vec<f32>,
    // wgpu関連の状態
    render_state: Option<WaveformRenderState>,
    // UI状態
    scale: f32,
    offset: f32,
}

// wgpu描画に必要な状態をまとめる
struct WaveformRenderState {
    // シェーダーやバッファなどのwgpuリソース
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            waveform_data: Vec::new(),
            render_state: None,
            scale: 1.0,
            offset: 0.0,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // UI部分
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Scale:");
                ui.add(Slider::new(&mut self.scale, 0.1..=10.0).text("Scale"));
            });
        });
    }
}

fn main() {
    let viewport = ViewportBuilder {
        inner_size: Some(vec2(300.0, 300.0)),
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Waveform Viewer",
        options,
        Box::new(|_cc: &eframe::CreationContext<'_>| {
            Ok(Box::<Application>::default())
        }),
    ).expect("Failed to run the application");
}