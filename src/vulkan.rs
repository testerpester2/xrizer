use crate::vr;
use arc_swap::ArcSwapOption;
use ash::vk::{self, Handle};
use openxr as xr;
use std::ffi::{c_char, CString};

#[derive(Debug)]
pub struct RealSessionData {
    images: Vec<vk::Image>,
    pool: vk::CommandPool,
    bufs: Vec<vk::CommandBuffer>,
}

pub struct VulkanData {
    _entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    real_data: ArcSwapOption<RealSessionData>,
}

impl Drop for VulkanData {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
        match &*self.real_data.load() {
            // Temporary session - we created these handles, so let's destroy them
            None => unsafe {
                self.device.destroy_device(None);
                self.instance.destroy_instance(None);
            },
            // Real session - the handles come from the app, only destroy the command pool we created
            Some(bufs) => unsafe {
                self.device.destroy_command_pool(bufs.pool, None);
            },
        }
    }
}

impl VulkanData {
    pub fn get_swapchain_create_info(
        &self,
        texture: &vr::VRVulkanTextureData_t,
        bounds: vr::VRTextureBounds_t,
        color_space: vr::EColorSpace,
    ) -> xr::SwapchainCreateInfo<xr::vulkan::Vulkan> {
        let (extent, _) = texture_extent_from_bounds(texture, bounds);
        xr::SwapchainCreateInfo {
            create_flags: xr::SwapchainCreateFlags::EMPTY,
            usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT
                | xr::SwapchainUsageFlags::TRANSFER_DST,
            format: get_colorspace_corrected_format(
                vk::Format::from_raw(texture.m_nFormat as _),
                color_space,
            )
            .as_raw() as _,
            sample_count: texture.m_nSampleCount,
            width: extent.width,
            height: extent.height,
            face_count: 1,
            array_size: 2,
            mip_count: 1,
        }
    }

    pub fn copy_texture_to_swapchain(
        &self,
        eye: vr::EVREye,
        texture: &vr::VRVulkanTextureData_t,
        bounds: vr::VRTextureBounds_t,
        image_index: usize,
        submit_flags: vr::EVRSubmitFlags,
    ) -> xr::Extent2Di {
        let (texture, array_data) =
            if (submit_flags & vr::EVRSubmitFlags::Submit_VulkanTextureWithArrayData).0 > 0 {
                let data = unsafe {
                    &*(texture as *const vr::VRVulkanTextureData_t
                        as *const vr::VRVulkanTextureArrayData_t)
                };
                (&data._base, Some(data))
            } else {
                (texture, None)
            };

        let guard = self.real_data.load();
        let data = guard.as_ref().unwrap();
        let swapchain_image = data.images[image_index];
        let buf = data.bufs[2 * image_index + eye as usize];
        drop(guard);

        let (extent, offset) = texture_extent_from_bounds(texture, bounds);
        log::trace!("{:?} extent: {:?}", eye, extent);
        unsafe {
            self.device
                .begin_command_buffer(
                    buf,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .unwrap();

            // transition swapchain image to TRANSFER_DST
            let swapchain_res = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: eye as u32,
                layer_count: 1,
            };

            self.device.cmd_pipeline_barrier(
                buf,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier {
                    src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    dst_access_mask: vk::AccessFlags::TRANSFER_READ,
                    old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    image: swapchain_image,
                    subresource_range: swapchain_res,
                    ..Default::default()
                }],
            );

            // transfer image
            let subresource = vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            };

            let game_image = vk::Image::from_raw(texture.m_nImage);
            let game_layer = array_data.map(|d| d.m_unArrayIndex).unwrap_or(0);

