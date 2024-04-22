use cgmath::{Array, SquareMatrix};
use winit::dpi::LogicalSize;
use winit::window::Window;

pub(crate) fn window_to_wgpu_transform(window: &Window) -> cgmath::Matrix4<f32> {
    let logical_size = window.inner_size().to_logical(window.scale_factor());
    let w: f32 = logical_size.width;
    let h: f32 = logical_size.height;

    cgmath::Matrix4::from_nonuniform_scale(2.0 / w, -2.0 / h, 1.0)
        * cgmath::Matrix4::from_translation(cgmath::Vector3::new(-w / 2.0, -h / 2.0, 0.0))
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    pub(crate) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Vertex::ATTRIBUTES,
        }
    }
}

impl From<cgmath::Vector3<f32>> for Vertex {
    fn from(value: cgmath::Vector3<f32>) -> Self {
        Self {
            position: value.into(),
        }
    }
}
