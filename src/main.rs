mod map;

use std::ops::Add;
use crate::map::{load_images, JourneyMapViewerState};
use bevy::app::App;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureDimension, TextureFormat};
use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use std::sync::Arc;
use std::sync::Mutex;
use bevy::input::mouse::MouseWheel;
use bevy::tasks::futures_lite::StreamExt;

#[derive(Debug, Clone, Default, Resource)]
struct MyApp {
    title: String,
    images: Arc<Mutex<Vec<((i32, i32), Box<[u8;512*512*4]>)>>>,
}


fn main() {
    let runner = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build().expect("Failed to create runtime");
    
    let myapp = MyApp::default();
    let arc_clone = myapp.images.clone();
    runner.spawn(async { 
        load_images(arc_clone).await.expect("Failed to load images");
    });
    
    
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins((
            bevy::app::PanicHandlerPlugin,
            bevy::transform::TransformPlugin,
            bevy::input::InputPlugin,
            bevy::window::WindowPlugin::default(),
            bevy::a11y::AccessibilityPlugin,
            bevy::app::TerminalCtrlCHandlerPlugin,
        ))
        .add_plugins((
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::winit::WinitPlugin::<bevy::winit::WakeUp>::default(),
            bevy::render::RenderPlugin::default(),
            bevy::render::texture::ImagePlugin::default_nearest(),
        ))
        .add_plugins((
            bevy::render::pipelined_rendering::PipelinedRenderingPlugin,
            bevy::core_pipeline::CorePipelinePlugin,
            bevy::sprite::SpritePlugin,
            bevy::picking::DefaultPickingPlugins,
        ))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        .insert_resource(myapp)
        .insert_resource(JourneyMapViewerState::default())
        .add_systems(
            Startup,
            (setup, ui_setup)
        )
        .add_systems(
            Update,
            update,
        )
        .add_systems(EguiContextPass, ui_system)
        .run();
}

fn ui_setup(mut contexts: EguiContexts) {
    let ctx_mut = contexts.ctx_mut();
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("Noto Sans JP".to_string(), Arc::new(FontData::from_static(include_bytes!("../fonts/NotoSansJP-Regular.ttf"))));
    fonts.families.insert(FontFamily::Proportional, vec!["Noto Sans JP".to_string()]);
    ctx_mut.set_fonts(fonts);
}

fn ui_system(
    // mut camera: Single<&mut Camera>,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<MyApp>,
) {
    let ctx = contexts.ctx_mut();
    bevy_egui::egui::Window::new("Editor").show(ctx, |ui| {
        ui.label("Hello, world!");
        if ui.button("Click me!").clicked() {
            println!("Button clicked!");
        }
        ui.text_edit_singleline(&mut ui_state.as_mut().title);
    });
}


fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
) {
    window.single_mut().unwrap().ime_enabled = true;
    // カメラを追加（これがないと何も表示されない）
    commands.spawn(Camera2d);
}

fn update(
    mut commands: Commands,
    myapp: Res<MyApp>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut assets: ResMut<Assets<Image>>,
    mut windows: Query<&mut Window>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut state: ResMut<JourneyMapViewerState>,
) {
    // ImageをWorldに落とし込む操作
    for ((region_x, region_z), colors) in myapp.images.lock().as_mut().unwrap().drain(..) {
        let image = Image::new_fill(map::EXTENT_SIZE, TextureDimension::D2, colors.as_ref(), TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
        let image_handle = assets.as_mut().add(image);
        let sprite = Sprite::from_image(image_handle);
        commands.spawn((
            sprite,
            Transform::from_xyz(region_x as f32 * 512.0, 0., region_z as f32 * 512.0),
        ));
        println!("Loaded region: ({}, {})", region_x, region_z);
    }
    let state_ref = state.as_mut();
    if mouse_button.just_pressed(MouseButton::Left) {
        let mut window = windows.single_mut().unwrap();
        if let Some(a) = window.cursor_position() {
            state_ref.clicked(a);
        };
        println!("Left mouse button clicked!");
    }
    let cam_mut = camera.as_mut();
    if mouse_button.pressed(MouseButton::Left) {
        let window = windows.single_mut().unwrap();
        if let Some(cursor_pos) = window.cursor_position() {
            let delta = state_ref.dragging(cursor_pos);
            let scale = cam_mut.scale;
            cam_mut.translation += delta.extend(0.) * scale;
        }
    }

    for event in mouse_wheel.read() {
        let y = event.y;
        let delta = state_ref.zoom(y);
        let window = windows.single().unwrap();
        let center = window.size() / 2.0;
        if let Some(cursor_pos) = window.cursor_position() {
            let mut mouse_pos_rel = cursor_pos - center;
            mouse_pos_rel.x = -mouse_pos_rel.x;
            let scale = camera.as_ref().scale;
            camera.as_mut().scale *= delta;
            camera.as_mut().translation += (mouse_pos_rel * (delta - 1.0)).extend(0.) * scale;
        }
    }

}