#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

mod autofocus;


use avian3d::prelude::*;
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    core_pipeline::{
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
        tonemapping::Tonemapping,
    },
    dev_tools::fps_overlay::FpsOverlayPlugin,
    pbr::{DefaultOpaqueRendererMethod, ScreenSpaceReflections},
    post_process::{bloom::Bloom, effect_stack::ChromaticAberration},
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
        .add_plugins(PhysicsPlugins::default())
        //.add_plugins(PhysicsDebugPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_freecam)
        .add_systems(Update, (ball_system, camera_target_system))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(-10.0, 5.0, 25.0) * 0.4)
            .looking_at(Vec3::new(1.2, -2.0, 0.0), Vec3::Y),
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.2,
            composite_mode: bevy::post_process::bloom::BloomCompositeMode::EnergyConserving,
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
        Transform::from_xyz(0.0, 0.0, 5.0).with_scale(Vec3::splat(0.05)),
        
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
        Transform::from_xyz(0.0, 1.0, 5.0),
        autofocus::FocusTarget,
        RigidBody::Dynamic,
        //Collider::sphere(1.0),
        LinearVelocity::ZERO,
        ConstantForce::default(),
        CameraTarget,
        Collider::sphere(0.05),
        Restitution {
            coefficient: 0.2,
            combine_rule: CoefficientCombine::Multiply,
        },
        Ball {
            radius: 0.1,
            stiffness: 0.3,
            damping: 0.005,
        },
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
    scene_ready: On<SceneInstanceReady>,
    children: Query<&Children>,
    mesh_query: Query<&Mesh3d>,
    meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    println!("Scene instance ready");
    for descendant in children.iter_descendants(scene_ready.entity) {
        println!("Checking descendant entity {:?}", descendant);
        if let Ok(Mesh3d(mesh_handle)) = mesh_query.get(descendant) {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let collider = Collider::trimesh_from_mesh(mesh).unwrap();
                commands.entity(descendant).insert(collider);
                commands.entity(descendant).insert(RigidBody::Static);
                commands.entity(descendant).insert(CollisionMargin(0.001));
                commands.entity(descendant).insert(Restitution {
                    coefficient: 1.0,
                    combine_rule: CoefficientCombine::Multiply,
                });
            }
        }
    }
}

fn ball_system(
    spatial_query: SpatialQuery,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut ConstantForce, &LinearVelocity, &Ball, &mut AngularVelocity), With<SceneRoot>>,
) {
    for (mut transform, mut constant_force, velocity, ball, mut angular_velocity) in query.iter_mut() {
        // Input handling
        
        let (input_force, look_vector) = {
            let look_vector = velocity.0.normalize();
            let speed = velocity.0.length() * 1.0;
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
            (force * 0.001, look_vector)
        };
        
        let mut total_force = input_force;
        // Spring force towards ground

        let origin = transform.translation;
        let maybe_result = spatial_query.project_point(origin, false, &SpatialQueryFilter::default());
        if let Some(result) = maybe_result {
            let delta = result.point - origin;
            let distance = delta.length();
            println!("Distance to ground: {}", distance);
            let compression = (ball.radius - distance).max(0.0);
            let spring_force = ball.stiffness * compression;

            total_force += -delta.normalize() * spring_force;
        }

        let damp_force = {
            let vertical_velocity = velocity.0.dot(Vec3::Y);
            -ball.damping * vertical_velocity
        };
        total_force += Vec3::Y * damp_force;
        println!("Total force applied to ball: {:?}", total_force);
        constant_force.0 = total_force;
        angular_velocity.0 = Vec3::ZERO;
        transform.look_to(look_vector, Vec3::Y);
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
    time: Res<Time>,
) {
    for (mut transform, follow_target) in camera_query.into_iter() {
        let target_position = target_query.translation + Vec3::Y * follow_target.vertical_offset;
        let current_position = transform.translation;
        let t = 1.0 - bevy::prelude::ops::powf(2.0, -time.delta_secs() * follow_target.strength);
        let new_position = target_position * t + (1.0 - t) * current_position;
        *transform =
            Transform::from_translation(new_position).looking_at(target_query.translation, Vec3::Y);
    }
}

#[derive(Component)]
struct Ball {
    radius: f32,
    stiffness: f32,
    damping: f32,
}
