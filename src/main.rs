mod map;

use bevy::app::App;
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};
use bevy_egui::egui::TextEdit;

#[derive(Debug, Clone, Default, Resource)]
struct MyApp {
    title: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
    