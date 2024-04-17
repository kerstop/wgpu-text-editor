pub trait Widget {
    fn render(&self, pass: &mut wgpu::RenderPass);

}

pub struct Button;

impl Widget for Button {
    fn render(&self, pass: &mut wgpu::RenderPass) {
        //pass.draw
    }
}