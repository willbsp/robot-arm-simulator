use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use std::f32::consts::PI;
use std::f32::consts::TAU;

#[derive(Component)]
struct RotationalJoint {
    speed: f32,
    pivot: Vec3,
    rotation_axis: Vec3,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, process_rotations)
        .run();
}

// TODO implement proportional control for rotating joints
// TODO then process_rotations can become process_joint_movements

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
                speed: 1.0,
                pivot: Vec3::new(0.0, 0.0, 0.0),
                rotation_axis: Vec3::Y,
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Cuboid::new(0.3, 2.0, 0.3)),
                        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
                        transform: Transform::from_xyz(0.0, 2.0, 0.0),
                        ..default()
                    },
                    RotationalJoint {
                        speed: 2.0,
                        pivot: Vec3::new(0.0, 1.0, 0.0),
                        rotation_axis: Vec3::X,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(0.3, 2.0, 0.3)),
                            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
                            transform: Transform::from_xyz(0.0, 2.0, 0.0),
                            ..default()
                        },
                        RotationalJoint {
                            speed: 3.0,
                            pivot: Vec3::new(0.0, 1.0, 0.0),
                            rotation_axis: Vec3::Z,
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

fn process_rotations(mut cubes: Query<(&mut Transform, &RotationalJoint)>, timer: Res<Time>) {
    for (mut transform, cube) in &mut cubes {
        transform.rotate_around(
            cube.pivot,
            get_joint_rotation(cube.rotation_axis, timer.delta_seconds() * cube.speed)        );
    }
}

fn get_joint_rotation(rotation_axis: Vec3, multiplier: f32) -> Quat {
    return Quat::from_rotation_x(rotation_axis.x * multiplier)
        * Quat::from_rotation_y(rotation_axis.y * multiplier)
        * Quat::from_rotation_z(rotation_axis.z * multiplier);
}
