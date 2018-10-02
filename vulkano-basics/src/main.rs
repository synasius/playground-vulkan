#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::QueuesIter;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;

use vulkano::sync::GpuFuture;

use vulkano::pipeline::ComputePipeline;

use std::sync::Arc;

mod cs {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[src = "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"]
    struct Dummy;
}

fn main() {
    //copy_buffer();
    multiply();
}

fn multiply() {
    let (device, mut queues) = init();

    // select the first queue found
    let queue = queues.next().unwrap();

    let data_iter = 0..65536;
    let data_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter)
        .expect("failed to create buffer");

    let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
            .expect("failed to create compute pipeline"),
    );

    let set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_buffer(data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .dispatch([1024, 1, 1], compute_pipeline.clone(), set.clone(), ())
        .unwrap()
        .build()
        .unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }
}

/* A first exercise where we ask the GPU to copy the content of a buffer into
 * second one.
 */
fn copy_buffer() {
    let (device, mut queues) = init();

    // select the first queue found
    let queue = queues.next().unwrap();

    // let's start to do something with the GPU
    let source_content = 0..64;
    let source = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), source_content)
        .expect("failed to create buffer");

    let dest_content = (0..64).map(|_| 0);
    let dest = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), dest_content)
        .expect("failed to create buffer");

    // create a command buffer
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .copy_buffer(source.clone(), dest.clone())
        .unwrap()
        .build()
        .unwrap();

    // wait for the execution on the GPU to finish
    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    // read results
    let src_content = source.read().unwrap();
    let dest_content = dest.read().unwrap();
    assert_eq!(&*src_content, &*dest_content);
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
