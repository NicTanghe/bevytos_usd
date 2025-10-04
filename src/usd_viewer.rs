// vim: set filetype=rust:
//! A simple 3D scene with light shining over a cube sitting on a plane.

use crate::usdish::meshdata_to_bevy;

use crate::open_rs_loader::{fetch_stage_usd, MeshInstance};

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    render::mesh::MeshTag,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

const USD_STAGE_PATH: &str = "C:/Users/Nicol/CGI/year5/slay/usd/helmet_bus_3.usdc";

pub fn usd_viewer() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(DirectionalLightShadowMap { size: 8192 })
        .add_systems(Startup, setup);
    app
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // import USD data without baking transforms into vertex data
    let scene = fetch_stage_usd(USD_STAGE_PATH);

    // cache Mesh handles so instances can reuse geometry
    let mesh_handles: Vec<Handle<Mesh>> = scene
        .meshes
        .iter()
        .map(|mesh| meshes.add(meshdata_to_bevy(mesh)))
        .collect();

    let material_handles: Vec<Handle<StandardMaterial>> = scene
        .meshes
        .iter()
        .map(|mesh| {
            let mut material = StandardMaterial::from(Color::srgb(0.7, 0.4, 1.0));
            material.double_sided = mesh.double_sided;

            // ✅ Ensure culling is disabled when double-sided
            if mesh.double_sided {
                material.cull_mode = None;
            }

            materials.add(material)
        })
        .collect();

    for instance in &scene.instances {
        if let (Some(mesh_handle), Some(material_handle)) = (
            mesh_handles.get(instance.mesh_index),
            material_handles.get(instance.mesh_index),
        ) {
            commands.spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(material_handle.clone()),
                MeshTag(instance.mesh_index as u32),
                instance_to_transform(instance),
            ));
        }
    }

    // light
    // directional sun

    use bevy::math::EulerRot;

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            shadow_depth_bias: 0.02, // tweak if you see acne
            shadow_normal_bias: 0.6,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 4,       // 4 cascades = sharper near shadows
            minimum_distance: 0.1, // start very close to the camera
            first_cascade_far_bound: 10.0,
            maximum_distance: 100.0, // shadows stop after 100 units
            overlap_proportion: 0.1, // overlap to reduce seams
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            std::f32::consts::PI,
            -std::f32::consts::FRAC_PI_4, // pitch = -45°
            0.0,
        )),
    ));

    // uniform ambient
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6405, 0.822, 1.0035),
        brightness: 200.0,
        affects_lightmapped_meshes: true,
    });

    // camera
    commands.spawn((
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));
}

fn instance_to_transform(instance: &MeshInstance) -> Transform {
    let mat = Mat4::from_cols_array(&[
        instance.transform[0][0],
        instance.transform[1][0],
        instance.transform[2][0],
        instance.transform[3][0],
        instance.transform[0][1],
        instance.transform[1][1],
        instance.transform[2][1],
        instance.transform[3][1],
        instance.transform[0][2],
        instance.transform[1][2],
        instance.transform[2][2],
        instance.transform[3][2],
        instance.transform[0][3],
        instance.transform[1][3],
        instance.transform[2][3],
        instance.transform[3][3],
    ]);

    Transform::from_matrix(mat)
}
