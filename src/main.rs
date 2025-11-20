#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

mod autofocus;

use bevy::{
    core_pipeline::{
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
        tonemapping::Tonemapping,
        experimental::taa::TemporalAntiAliasing,
        bloom::Bloom,
        post_process::ChromaticAberration,
    },
    dev_tools::fps_overlay::FpsOverlayPlugin,
    pbr::DefaultOpaqueRendererMethod,
    pbr::ScreenSpaceReflections,
    prelude::*,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use camera_controller::{CameraController, CameraControllerPlugin};

fn main() {
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraControllerPlugin)
        .add_plugins(autofocus::AutoFocusDofPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_freecam)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_translation(Vec3::new(-10.0, 5.0, 25.0) * 0.4)
            .looking_at(Vec3::new(1.2, -2.0, 0.0), Vec3::Y),
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.1,
            composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode::Additive,
            ..default()
        },
        ScreenSpaceReflections {
            perceptual_roughness_threshold: 0.5,
            linear_steps: 64,
            linear_march_exponent: 1.2,
            bisection_steps: 16,
            use_secant: false,
            ..default()
        },
        DepthPrepass,
        MotionVectorPrepass,
        DeferredPrepass,
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ChromaticAberration::default(),
        /*/
        DepthOfField {
            aperture_f_stops: 5.0,
            ..default()
        },

        CameraController {
            walk_speed: 3.0,
            run_speed: 10.0,
            ..Default::default()
        },
        */
    ));
    commands.spawn(DirectionalLight {
        illuminance: 0.0,
        ..Default::default()
    });
    
    commands.insert_resource(AmbientLight {
        color: Color::BLACK,
        brightness: 0.0,
        ..default()
    });

    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronMap_Base.glb")),
        ),
        Transform::from_xyz(0.0, -1.0, 0.0),
        
    ));

    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronMap_Neon.glb")),
        ),
        Transform::from_xyz(0.0, -1.0, 0.0),
        autofocus::FocusTarget,

    ));

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronBike.glb"))),
        Transform::from_xyz(0.0, -1.0, 0.0),
        autofocus::FocusTarget,
    ));
}

fn toggle_freecam(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, Option<&CameraController>), With<Camera3d>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        for (entity, controller) in query.iter_mut() {
            if controller.is_some() {
                commands.entity(entity).remove::<CameraController>();
            } else {
                commands.entity(entity).insert(CameraController::default());
            }
        }
    }
}
