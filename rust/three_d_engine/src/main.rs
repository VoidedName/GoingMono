use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::f32::consts::PI;
use std::{fs, ops};
use std::process::Output;
use vn_engine::color::{BLACK, RGBA, VIOLET, WHITE};
use vn_engine::engine::{VNERunner, VNEngine, VNEngineState};
use vn_engine::opengl::OpenGLRenderer;
use vn_engine::render::{PixelPosition, VNERenderer};

#[derive(Debug, Copy, Clone)]
struct Vec3D {
    x: f32,
    y: f32,
    z: f32,
}

macro_rules! vec3d {
    [$x:expr, $y:expr, $z:expr] => {
        Vec3D { x: $x, y: $y, z: $z }
    }
}

impl Vec3D {
    pub fn cross_product(&self, other: &Vec3D) -> Vec3D {
        Vec3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn dot_product(&self, other: &Vec3D) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn normalize(&mut self) {
        let l = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x /= l;
        self.y /= l;
        self.z /= l;
    }
}

impl ops::Sub for Vec3D {
    type Output = Vec3D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Add for Vec3D {
    type Output = Vec3D;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Triangle {
    p: [Vec3D; 3],
}

#[derive(Debug, Clone)]
struct Mesh {
    tris: Vec<Triangle>,
}

impl Mesh {
    fn load_from_file(filename: &str) -> Result<Mesh, String> {
        let content = fs::read_to_string(filename);
        if let Err(e) = content { return Err(e.to_string()); }

        let content = content.unwrap();
        let mut lines = content.lines();
        let mut vertices: Vec<Vec3D> = Vec::new();
        let mut tris: Vec<Triangle> = Vec::new();

        for line in lines {
            if line.starts_with("v") {
                let parts: Vec<&str> = line[2..].split(" ").collect();
                let x: f32 = parts[0].parse().unwrap();
                let y: f32 = parts[1].parse().unwrap();
                let z: f32 = parts[2].parse().unwrap();
                vertices.push(vec3d![x, y, z]);
            }

            if line.starts_with("f") {
                let parts: Vec<&str> = line[2..].split(" ").collect();
                let x: usize = parts[0].parse().unwrap();
                let y: usize = parts[1].parse().unwrap();
                let z: usize = parts[2].parse().unwrap();
                tris.push(Triangle { p: [vertices[x - 1], vertices[z - 1], vertices[y - 1]] });
            }
        }

        Ok(Mesh {
            tris
        })
    }
}

#[derive(Debug, Copy, Clone)]
struct Matrix4x4 {
    // [row][col]
    m: [[f32; 4]; 4],
}

struct Rasterizer {
    width: u32,
    height: u32,
    rng: ThreadRng,
    time_since_last_fps_draw: f64,
    frames_since_last_fps_draw: u128,

    elapsed: f32,

    mesh_cube: Mesh,
    suzanne: Mesh,
    projection: Matrix4x4,
    camera: Vec3D,
}

impl Rasterizer {
    pub fn new(width: u32, height: u32) -> Rasterizer {
        let aspect_ration = height as f32 / width as f32;
        let field_of_view = PI / 2.0;
        let fov_scale = 1.0 / (field_of_view / 2.0).tan();
        let z_far = 1000.0;
        let z_near = 0.1;

        Rasterizer {
            suzanne: Mesh::load_from_file("./Suzanne.obj").unwrap(),
            width,
            height,
            elapsed: 0.0,
            rng: rand::thread_rng(),
            time_since_last_fps_draw: 0.0,
            frames_since_last_fps_draw: 0,
            #[rustfmt::skip]
            projection: Matrix4x4 {
                m: [[
                    aspect_ration * fov_scale, 0.0, 0.0, 0.0,
                ], [
                    0.0, fov_scale, 0.0, 0.0,
                ], [
                    0.0, 0.0, z_far / (z_far - z_near), 1.0,
                ], [
                    0.0, 0.0, (-z_far * z_near) / (z_far - z_near), 0.0,
                ]]
            },
            #[rustfmt::skip]
            mesh_cube: Mesh {
                tris: vec![
                    // SOUTH
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 0.0 }, Vec3D { x: 0.0, y: 1.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 0.0 }] },
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 0.0 }, Vec3D { x: 1.0, y: 0.0, z: 0.0 }] },

