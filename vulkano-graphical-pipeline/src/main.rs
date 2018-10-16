#[macro_use]
extern crate vulkano;

mod init;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl_vertex!(Vertex, position);

fn main() {
    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [ 0.0,  0.5] };
    let vertex3 = Vertex { position: [ 0.5, -0.25] };
}
