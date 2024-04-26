use std::sync::OnceLock;

use crate::{math::Vertex, RenderContext};
use bytemuck::bytes_of;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, RenderPass};

const RECT_VERTEX_COUNT: u32 = 4;
const RECT_INDEX_COUNT: u32 = 6;
static RECT_INDEX_BUFFER: OnceLock<Buffer> = OnceLock::new();

#[non_exhaustive]
pub enum Event {
    MouseEnter,
    MouseExit,
    Clicked,
}

pub trait Widget {
    fn bounds(&self) -> Bounds;
    fn process_event(&mut self, event: &Event);
    fn render<'a>(&'a self, pass: &mut RenderPass<'a>);
}

pub struct Bounds {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Bounds {
    fn is_inside(&self, (x, y): (f32, f32)) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

pub struct Rect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    vertex_buffer: Buffer,
}

impl Rect {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        render_context: &mut RenderContext,
    ) -> Self {
        let z = 0.0;
        let points = [
            Vertex {
                position: [x, y, z],
            },
            Vertex {
                position: [x + width, y, z],
            },
            Vertex {
                position: [x + width, y + height, z],
            },
            Vertex {
                position: [x, y + height, z],
            },
        ];

        let vertex_buffer = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytes_of(&points),
                usage: BufferUsages::VERTEX,
            });

        let _ = RECT_INDEX_BUFFER.get_or_init(|| {
            let indecies: [u16; 6] = [0, 1, 2, 2, 3, 0];
            render_context
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytes_of(&indecies),
                    usage: BufferUsages::INDEX,
                })
        });

        Rect {
            x,
            y,
            width,
            height,
            vertex_buffer,
        }
    }
}

impl Widget for Rect {
    fn bounds(&self) -> Bounds {
        Bounds {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }

    fn process_event(&mut self, event: &Event) {
        match event {
            _ => (),
        }
    }

    fn render<'a>(&'a self, pass: &mut RenderPass<'a>) {
        pass.set_index_buffer(
            RECT_INDEX_BUFFER
                .get()
                .expect("at least one rect should have been constructed before one gets rendered")
                .slice(..),
            wgpu::IndexFormat::Uint16,
        );
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw_indexed(0..RECT_INDEX_COUNT, 0, 0..1);
    }
}