            let copy = vk::ImageResolve {
                src_subresource: vk::ImageSubresourceLayers {
                    base_array_layer: game_layer,
                    ..subresource
                },
                src_offset: offset,
                dst_subresource: vk::ImageSubresourceLayers {
                    base_array_layer: eye as u32,
                    ..subresource
                },
                dst_offset: vk::Offset3D::default(),
                extent,
            };
            if texture.m_nSampleCount > 1 {
                self.device.cmd_resolve_image(
                    buf,
                    game_image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    swapchain_image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[copy],
                );
            } else {
                self.device.cmd_copy_image(
                    buf,
                    game_image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    swapchain_image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[
                        // SAFETY: ImageResolve and ImageCopy have the same fields and layout.
                        #[allow(unused_unsafe)]
                        unsafe {
                            std::mem::transmute(copy)
                        },
                    ],
                );
            }

            // transition swapchain image back to OPTIMAL
            self.device.cmd_pipeline_barrier(
                buf,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::empty(),
                    old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    image: swapchain_image,
                    subresource_range: swapchain_res,
                    ..Default::default()
                }],
            );

            self.device.end_command_buffer(buf).unwrap();

            self.device
                .queue_submit(
                    self.queue,
                    &[vk::SubmitInfo::default().command_buffers(&[buf])],
                    vk::Fence::null(),
                )
                .unwrap();
        }

        xr::Extent2Di {
            width: extent.width as _,
            height: extent.height as _,
        }
    }
}

impl VulkanData {
    pub fn as_session_create_info(&self) -> xr::vulkan::SessionCreateInfo {
        let queue_families = unsafe {
            self.instance
                .get_physical_device_queue_family_properties(self.physical_device)
        };
        let family = queue_families[self.queue_family_index as usize];
        let queue_index = (0..family.queue_count)
            .find(|idx| {
                let queue = unsafe { self.device.get_device_queue(self.queue_family_index, *idx) };
                queue == self.queue
            })
            .expect(&format!(
                "Could not find queue index for queue {:?} in family {}",
                self.queue, self.queue_family_index
            ));

        xr::vulkan::SessionCreateInfo {
            instance: self.instance.handle().as_raw() as _,
            physical_device: self.physical_device.as_raw() as _,
            device: self.device.handle().as_raw() as _,
            queue_family_index: self.queue_family_index,
            queue_index,
        }
    }

