fn example() {
    // Стандартный распределитель, подходит для большинства выделений
    let free_list_memory_allocator = GenericMemoryAllocator::<Arc<FreeListAllocator>>::new_default(device.clone());

    // Лучше подходит для выделений на очень короткий период с полным сбросом
    let bump_memory_allocator = Arc::new(GenericMemoryAllocator::<Arc<BumpAllocator>>::new_default(device.clone()));

    // Лучше подходит для одинаковых выделений
    let pool_memory_allocator = GenericMemoryAllocator::<Arc<PoolAllocator<{ 64 * 1024 }>>>::new(
        device.clone(),
        GenericMemoryAllocatorCreateInfo {
            block_sizes: &[(0, 64 * 1024 * 1024)],
            allocation_type: AllocationType::Linear,
            ..Default::default()
        },
    ).unwrap();

    // Может подойти для выделения большого количества изображений разных размеров?
    let buddy_memory_allocator = GenericMemoryAllocator::<Arc<BuddyAllocator>>::new(
        device.clone(),
        GenericMemoryAllocatorCreateInfo {
            block_sizes: &[(0, 64 * 1024 * 1024)],
            ..Default::default()
        },
    )
    .unwrap();

    let data: Vec<i32> = (0..99_999).collect();

    let free_list_local_buffer = DeviceLocalBuffer::<i32>::new(
        &free_list_memory_allocator,
        BufferUsage {
            transfer_dst: true,
            storage_buffer: true,
            ..Default::default()
        },
        device.active_queue_family_indices().iter().copied()
    ).unwrap();
    println!("{}", byte_size(free_list_local_buffer.size() / 8));

    let bump_buffer = CpuBufferPool::new(
        Arc::new(bump_memory_allocator.clone()),
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        MemoryUsage::Upload,
    );
    bump_buffer.from_iter(data.clone()).unwrap();

    let bump_attachment_image = AttachmentImage::new(
        &bump_memory_allocator,
        [800, 600],
        Format::R8G8B8A8_SRGB
    ).unwrap();
    let bump_image_view = ImageView::new_default(bump_attachment_image.clone());

    let pool_buffer = CpuAccessibleBuffer::from_iter(
        &pool_memory_allocator,
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        false,
        data.clone()
    ).unwrap();
    println!("{}", ByteSize(pool_buffer.size()));

    let buddy_buffer = CpuAccessibleBuffer::from_iter(
        &buddy_memory_allocator,
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        false,
        data.clone()
    ).unwrap();
    println!("{}", ByteSize(buddy_buffer.size()));
}
