mod map;

use crate::map::{load_images, EditingMode, EditingType, JourneyMapViewerState};
use bevy::app::App;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureDimension, TextureFormat};
use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use std::sync::Arc;
use std::sync::Mutex;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;

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
            TransformPlugin,
            bevy::input::InputPlugin,
            WindowPlugin::default(),
            bevy::a11y::AccessibilityPlugin,
            bevy::app::TerminalCtrlCHandlerPlugin,
        ))
        .add_plugins((
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::winit::WinitPlugin::<bevy::winit::WakeUp>::default(),
            bevy::render::RenderPlugin::default(),
            ImagePlugin::default_nearest(),
        ))
        .add_plugins((
            bevy::render::pipelined_rendering::PipelinedRenderingPlugin,
            bevy::core_pipeline::CorePipelinePlugin,
            bevy::sprite::SpritePlugin,
            DefaultPickingPlugins,
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
            (
                reading_image,
                camera_handling,
            )
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
    mut state: ResMut<JourneyMapViewerState>,
) {
    let ctx = contexts.ctx_mut();
    bevy_egui::egui::Window::new("Editor").show(ctx, |ui| {
        ui.label("Hello, world!");
        let mode_str = match state.as_ref().editing_mode() {
            EditingMode::Delete => "削除",
            EditingMode::Insert => "挿入",
            EditingMode::Select => "選択",
            EditingMode::View => "閲覧",
        };
        let type_str = match state.as_ref().editing_type() {
            EditingType::Fill => "塗りつぶし (建物ポリゴン)",
            EditingType::Stroke => "線引き (道路など)",
            EditingType::Poi => "POI (マーカー)",
        };
        
        ui.label(format!("モード: {}", mode_str));
        ui.label(format!("編集の種別: {}", type_str));
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

fn reading_image (
    mut commands: Commands,
    myapp: Res<MyApp>,
    mut assets: ResMut<Assets<Image>>,
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
}

fn camera_handling(
    mut state: ResMut<JourneyMapViewerState>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut windows: Query<&mut Window>,
    mut keys: EventReader<KeyboardInput>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    let state_ref = state.as_mut();
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single_mut().unwrap();
        if let Some(a) = window.cursor_position() {
            state_ref.clicked(a);
        };
        println!("Left mouse button clicked!");
    }
    let cam_mut = camera.as_mut();


    for event in mouse_wheel.read() {
        let y = event.y;
        let delta = state_ref.zoom(y);
        let window = windows.single().unwrap();
        let center = window.size() / 2.0;
        if let Some(cursor_pos) = window.cursor_position() {
            let mut mouse_pos_rel = cursor_pos - center;
            mouse_pos_rel.x = -mouse_pos_rel.x;
            let scale = cam_mut.scale;
            cam_mut.scale *= delta;
            cam_mut.translation += (mouse_pos_rel * (delta - 1.0)).extend(0.) * scale;
        }
    }

    let mut shifted = false;
    for event in keys.read() {
        match event.key_code {
            KeyCode::KeyE => {
                state_ref.toggle_editing_type();
            }
            KeyCode::KeyI => {
                state_ref.set_editing_mode(EditingMode::Insert);
            }
            KeyCode::KeyD => {
                state_ref.set_editing_mode(EditingMode::Delete);
            }
            KeyCode::KeyS => {
                state_ref.set_editing_mode(EditingMode::Select);
            }
            KeyCode::KeyV => {
                state_ref.set_editing_mode(EditingMode::View);
            }
            _ => {

            }
        }
        if event.key_code == KeyCode::ShiftLeft || event.key_code == KeyCode::ShiftRight {
            shifted = true;
        }
    }

    if shifted {
        if mouse_button.just_pressed(MouseButton::Left) {
            match state_ref.editing_mode() {
                EditingMode::Insert => {
                    // 何かしらのインサート処理をする
                    println!("インサートするぜ！！");
                }
                _ => {}
            }
        }
    } else if mouse_button.pressed(MouseButton::Left) {
        let window = windows.single().unwrap();
        if let Some(cursor_pos) = window.cursor_position() {
            let delta = state_ref.dragging(cursor_pos);
            let scale = cam_mut.scale;
            cam_mut.translation += delta.extend(0.) * scale;
        }
    }
}