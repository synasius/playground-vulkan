extern crate image;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

use image::{ImageBuffer, Rgba};

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;

use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::QueuesIter;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::format::ClearValue;
use vulkano::format::Format;

use vulkano::image::Dimensions;
use vulkano::image::StorageImage;

use vulkano::sync::GpuFuture;

use vulkano::pipeline::ComputePipeline;

use std::sync::Arc;

mod cs {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[src = "

#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

void main() {
    vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

    vec2 z = vec2(0.0, 0.0);
    float i;
    float l;
    for (i = 0.0; i < 1.0; i += 0.005) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );


        l = length(z);
        if (l > 4.0) {
            break;
        }
    }

    vec4 to_write = vec4(0.0, 0.0, 0.0, 0.0);
    if (i < 0.2) {
        to_write = vec4(i, i / l, i, 1.0);
    } else if (i < 0.7) {
        to_write = vec4(vec2(i), i / l, 1.0);
    } else {
        to_write = vec4(i/ l, vec2(i), 1.0);
    }
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}"]
    struct Dummy;
}

fn main() {
    let size_x = 1024;
    let size_y = 1024;

    let (device, mut queues) = init();

    let queue = queues.next().expect("Couldn't get the first queue");

    // create the compute pipeline
    let shader = cs::Shader::load(device.clone()).expect("failed to load shader module");
    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
            .expect("failed to create compute pipeline"),
    );

    // allocate an image
    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: size_x,
            height: size_y,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    ).unwrap();

    // bind the image to the shade with a descriptor set
    let set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_image(image.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // Create a buffer to read the resulting image
    let buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        (0 .. size_x * size_y * 4).map(|_| 0u8),
    ).expect("failed to create the buffer");

    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .dispatch([size_x / 8, size_y / 8, 1], compute_pipeline.clone(), set.clone(), ())
        .unwrap()
        .copy_image_to_buffer(image.clone(), buffer.clone())
        .unwrap()
        .build()
        .unwrap();

    // execute the commands
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

fn init() -> (Arc<Device>, QueuesIter) {
    // Create an instance of the vulkan API
    let instance =
        Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");

    // List all the physical devices that support vulkan
    for physical_device in PhysicalDevice::enumerate(&instance) {
        println!("Available device: {}", physical_device.name());
    }

    // now we just get the first
    let physical = PhysicalDevice::from_index(&instance, 0).expect("no device available");

    // list all the queue families available for the device
    for family in physical.queue_families() {
        println!(
            "Found a queue family with {:?} queue(s)",
            family.queues_count()
        );
    }

    // select a queue that supports graphical operations
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, queues) = {
        Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        ).expect("failed to create device")
    };

    (device, queues)
}
