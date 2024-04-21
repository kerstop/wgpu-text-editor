use cgmath::SquareMatrix;
use winit::dpi::LogicalSize;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct WindowToWgpuMatrix {
    pub(crate) matrix: [[f32; 4]; 4],
}

impl WindowToWgpuMatrix {
    pub(crate) fn new(logical_size: LogicalSize<f32>) -> Self {
        log::debug!("{:?}", logical_size);
        let w = logical_size.width;
        let h = logical_size.height;
        log::debug!("{:?}", cgmath::ortho(0.0, w, h, 0.0, -1.0, 1.0));
        // #[rustfmt::skip]
        // let matrix = (cgmath::Matrix4::new(
        //     w,  0.0    , 0.0, -w / 2.0,
        //     0.0    , -h, 0.0, -h / 2.0,
        //     0.0    ,  0.0    , 1.0, 0.0     ,
        //     0.0    ,  0.0    , 0.0, 1.0     ,
        // ) * OPENGL_TO_WGPU_MATRIX).into();

        let matrix = cgmath::Matrix4::identity().into();

        Self { matrix }
    }
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
