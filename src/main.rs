#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

mod autofocus;

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    core_pipeline::{
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
        tonemapping::Tonemapping,
    },
    dev_tools::fps_overlay::FpsOverlayPlugin,
    pbr::DefaultOpaqueRendererMethod,
    pbr::ScreenSpaceReflections,
    post_process::bloom::Bloom,
    post_process::{dof::DepthOfField, effect_stack::ChromaticAberration},
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
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_freecam)
        .add_systems(
            Update,
            add_colliders_to_scene.run_if(resource_exists::<Assets<Mesh>>),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(-10.0, 5.0, 25.0) * 0.4)
            .looking_at(Vec3::new(1.2, -2.0, 0.0), Vec3::Y),
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
        NeedsCollider,
    ));

    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronMap_Neon.glb")),
        ),
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

fn add_colliders_to_scene(
    mut commands: Commands,
    scene_query: Query<(Entity, &Children), With<NeedsCollider>>,
    mesh_query: Query<&Handle<>>,
    collider_query: Query<&Collider>,
    meshes: Res<Assets<Mesh>>,
) {
    for (scene_entity, children) in scene_query.iter() {
        let mut all_meshes_loaded = true;
        let mut colliders_added = 0;
        
        // Recursively check all children for meshes
        for &child in children.iter() {
            if let Ok(mesh_handle) = mesh_query.get(child) {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    // Only add collider if it doesn't already have one
                    if collider_query.get(child).is_err() {
                        if let Ok(collider) = Collider::from_bevy_mesh(
                            mesh, 
                            &ComputedColliderShape::TriMesh
                        ) {
                            commands.entity(child).insert((
                                RigidBody::Fixed,
                                collider,
                            ));
                            colliders_added += 1;
                        }
                    }
                } else {
                    all_meshes_loaded = false;
                }
            }
        }
        
        // Remove marker once all meshes are processed
        if all_meshes_loaded && colliders_added > 0 {
            commands.entity(scene_entity).remove::<NeedsCollider>();
            println!("Added {} colliders to scene", colliders_added);
        }
    }
}

#[derive(Component)]
struct NeedsCollider;