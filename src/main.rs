mod map;

use bevy::app::{plugin_group, App};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};

#[derive(Debug, Clone, Default, Resource)]
struct MyApp {
    title: String,
}


fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            bevy::app::PanicHandlerPlugin,
            bevy::transform::TransformPlugin,
        ))
        .add_plugins((
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
            bevy::render::texture::ImagePlugin::default(),
        ))
        .add_plugins((
            bevy::render::pipelined_rendering::PipelinedRenderingPlugin,
            bevy::core_pipeline::CorePipelinePlugin,
            bevy::sprite::SpritePlugin,
            bevy::state::app::StatesPlugin,
            bevy::picking::DefaultPickingPlugins,
        ))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        .init_resource::<MyApp>()
        .add_systems(
            Startup,
            setup,
        )
        .add_systems(EguiContextPass, ui_system)
        .run();
}

fn ui_system(
    mut camera: Single<&mut Camera>,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<MyApp>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut window: Query<&mut Window>,
) {
    window.single_mut().unwrap().ime_enabled = true;
    // カメラを追加（これがないと何も表示されない）
    commands.spawn(Camera2d);

    // 円
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.1, 0.0)))),
        Transform::from_translation(Vec3::new(-150., 0., 0.)),
    ));

    // 四角
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100., 100.))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.0, 0.1, 0.2)))),
        Transform::from_translation(Vec3::new(150., 0., 0.)),
    ));
}
    