use bevy::input::mouse::MouseMotion;
use bevy::input::ButtonInput;
use bevy::math::{EulerRot, Quat, Vec3};
use bevy::prelude::*;
use bevy::time::Time;
use bevy::window::{CursorGrabMode, CursorOptions};

#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}


impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

/// Key configuration
#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Grabs/ungrabs mouse cursor
/// fn cursor_grab(
//     ,
// )
pub fn toggle_grab_cursor(cursor_options: &mut CursorOptions) {
    match cursor_options.grab_mode {
        CursorGrabMode::None => {
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
        }
        _ => {
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        }
    }
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    toggle_grab_cursor(&mut cursor_options);
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        FlyCam,
        Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cursor_options: Single<&mut CursorOptions>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut query: Query<(&FlyCam, &mut Transform)>, //    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    for (_camera, mut transform) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);

        for key in keys.get_pressed() {
            match cursor_options.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    let key = *key;
                    if key == key_bindings.move_forward {
                        velocity += forward;
                    } else if key == key_bindings.move_backward {
                        velocity -= forward;
                    } else if key == key_bindings.move_left {
                        velocity -= right;
                    } else if key == key_bindings.move_right {
                        velocity += right;
                    } else if key == key_bindings.move_ascend {
                        velocity += Vec3::Y;
                    } else if key == key_bindings.move_descend {
                        velocity -= Vec3::Y;
                    }
                }
            }
        }

        velocity = velocity.normalize_or_zero();

        transform.translation += velocity * time.delta_secs() * settings.speed
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    mut state: MessageReader<MouseMotion>,
    cursor_options: Single<&mut CursorOptions>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    for mut transform in query.iter_mut() {
        for ev in state.read() {
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            match cursor_options.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = 1080f32;
                    pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                }
            }

            pitch = pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
}

fn cursor_grab(
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        toggle_grab_cursor(&mut cursor_options);
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementSettings>()
            .init_resource::<KeyBindings>()
            .add_systems(Startup, setup_player)
            .add_systems(Startup, initial_grab_cursor)
            .add_systems(Update, player_move)
            .add_systems(Update, player_look)
            .add_systems(
                Update,
                cursor_grab.run_if(not(bevy_egui::input::egui_wants_any_input)),
            );
    }
}
