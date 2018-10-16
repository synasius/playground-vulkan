extern crate vulkano;


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
