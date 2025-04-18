mod map;

use bevy::app::App;
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use bevy_egui::egui_node::EguiBevyPaintCallback;

#[derive(Debug, Clone, Default, Resource)]
struct MyApp {
    title: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
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
    window: Single<&mut Window, With<PrimaryWindow>>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.label("Hello, world!");
        if ui.button("Click me!").clicked() {
            println!("Button clicked!");
        }
        ui.text_edit_singleline(&mut ui_state.as_mut().title)
    });

    let pos = UVec2::new(0, 0);
    let size = UVec2::new(window.physical_width(), window.physical_height());

    camera.viewport = Some(Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // カメラを追加（これがないと何も表示されない）
    commands.spawn(Camera2d);

    // 円
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.1, 0.0)))),
        Transform::from_translation(Vec3::new(-150., 0., 0.)),
    ));
}