extern crate image;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

mod core;
mod shaders;

use image::{ImageBuffer, Rgba};

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::command_buffer::DynamicState;

use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;

use vulkano::image::Dimensions;
use vulkano::image::StorageImage;

use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;

use vulkano::sync::GpuFuture;

use std::sync::Arc;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl_vertex!(Vertex, position);

fn main() {
    let size_x = 1024;
    let size_y = 1024;

    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };

    let (device, mut queues) = core::init();

    let queue = queues.next().expect("Couldn't get the first queue");

    // Create a buffer to read the resulting image
    let buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        (0 .. size_x * size_y * 4).map(|_| 0u8),
    ).expect("failed to create the buffer");

    // create a buffer for vertices
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        vec![vertex1, vertex2, vertex3].into_iter(),
    ).expect("failed to create buffer");

    // create the vertex and fragment shader
    let vs = shaders::vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = shaders::fs::Shader::load(device.clone()).expect("failed to create shader module");

    // Create render pass
    let render_pass = Arc::new(
        single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap(),
    );

    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: size_x,
            height: size_y,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    ).unwrap();

    // Create a framebuffer
    let framebuffer = Arc::new(
        Framebuffer::start(render_pass.clone())
            .add(image.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // Create the graphical pipeline
    let pipeline = Arc::new(
        GraphicsPipeline::start()
        // Defines what kind of vertex input is expected.
        .vertex_input_single_buffer::<Vertex>()
        // The vertex shader.
        .vertex_shader(vs.main_entry_point(), ())
        // Defines the viewport (explanations below).
        .viewports_dynamic_scissors_irrelevant(1)
        // The fragment shader.
        .fragment_shader(fs.main_entry_point(), ())
        // This graphics pipeline object concerns the first pass of the render pass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that everything is specified, we call `build`.
        .build(device.clone())
        .unwrap(),
    );

    let dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [1024.0, 1024.0],
            depth_range: 0.0 .. 1.0,
        }]),
        .. DynamicState::none()
    };

    let command_buffer =
        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
            .unwrap()
            .begin_render_pass(
                framebuffer.clone(),
                false,
                vec![[0.0, 0.0, 1.0, 1.0].into()],
            ).unwrap()
            .draw(
                pipeline.clone(),
                &dynamic_state,
                vertex_buffer.clone(),
                (),
                (),
            ).unwrap()
            .end_render_pass()
            .unwrap()
            .copy_image_to_buffer(image.clone(), buffer.clone())
            .unwrap()
            .build()
            .unwrap();

    // execute the pipeline
    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    // save buffer to an image file
    let buffer_content = buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(size_x, size_y, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
}
