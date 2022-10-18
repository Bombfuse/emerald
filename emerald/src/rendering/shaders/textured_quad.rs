use rapier2d::na::{Matrix4, Quaternion, Vector2, Vector4};

// lib.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub(crate) const VERTICES: &[Vertex] = &[
    // Changed
    Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // A
    Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    }, // B
    Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0],
    }, // C
    Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
];

pub(crate) const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub(crate) struct Instance {
    /// The (x, y) coordinates of the bottom left corner where the instance should be drawn.
    pub position: Vector2<f32>,

    /// The target area of the texture to draw.
    pub target: Vector4<f32>,

    /// The rotation applied to the drawn texture.
    pub rotation: Quaternion<f32>,
}
impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let position = Matrix4::from_columns(&[
            Vector4::new(self.position.x, 0.0, 0.0, 0.0),
            Vector4::new(0.0, self.position.y, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ]);

        InstanceRaw {
            model: (position * Matrix4::identity()).into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct InstanceRaw {
    model: [[f32; 4]; 4],
}
impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub(crate) struct Camera2D {
    pub view_height: u32,
    pub view_width: u32,
}

impl Camera2D {
    pub fn new(view_width: u32, view_height: u32) -> Self {
        Self {
            view_height,
            view_width,
        }
    }
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        // // 1.
        // let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // // 2.
        let proj = Matrix4::new_orthographic(-1.0, 1.0, -1.0, 1.0, 0.0, 100.0);

        // 3.
        return proj;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn from_camera_2d(camera: &Camera2D) -> Self {
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(camera);

        uniform
    }

    pub fn update_view_proj(&mut self, camera: &Camera2D) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

// Credit(https://github.com/not-fl3/good-web-game/blob/master/src/graphics/image.rs#L129)
// pub(crate) fn param_to_instance_transform(
//     rotation: f32,
//     scale: Vec2,
//     offset: Vec2,
//     dest: Vec2,
// ) -> Mat4 {
//     let cosr = rotation.cos();
//     let sinr = rotation.sin();
//     let m00 = cosr * scale.x;
//     let m01 = -sinr * scale.y;
//     let m10 = sinr * scale.x;
//     let m11 = cosr * scale.y;
//     let m03 = offset.x * (1.0 - m00) - offset.y * m01 + dest.x;
//     let m13 = offset.y * (1.0 - m11) - offset.x * m10 + dest.y;

//     Mat4::from_cols(
//         Vec4::new(m00, m10, 0.0, 0.0),
//         Vec4::new(m01, m11, 0.0, 0.0),
//         Vec4::new(0.0, 0.0, 1.0, 0.0),
//         Vec4::new(m03, m13, 0.0, 1.0),
//     )
// }
