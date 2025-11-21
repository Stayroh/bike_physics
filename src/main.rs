#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

mod autofocus;


use bevy::{
    core_pipeline::{
        bloom::Bloom,
        experimental::taa::TemporalAntiAliasing,
        post_process::ChromaticAberration,
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
        tonemapping::Tonemapping,
    },
    dev_tools::fps_overlay::FpsOverlayPlugin,
    pbr::{DefaultOpaqueRendererMethod, ScreenSpaceReflections},
    prelude::*,
    render::mesh::VertexAttributeValues,
    scene::SceneInstanceReady,
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
        .add_systems(Update, (ball_force_control, camera_target_system))
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
            intensity: 0.2,
            composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode::EnergyConserving,
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
        FollowTarget {
            strength: 2.0,
            vertical_offset: 0.2,
        },
        TemporalAntiAliasing::default(),
        ChromaticAberration {
            intensity: 0.02,
            ..default()
        },
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
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/Ball.glb"))),
        Transform::from_xyz(0.0, 1.0, 5.0).with_scale(Vec3::splat(0.05)),
        RigidBody::Dynamic,
        Collider::ball(1.0),
        Velocity::zero(),
        ExternalForce::default(),
        CameraTarget,
    ));

    commands
        .spawn((
            SceneRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/TronMap_Base.glb")),
            ),
            Transform::from_xyz(0.0, -1.0, 0.0),
        ))
        .observe(add_collider_on_scene_ready);

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

fn add_collider_on_scene_ready(
    scene_ready: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    mesh_query: Query<&Mesh3d>,
    meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    println!("Scene instance ready");
    for descendant in children.iter_descendants(scene_ready.target()) {
        println!("Checking descendant entity {:?}", descendant);
        if let Ok(Mesh3d(mesh_handle)) = mesh_query.get(descendant) {
            if let Some(mesh) = meshes.get(mesh_handle) {
                if let Some(collider) = mesh_to_trimesh_collider(mesh) {
                    commands.entity(descendant).insert(collider);
                    println!("Added collider to mesh entity {:?}", descendant);
                }
            }
        }
    }
}

pub fn mesh_to_trimesh_collider(mesh: &Mesh) -> Option<Collider> {
    // Extract vertices
    let vertices = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(positions)) => positions
            .iter()
            .map(|[x, y, z]| Vect::new(*x, *y, *z))
            .collect::<Vec<_>>(),
        _ => {
            return {
                warn!("Mesh does not have valid position attribute for collider generation");
                None
            };
        }
    };

    // Extract indices
    let indices = match mesh.indices() {
        Some(bevy::render::mesh::Indices::U32(idx)) => idx
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect::<Vec<_>>(),
        Some(bevy::render::mesh::Indices::U16(idx)) => idx
            .chunks_exact(3)
            .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
            .collect::<Vec<_>>(),
        None => {
            return {
                warn!("Mesh does not have indices for collider generation");
                None
            };
        }
    };

    match Collider::trimesh(vertices, indices) {
        Ok(collider) => Some(collider),
        Err(e) => {
            warn!("Failed to create trimesh collider: {}", e);
            None
        }
    }
}

fn ball_force_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut ExternalForce, &Velocity), With<SceneRoot>>
) {
    for (mut externalforce, velocity) in query.iter_mut() {
        let look_vector = velocity.linvel.normalize();
        let speed = velocity.linvel.length() * 1.0;
        let up_vector = Vec3::new(0.0, 1.0, 0.0);
        let right_vector = look_vector.cross(up_vector).normalize();
        let mut force = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            force += look_vector;
            println!("Key W is pressed");
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            force -= look_vector;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            force -= right_vector * speed;
            println!("Key A is pressed");

        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            force += right_vector * speed;
        }
        if force != Vec3::ZERO {
            force = force * 0.001;
            externalforce.force = force;
        } else {
            externalforce.force = Vec3::ZERO;
        }
    }
}

#[derive(Component)]
struct FollowTarget {
    strength: f32,
    vertical_offset: f32,
}

#[derive(Component)]
struct CameraTarget;

fn camera_target_system(
    camera_query: Query<(&mut Transform, &FollowTarget), With<Camera3d>>,
    target_query: Single<&Transform, (With<CameraTarget>, Without<Camera3d>)>,
    time: Res<Time>
) {
    for (mut transform, follow_target) in camera_query.into_iter() {
        let target_position = target_query.translation + Vec3::Y * follow_target.vertical_offset;
        let current_position = transform.translation;
        let t = 1.0 - bevy::prelude::ops::powf(2.0, -time.delta_secs() * follow_target.strength);
        let new_position = target_position * t + (1.0 - t) * current_position;
        *transform = Transform::from_translation(new_position).looking_at(target_query.translation, Vec3::Y);

    }
}
