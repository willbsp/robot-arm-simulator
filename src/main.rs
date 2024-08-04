use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use std::f32::consts::PI;
use std::f32::consts::TAU;

#[derive(Component)]
struct RotationalJoint {
    p_gain: f32,
    i_gain: f32,
    d_gain: f32,
    p_prior: Vec3,
    i_prior: Vec3,
    pivot: Vec3,
    rotation_axis: Vec3,
    target_angle: f32,
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct DebugText;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_input,
                text_update_system,
                text_debug_update_system,
                pid_controller,
            ),
        )
        .run();
}

// TODO implement proportional control for rotating joints
// TODO then process_rotations can become process_joint_movements

const P_GAIN: f32 = 120.0;
const I_GAIN: f32 = 80.0;
const D_GAIN: f32 = 0.5;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    ..default()
                },
            ),
            TextSection::from_style(if cfg!(feature = "default_font") {
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    // If no font is specified, the default font (a minimal subset of FiraMono) will be used.
                    ..default()
                }
            } else {
                // "default_font" feature is unavailable, load a font to use instead.
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                }
            }),
        ]),
        FpsText,
    ));

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "hello bevy!",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        DebugText,
    ));

    // cube
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.3, 2.0, 0.3)),
                material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            },
            RotationalJoint {
                p_gain: P_GAIN,
                i_gain: I_GAIN,
                d_gain: D_GAIN,
                p_prior: Vec3::ZERO,
                i_prior: Vec3::ZERO,
                pivot: Vec3::new(0.0, 0.0, 0.0),
                rotation_axis: Vec3::Y,
                target_angle: 0.0,
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Cuboid::new(0.3, 2.0, 0.3)),
                        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
                        transform: Transform::from_xyz(0.3, 2.0, 0.0),
                        ..default()
                    },
                    RotationalJoint {
                        p_gain: P_GAIN,
                        i_gain: I_GAIN,
                        d_gain: D_GAIN,
                        p_prior: Vec3::ZERO,
                        i_prior: Vec3::ZERO,
                        pivot: Vec3::new(0.0, 1.0, 0.0),
                        rotation_axis: Vec3::X,
                        target_angle: 0.0,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(0.3, 2.0, 0.3)),
                            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
                            transform: Transform::from_xyz(0.0, 2.0, 0.3),
                            ..default()
                        },
                        RotationalJoint {
                            p_gain: P_GAIN,
                            i_gain: I_GAIN,
                            d_gain: D_GAIN,
                            p_prior: Vec3::ZERO,
                            i_prior: Vec3::ZERO,
                            pivot: Vec3::new(0.0, 1.0, 0.0),
                            rotation_axis: Vec3::Z,
                            target_angle: 0.0,
                        },
                    ));
                });
        });

    // camera
    commands.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            // 6 world units per window height.
            scaling_mode: ScalingMode::FixedVertical(6.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        ..default()
    });

    // plane
    /*commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..default()
    });*/

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(3.0, 6.0, 3.0),
        ..default()
    });
}

fn process_input(
    mut keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut RotationalJoint)>,
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for (mut arm) in &mut query {
            arm.target_angle = (arm.target_angle + 1.0).clamp(0.0, 360.0);
        }
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        for (mut arm) in &mut query {
            arm.target_angle = (arm.target_angle - 1.0).clamp(0.0, 360.0);
        }
    } else {
    }
}

fn process_rotations(mut cubes: Query<(&mut Transform, &RotationalJoint)>, timer: Res<Time>) {
    for (mut transform, cube) in &mut cubes {
        transform.rotate_around(
            cube.pivot,
            get_joint_rotation(cube.rotation_axis, timer.delta_seconds() * cube.p_gain),
        );
    }
}

fn pid_controller(mut query: Query<(&mut Transform, &mut RotationalJoint)>, timer: Res<Time>) {
    for (mut transform, mut joint) in &mut query {
        let current_rotation = transform.rotation;
        let angle_rads = degrees_to_rads(joint.target_angle);
        let target_rotation = get_joint_rotation(joint.rotation_axis, angle_rads);

        let p = calculate_error_quaternion(current_rotation, target_rotation).xyz();
        let i = joint.i_prior + p * timer.delta_seconds();
        let d = (p - joint.p_prior) / timer.delta_seconds();
        let mut out = (p * joint.p_gain) + (i * joint.i_gain);
        if !d.is_nan() {
            out += (d * joint.d_gain)
        }

        println!("\np: {p} \n i: {i} \n d: {d} \n out: {out}");
        let final_quat = get_joint_rotation(out, timer.delta_seconds());
        transform.rotate_around(joint.pivot, final_quat);
        joint.p_prior = p;
        joint.i_prior = i;
        /*println!("current_rotation {0}", current_rotation);
        println!("angle_rads {0}", angle_rads);
        println!("target_rotation {0}", target_rotation);
        println!("error_quat {0}", error_quat);
        println!("final_quat {0}", final_quat);
        println!("");*/
    }
}

fn degrees_to_rads(angle: f32) -> f32 {
    (angle * (PI / 180.0))
}

fn calculate_error_quaternion(q1: Quat, q2: Quat) -> Quat {
    let q1_inverse = q1.conjugate();
    (q1_inverse * q2).normalize()
}

fn get_joint_rotation(rotation_axis: Vec3, multiplier: f32) -> Quat {
    return Quat::from_rotation_x(rotation_axis.x * multiplier)
        * Quat::from_rotation_y(rotation_axis.y * multiplier)
        * Quat::from_rotation_z(rotation_axis.z * multiplier);
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn text_debug_update_system(
    mut text_query: Query<&mut Text, With<DebugText>>,
    joints: Query<(&RotationalJoint)>,
) {
    let mut joint_angles: Vec<f32> = Vec::new();
    for joint in &joints {
        joint_angles.push(joint.target_angle);
    }
    for mut debug_text in &mut text_query {
        let mut text = String::from("Joints ");
        for (i, joint_angle) in joint_angles.iter().enumerate() {
            text.push_str(format!("[{i}]: {joint_angle} ").as_str());
        }
        debug_text.sections[0].value = text;
    }
}
