

pub fn shm_to_vulkan_format(shm_format: ews::ShmFormat)->Option<screen_task::TextureFormat> {
    match shm_format {
        ews::ShmFormat::Rgba8888=>Some(screen_task::TextureFormat::Rgba8Snorm),
        _=>None
    }
}

pub fn shm_convert_format(data: &[u8],info: ews::BufferData)->screen_task::SurfaceSource {
    let data = &data[info.offset as usize..(info.offset+info.height*info.stride) as usize];
    match info.format {
        ews::ShmFormat::Rgba8888=>{
            let info = screen_task::HostAllocationInfo {
                size: [info.width as u32,info.height as u32],
                stride: info.stride as u32,
                format: wgpu_engine::TextureFormat::Rgba8UnormSrgb
            };
            let data = data.to_vec();
            screen_task::SurfaceSource::HostAllocation {info,data}
        },
        ews::ShmFormat::Argb8888 | ews::ShmFormat::Xrgb8888=>{
            let packed_data: &[u32] = bytemuck::cast_slice(data);
            let data: Vec<u8> = packed_data.iter().map(|data|*data << 8).map(|data|data.to_le_bytes()).flatten().collect();
            let info = screen_task::HostAllocationInfo {
                size: [info.width as u32,info.height as u32],
                stride: info.stride as u32,
                format: wgpu_engine::TextureFormat::Rgba8UnormSrgb
            };
            let data = data.to_vec();
            screen_task::SurfaceSource::HostAllocation {info,data}
        /*

            let img: image::ImageBuffer<image::Rgba<u8>, Vec<_>> = image::ImageBuffer::from_vec(info.width as u32,info.height as u32],data.clone()).unwrap();
            width = img.dimensions().0;
            height = img.dimensions().1;
            depth_or_array_layers = 1;
            let sample_layout = img.sample_layout();

            image_layout = wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(sample_layout.width_stride as u32),
                    //sample_layout.width * sample_layout.cha as u32 * 1,
                //),
                rows_per_image: std::num::NonZeroU32::new(sample_layout.height),
            };

            texture_data = Some(img.into_raw());
            texture_source = TextureSource::Local;
            texture_format = wgpu::TextureFormat::Rgba8UnormSrgb
            */
        },
        _=>panic!()
    }
}
