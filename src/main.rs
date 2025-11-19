use bevy::{
    core_pipeline::{tonemapping::Tonemapping, prepass::{DepthPrepass, MotionVectorPrepass, DeferredPrepass}}, dev_tools::fps_overlay::FpsOverlayPlugin,
    pbr::DefaultOpaqueRendererMethod, pbr::ScreenSpaceReflections, post_process::bloom::Bloom,
    anti_alias::fxaa::Fxaa,
    post_process::{effect_stack::ChromaticAberration, dof::DepthOfField},
    prelude::*,
};

fn main() {
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins(DefaultPlugins)
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
            ..default()
        },
        DepthPrepass,
        MotionVectorPrepass,
        DeferredPrepass,
        Msaa::Off,
        Fxaa::default(),
        ChromaticAberration::default(),
        DepthOfField {
            focal_distance: 8.0,
            aperture_f_stops : 1.0,
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
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronMap.glb"))),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));
}