    pub fn post_swapchain_create(&self, images: Vec<vk::Image>) {
        let pool = unsafe {
            self.device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(self.queue_family_index),
                    None,
                )
                .unwrap()
        };
        let bufs = unsafe {
            self.device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        // We have to copy 2 eyes per swapchain image
                        .command_buffer_count(images.len() as u32 * 2),
                )
                .unwrap()
        };

        if let Some(data) = self
            .real_data
            .swap(Some(RealSessionData { images, pool, bufs }.into()))
        {
            unsafe {
                self.device.destroy_command_pool(data.pool, None);
            }
        }
    }

    pub fn new(data: &vr::VRVulkanTextureData_t) -> Self {
        let entry = new_entry();
        let instance = unsafe {
            ash::Instance::load(
                entry.static_fn(),
                vk::Instance::from_raw(data.m_pInstance as _),
            )
        };
        let device = unsafe {
            ash::Device::load(
                instance.fp_v1_0(),
                vk::Device::from_raw(data.m_pDevice as _),
            )
        };

        Self {
            _entry: entry,
            instance,
            physical_device: vk::PhysicalDevice::from_raw(data.m_pPhysicalDevice as _),
            device,
            queue: vk::Queue::from_raw(data.m_pQueue as _),
            queue_family_index: data.m_nQueueFamilyIndex,
            real_data: Default::default(),
        }
    }

    pub fn new_temporary(xr_instance: &xr::Instance, system_id: xr::SystemId) -> Self {
        let entry = new_entry();

        let inst_exts = xr_instance
            .vulkan_legacy_instance_extensions(system_id)
            .unwrap();
        let inst_exts: Vec<CString> = inst_exts
            .split_ascii_whitespace()
            .map(|ext| CString::new(ext).unwrap())
            .collect();
        let inst_exts: Vec<*const c_char> = inst_exts.iter().map(|ext| ext.as_ptr()).collect();

        let instance = unsafe {
            entry
                .create_instance(
                    &vk::InstanceCreateInfo::default()
                        .application_info(
                            &vk::ApplicationInfo::default()
                                .api_version(vk::API_VERSION_1_0)
                                .application_name(c"XRizer temporary session"),
                        )
                        .enabled_extension_names(&inst_exts),
                    None,
                )
                .expect("Failed to create temporary Vulkan instance")
        };

        let physical_device = vk::PhysicalDevice::from_raw(unsafe {
            xr_instance
                .vulkan_graphics_device(system_id, instance.handle().as_raw() as _)
                .expect("Failed to get temporary Vulkan physical device") as _
        });

        let dev_exts = xr_instance
            .vulkan_legacy_device_extensions(system_id)
            .unwrap();
        let dev_exts: Vec<CString> = dev_exts
            .split_ascii_whitespace()
            .map(|ext| CString::new(ext).unwrap())
            .collect();
        let dev_exts: Vec<*const c_char> = dev_exts.iter().map(|ext| ext.as_ptr()).collect();

        // find whatever graphics queue family
        let queue_family_index =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) }
                .into_iter()
                .enumerate()
                .find_map(|(idx, family)| {
                    (family.queue_flags.contains(vk::QueueFlags::GRAPHICS)).then_some(idx)
                })
                .unwrap() as u32;

        let device = unsafe {
            instance
                .create_device(
                    physical_device,
                    &vk::DeviceCreateInfo::default()
                        .queue_create_infos(std::slice::from_ref(
                            &vk::DeviceQueueCreateInfo::default()
                                .queue_family_index(queue_family_index)
                                .queue_priorities(&[1.0]),
                        ))
                        .enabled_extension_names(&dev_exts),
                    None,
                )
                .expect("Could not create temporary vulkan device")
        };

        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Self {
            _entry: entry,
            instance,
            physical_device,
            device,
            queue,
            queue_family_index,
            real_data: Default::default(),
        }
    }
}

fn get_colorspace_corrected_format(format: vk::Format, color_space: vr::EColorSpace) -> vk::Format {
    // https://github.com/ValveSoftware/openvr/wiki/Vulkan#image-formats
    match color_space {
        vr::EColorSpace::ColorSpace_Auto | vr::EColorSpace::ColorSpace_Gamma => match format {
            vk::Format::R8G8B8A8_UNORM | vk::Format::R8G8B8A8_SRGB => vk::Format::R8G8B8A8_SRGB,
            vk::Format::B8G8R8A8_UNORM | vk::Format::B8G8R8A8_SRGB => vk::Format::B8G8R8A8_SRGB,
            _ => panic!("Unhandled texture format: {format:?}"),
        },
        vr::EColorSpace::ColorSpace_Linear => todo!(),
    }
}

fn texture_extent_from_bounds(
    texture: &vr::VRVulkanTextureData_t,
    bounds: vr::VRTextureBounds_t,
) -> (vk::Extent3D, vk::Offset3D) {
    let width_min = bounds.uMin * texture.m_nWidth as f32;
    let width_max = bounds.uMax * texture.m_nWidth as f32;
    let height_min = bounds.vMin * texture.m_nHeight as f32;
    let height_max = bounds.vMax * texture.m_nHeight as f32;

    (
        vk::Extent3D {
            width: (width_max - width_min).abs() as u32,
            height: (height_max - height_min).abs() as u32,
            depth: 1,
        },
        vk::Offset3D {
            x: width_min.min(width_max) as i32,
            y: height_min.min(height_max) as i32,
            z: 0,
        },
    )
}

fn new_entry() -> ash::Entry {
    #[cfg(not(test))]
    unsafe {
        ash::Entry::load().unwrap()
    }

    #[cfg(test)]
    unsafe {
        ash::Entry::from_static_fn(ash::StaticFn {
            get_instance_proc_addr: fakexr::vulkan::get_instance_proc_addr,
        })
    }
}