                    // EAST
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 }] },
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 }, Vec3D { x: 1.0, y: 0.0, z: 1.0 }] },

                    // NORTH
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 1.0 }] },
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 1.0 }, Vec3D { x: 0.0, y: 0.0, z: 1.0 }] },

                    // WEST
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 0.0 }] },
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 0.0 }, Vec3D { x: 0.0, y: 0.0, z: 0.0 }] },

                    // TOP
                    Triangle { p: [Vec3D { x: 0.0, y: 1.0, z: 0.0 }, Vec3D { x: 0.0, y: 1.0, z: 1.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 }] },
                    Triangle { p: [Vec3D { x: 0.0, y: 1.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 }, Vec3D { x: 1.0, y: 1.0, z: 0.0 }] },

                    // BOTTOM
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 0.0, z: 0.0 }] },
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 0.0, z: 0.0 }, Vec3D { x: 1.0, y: 0.0, z: 0.0 }] },
                ]
            },
            camera: Vec3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    fn update_fps(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        self.time_since_last_fps_draw += engine.delta;
        self.frames_since_last_fps_draw += 1;
        if self.time_since_last_fps_draw > 0.2 {
            let fps = (1.0 / self.time_since_last_fps_draw) * self.frames_since_last_fps_draw as f64;
            self.time_since_last_fps_draw = 0.0;
            self.frames_since_last_fps_draw = 0;
            renderer.set_title(format!("Test - FPS: {}", fps).as_str());
        }
    }

    fn multiply_vector_matrix(vec: &Vec3D, mat: &Matrix4x4) -> Vec3D {
        let mut x =
            (vec.x * mat.m[0][0]) + (vec.y * mat.m[1][0]) + (vec.z * mat.m[2][0]) + mat.m[3][0];
        let mut y =
            (vec.x * mat.m[0][1]) + (vec.y * mat.m[1][1]) + (vec.z * mat.m[2][1]) + mat.m[3][1];
        let mut z =
            (vec.x * mat.m[0][2]) + (vec.y * mat.m[1][2]) + (vec.z * mat.m[2][2]) + mat.m[3][2];
        let w = (vec.x * mat.m[0][3]) + (vec.y * mat.m[1][3]) + (vec.z * mat.m[2][3]) + mat.m[3][3];

        if w != 0.0 {
            x /= w;
            y /= w;
            z /= w;
        }

        Vec3D { x, y, z }
    }
}

struct TriangleToDraw {
    v1: Vec3D,
    v2: Vec3D,
    v3: Vec3D,
    color: RGBA,
}

