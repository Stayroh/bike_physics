use bevy::prelude::*;
use bevy::post_process::dof::DepthOfField;

/// Component that enables smooth auto-focus for depth of field
#[derive(Component)]
pub struct AutoFocus {
    /// Speed at which focus distance adjusts (higher = faster)
    pub adjust_speed: f32,
}

impl Default for AutoFocus {
    fn default() -> Self {
        Self {
            adjust_speed: 2.0,
        }
    }
}

/// Marker component for objects that can be focused on
#[derive(Component)]
pub struct FocusTarget;

/// System that smoothly adjusts depth of field focus distance
pub fn auto_focus_dof_system(
    time: Res<Time>,
    mut cameras: Query<(&GlobalTransform, &mut DepthOfField, &AutoFocus), With<Camera3d>>,
    targets: Query<&GlobalTransform, (Without<Camera3d>, With<FocusTarget>)>,
) {
    for (cam_transform, mut dof, auto_focus) in cameras.iter_mut() {
        let cam_pos = cam_transform.translation();
        let cam_forward = cam_transform.forward();
        
        let mut target_distance: f32 = 100.0;
        
        // Find closest object in front of camera
        for target_transform in targets.iter() {
            let to_target = target_transform.translation() - cam_pos;
            let distance = to_target.length();
            
            // Check if target is roughly in front of camera (dot product > 0)
            if to_target.normalize().dot(*cam_forward) > 0.5 {
                target_distance = target_distance.min(distance);
            }
        }
        
        // Smoothly interpolate focus distance
        let delta = time.delta_secs();
        dof.focal_distance = dof.focal_distance.lerp(
            target_distance,
            (auto_focus.adjust_speed * delta).min(1.0)
        );
    }
}

// Plugin to add the system
pub struct AutoFocusDofPlugin;

impl Plugin for AutoFocusDofPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, auto_focus_dof_system);
    }
}