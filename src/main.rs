use bevy::math::VectorSpace;
use bevy::render::mesh::Indices;
use bevy::{prelude::*, window::WindowResolution};
use bevy_pixel_buffer::prelude::*;

const WINDOW_WIDTH: f32 = 500.;
const WINDOW_HEIGHT: f32 = 500.;

fn main() {
    let size = PixelBufferSize {
        size: UVec2::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
        pixel_size: UVec2::new(1, 1),
    };

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                    ..default()
                }),
                ..default()
            }),
            PixelBufferPlugin,
        ))
        .add_systems(Startup, pixel_buffer_setup(size))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let torus = meshes.add(Torus::default());
    let mut transform = Transform::from_xyz(0.0, 0.0, 5.0);
    transform.rotate(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI * 0.5));
    commands.spawn(PbrBundle {
        mesh: torus.clone(),
        material: materials.add(StandardMaterial {
            base_color: Srgba::hex("#ffd891").unwrap().into(),
            unlit: true,
            ..default()
        }),
        transform,
        ..default()
    });

    if let Some(mesh) = meshes.get(&torus) {
        let triangles = gen_triangles(mesh);
        commands.spawn(TriangleComponent { triangles });
    }

    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3 { x: 0.0, y: 0.0, z: 10.0 }, Vec3::Y),
        ..default()
    }, RaycasterCamera));
}

fn update(mut pb: QueryPixelBuffer, query: Query<&TriangleComponent>,
          camera_query: Query<&Transform, With<RaycasterCamera>>) {
    // query: Query<&Handle<Mesh>>,
    // meshes: Res<Assets<Mesh>>) {
    // let handle = query.get_single().unwrap();
    let triangle_component = query.single();
    let camera = camera_query.single();
    pb.frame()
        .per_pixel(|x, _| update_pixel(x, triangle_component, camera));
}

fn update_pixel(x: UVec2, component: &TriangleComponent, camera_transform: &Transform) -> Pixel {
    // define ray through screen space position
    // loop over each triangle
    // calculate ray intersection and find closest triangle
    // gather sum of scattered light

    let ray = Ray {
        origin: camera_transform.translation,
        direction: camera_transform.rotation * Vec3::Z,
    };

    let firstTriangle = component.triangles.first().unwrap();
    if let Some(point) = firstTriangle.intersects_ray(&ray) {
        // println!("intersects at {}", point);
        Pixel {
            r: 255,
            g: 255,
            b: 255,
            a: 254,
        }
    } else {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 254,
        }
    }


    // Pixel {
    //     r: ((x.x as f32) / WINDOW_WIDTH * 256.0) as u8,
    //     g: ((x.x as f32) / WINDOW_WIDTH * 256.0) as u8,
    //     b: ((x.x as f32) / WINDOW_WIDTH * 256.0) as u8,
    //     a: 254,
    // }
}

#[derive(Component)]
struct RaycasterCamera;

struct Triangle {
    vertices: Vec<Vec3>,
    normal: Vec3,
}

impl Triangle {
    fn intersects_ray(&self, ray: &Ray) -> Option<Vec3> {
        // is parallel?
        let normal_ray_dot = self.normal.dot(ray.direction);
        if normal_ray_dot.abs() < 0.01 {
            return None;
        }

        // is it behind the origin
        let d = -self.normal.dot(self.vertices[0]);
        let t = -(self.normal.dot(ray.origin) + d) / normal_ray_dot;
        if t < 0. {
            return None;
        }

        // plane intersection point
        let P = ray.origin + t * ray.direction;

        // test behind each edge
        let edge0 = self.vertices[1] - self.vertices[0];
        let vp0 = P - self.vertices[0];
        if self.normal.dot(edge0.cross(vp0)) < 0. {
            return None;
        }

        let edge1 = self.vertices[2] - self.vertices[1];
        let vp1 = P - self.vertices[1];
        if self.normal.dot(edge1.cross(vp1)) < 0. {
            return None;
        }

        let edge2 = self.vertices[0] - self.vertices[2];
        let vp2 = P - self.vertices[2];
        if self.normal.dot(edge2.cross(vp2)) < 0. {
            return None;
        }

        Some(P)
    }
}

#[derive(Component)]
struct TriangleComponent {
    triangles: Vec<Triangle>,
}

struct Ray {
    direction: Vec3,
    origin: Vec3,
}

fn gen_triangles(mesh: &Mesh) -> Vec<Triangle> {
    if let Some(indices) = mesh.indices() {
        if let Some(vertex_attribute) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            let vertex_positions = vertex_attribute.as_float3().unwrap();
            return match indices {
                Indices::U32(indices) => indices
                    .chunks(3)
                    .map(|c| {
                        let i0 = c[0] as usize;
                        let i1 = c[1] as usize;
                        let i2 = c[2] as usize;

                        let v0 = Vec3::from_array(vertex_positions[i0]);
                        let v1 = Vec3::from_array(vertex_positions[i1]);
                        let v2 = Vec3::from_array(vertex_positions[i2]);

                        let normal = Vec3::new(
                            v0.y * v1.z - v0.z * v1.y,
                            v0.z * v1.x - v0.x * v1.z,
                            v0.x * v1.y - v0.y * v1.x,
                        )
                            .normalize();

                        Triangle {
                            vertices: vec![v0, v1, v2],
                            normal,
                        }
                    })
                    .collect(),
                _ => vec![],
            };
        }
    }
    vec![]
}

