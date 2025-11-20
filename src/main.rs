#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

mod autofocus;

use bevy::{
    core_pipeline::{tonemapping::Tonemapping, prepass::{DepthPrepass, MotionVectorPrepass, DeferredPrepass}, Skybox}, dev_tools::fps_overlay::FpsOverlayPlugin,
    pbr::DefaultOpaqueRendererMethod, pbr::ScreenSpaceReflections, post_process::bloom::Bloom,
    anti_alias::taa::TemporalAntiAliasing,
    post_process::{effect_stack::ChromaticAberration, dof::DepthOfField},
    prelude::*,
};
use camera_controller::{CameraController, CameraControllerPlugin};


fn main() {
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraControllerPlugin)
        .add_plugins(autofocus::AutoFocusDofPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(-10.0, 5.0, 25.0) * 0.4).looking_at(Vec3::new(1.2, -2.0, 0.0), Vec3::Y),
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
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
        (
        ChromaticAberration {
            intensity: 0.01,
            ..default()
        },
        DepthOfField {
            aperture_f_stops: 5.0,
            ..default()
        },
        ),
        CameraController {
            walk_speed: 3.0,
            run_speed: 10.0,
            ..Default::default()
        },
        autofocus::AutoFocus::default(),
        Skybox {
            image: asset_server.load("textures/skyboxes/puresky.ktx2"),
            brightness: 30.0,
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("textures/skyboxes/puresky.ktx2"),
            specular_map: asset_server.load("textures/skyboxes/puresky.ktx2"),
            intensity: 30.0,
            ..default()
        },
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
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronMap_Base.glb"))),
        Transform::from_xyz(0.0, -1.0, 0.0),
        autofocus::FocusTarget,

    ));
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronMap_Neon.glb"))),
        Transform::from_xyz(0.0, -1.0, 0.0),
        autofocus::FocusTarget,

    ));

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronBike.glb"))),
        Transform::from_xyz(0.0, -1.0, 0.0),
        autofocus::FocusTarget,
    ));
}
