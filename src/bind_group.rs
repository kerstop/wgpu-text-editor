use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, Buffer, BufferBindingType, Device, RenderPass, ShaderStages,
};
pub trait BindGroupInstance {
    fn bind_group(&self) -> &BindGroup;

    fn bind<'a>(&'a self, index: u32, pass: &mut RenderPass<'a>) {
        pass.set_bind_group(index, self.bind_group(), &[])
    }

    const LAYOUT_DESCRIPTOR: &'static BindGroupLayoutDescriptor<'static>;
}

pub struct WindowBindGroup {
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    pub window_to_device_transform_buffer: Buffer,
    pub window_size_buffer: Buffer,
}

impl WindowBindGroup {
    pub fn new(device: &Device, window: &winit::window::Window) -> Self {
        let transform = crate::math::window_to_wgpu_transform(window);
        let window_to_device_transform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &bytemuck::cast_slice(AsRef::<[f32; 16]>::as_ref(&transform)),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let window_size = window.inner_size().to_logical(window.scale_factor());
        let window_size_vector: [f32; 4] = [window_size.width, window_size.height, 0.0, 0.0];

        let window_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&window_size_vector),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(WindowBindGroup::LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("WindowBindGroup"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: window_to_device_transform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: window_size_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            bind_group,
            bind_group_layout,
            window_to_device_transform_buffer,
            window_size_buffer,
        }
    }
}

impl BindGroupInstance for WindowBindGroup {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    const LAYOUT_DESCRIPTOR: &'static BindGroupLayoutDescriptor<'static> =
        &BindGroupLayoutDescriptor {
            label: Some("WindowBindGroup"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };
}