impl VNERunner for Rasterizer {
    fn tick(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        self.update_fps(engine, renderer);

        self.elapsed += engine.delta as f32;

        renderer.clear_screen(BLACK);

        let theta = 1.0 * self.elapsed;

        #[rustfmt::skip]
            let rot_z = Matrix4x4 {
            m: [[
                theta.cos(), theta.sin(), 0.0, 0.0,
            ], [
                -theta.sin(), theta.cos(), 0.0, 0.0,
            ], [
                0.0, 0.0, 1.0, 0.0,
            ], [
                0.0, 0.0, 0.0, 1.0,
            ]]
        };

        #[rustfmt::skip]
            let rot_x = Matrix4x4 {
            m: [[
                1.0, 0.0, 0.0, 0.0,
            ], [
                0.0, (theta * 0.5).cos(), (theta * 0.5).sin(), 0.0,
            ], [
                0.0, -(theta * 0.5).sin(), (theta * 0.5).cos(), 0.0,
            ], [
                0.0, 0.0, 0.0, 1.0,
            ]]
        };

        let mut to_draw: Vec<TriangleToDraw> = Vec::new();

        // Draw Triangles
        // screen space normalized left/right x=-1 / 1; bottom/top y=-1 / 1; front/back z=-1 / 1
        for tri in self.suzanne.tris.iter() {
            let mut tri_rotated_z = tri.clone();
            tri_rotated_z.p[0] = Rasterizer::multiply_vector_matrix(&tri.p[0], &rot_z);
            tri_rotated_z.p[1] = Rasterizer::multiply_vector_matrix(&tri.p[1], &rot_z);
            tri_rotated_z.p[2] = Rasterizer::multiply_vector_matrix(&tri.p[2], &rot_z);

            let mut tri_rotated_x = tri.clone();
            tri_rotated_x.p[0] = Rasterizer::multiply_vector_matrix(&tri_rotated_z.p[0], &rot_x);
            tri_rotated_x.p[1] = Rasterizer::multiply_vector_matrix(&tri_rotated_z.p[1], &rot_x);
            tri_rotated_x.p[2] = Rasterizer::multiply_vector_matrix(&tri_rotated_z.p[2], &rot_x);

            let mut tri_translated = tri_rotated_x.clone();
            tri_translated.p[0].z += 3.0;
            tri_translated.p[1].z += 3.0;
            tri_translated.p[2].z += 3.0;

            let line1 = tri_translated.p[1] - tri_translated.p[0];
            let line2 = tri_translated.p[2] - tri_translated.p[0];
            let mut normal = line1.cross_product(&line2);
            normal.normalize();

            let similarity = normal.dot_product(&(tri_translated.p[0] - self.camera));

            if similarity < 0.0 {
                let mut v1 =
                    Rasterizer::multiply_vector_matrix(&tri_translated.p[0], &self.projection);
                let mut v2 =
                    Rasterizer::multiply_vector_matrix(&tri_translated.p[1], &self.projection);
                let mut v3 =
                    Rasterizer::multiply_vector_matrix(&tri_translated.p[2], &self.projection);

                let mut light_direction = vec3d![0.25, 0.25, -1.0];
                light_direction.normalize();

                let lum = normal.dot_product(&light_direction);
                let color = RGBA { r: lum, g: lum, b: lum, a: 1.0 };

                v1.x += 1.0;
                v1.y += 1.0;

                v2.x += 1.0;
                v2.y += 1.0;

                v3.x += 1.0;
                v3.y += 1.0;

                v1.x *= 0.5 * self.width as f32;
                v1.y *= 0.5 * self.height as f32;

                v2.x *= 0.5 * self.width as f32;
                v2.y *= 0.5 * self.height as f32;

                v3.x *= 0.5 * self.width as f32;
                v3.y *= 0.5 * self.height as f32;

                to_draw.push(TriangleToDraw {
                    v1,
                    v2,
                    v3,
                    color,
                })
            }
        }

        to_draw.sort_by(|a, b| {
            let z1 = (a.v1.z + a.v2.z + a.v3.z / 3.0);
            let z2 = (b.v1.z + b.v2.z + b.v3.z / 3.0);

            z1.partial_cmp(&z2).unwrap()
        });

        for tris in to_draw.iter() {
            renderer.fill_triangle(
                PixelPosition {
                    x: tris.v1.x.round() as u32,
                    y: tris.v1.y.round() as u32,
                },
                PixelPosition {
                    x: tris.v2.x.round() as u32,
                    y: tris.v2.y.round() as u32,
                },
                PixelPosition {
                    x: tris.v3.x.round() as u32,
                    y: tris.v3.y.round() as u32,
                },
                PixelPosition {
                    x: 0,
                    y: 0,
                },
                PixelPosition {
                    x: self.width,
                    y: self.height,
                },
                tris.color
            );
        }
    }
}

fn main() {
    let (width, height) = (256, 240);
    let mut engine = VNEngine::new_sprite_based(width, height, 8);
    let mut runner = Rasterizer::new(width, height);
    engine.run(&mut runner)
}
