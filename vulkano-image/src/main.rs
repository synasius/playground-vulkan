extern crate image;
extern crate vulkano;

use image::{ImageBuffer, Rgba};

use std::sync::Arc;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::QueuesIter;

use vulkano::format::ClearValue;
use vulkano::format::Format;

use vulkano::image::Dimensions;
use vulkano::image::StorageImage;

use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

use vulkano::sync::GpuFuture;

fn main() {
    let (device, mut queues) = init();

    // select the first queue found
    let queue = queues.next().unwrap();

    println!("Creating image");
    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: 1024,
            height: 1024,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    ).unwrap();

    // Here we put the image so we can read values
    let buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        (0..1024 * 1024 * 4).map(|_| 0u8),
    ).expect("failed to create the buffer");

    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .clear_color_image(image.clone(), ClearValue::Float([0.0, 0.0, 1.0, 1.0]))
        .unwrap()
        .copy_image_to_buffer(image.clone(), buffer.clone())
        .unwrap()
        .build()
        .unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    // save buffer to an image file
    let buffer_content = buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
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
