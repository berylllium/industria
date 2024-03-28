use ash::{extensions::khr::Surface, vk};
use super::vkcontext::{VkContext, QueueFamilyIndices};
use super::utility::create_image_view;

pub struct Swapchain {
    pub out_of_date: bool,

    pub image_views: Vec<vk::ImageView>,
    pub images: Vec<vk::Image>,

    pub swapchain_properties: SwapchainProperties,

    pub handle: vk::SwapchainKHR,
}

impl Swapchain {
    pub fn new(
        vkcontext: &VkContext,
        queue_family_indices: QueueFamilyIndices,
    ) -> Self {
        let details = SwapchainSupportDetails::query(
            vkcontext.physical_device,
            &vkcontext.loaders.surface,
            vkcontext.surface_khr
        );

        let properties = details.get_ideal_swapchain_properties();

        let format = properties.format;
        let present_mode = properties.present_mode;
        let extent = properties.extent;

        let image_count = {
            let max = details.capabilities.max_image_count;
            let mut preferred = details.capabilities.min_image_count + 1;
            if max > 0 && preferred > max {
                preferred = max;
            }
            preferred
        };

        log::debug!(
            "Creating swapchain.\n\tFormat: {:?}\n\tColorSpace:{:?}\n\tPresentMode:{:?}\n\tExtent:{:?}\n\tImageCount:{:?}",
            format.format,
            format.color_space,
            present_mode,
            extent,
            image_count,
        );

        let graphics = queue_family_indices.graphics_index;
        let present = queue_family_indices.present_index;
        let families_indices = [graphics, present];

        let create_info = {
            let mut builder = vk::SwapchainCreateInfoKHR::builder()
                .surface(vkcontext.surface_khr)
                .min_image_count(image_count)
                .image_format(format.format)
                .image_color_space(format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

            builder = if graphics != present {
                builder
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&families_indices)
            } else {
                builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            };

            builder
                .pre_transform(details.capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .build()
        };

        let swapchain =
            unsafe { vkcontext.loaders.swapchain.create_swapchain(&create_info, None).unwrap() };
        let images = unsafe { vkcontext.loaders.swapchain.get_swapchain_images(swapchain).unwrap() };
        
        let image_views = images
            .iter()
            .map(|image| {
                create_image_view(
                    &vkcontext.device,
                    *image,
                    properties.format.format,
                    vk::ImageAspectFlags::COLOR,
                    1
                )
            })
            .collect::<Vec<_>>();

        Self {
            out_of_date: false,
            image_views,
            images,
            swapchain_properties: properties,
            handle: swapchain,
        }
    }

    pub fn destroy(&self, vkcontext: &VkContext) {
        // Free image views.
        for image_view in self.image_views.iter() {
            unsafe { vkcontext.device.destroy_image_view(*image_view, None) };
        }

        unsafe { vkcontext.loaders.swapchain.destroy_swapchain(self.handle, None) };
    }
}

impl Swapchain {
    pub fn acquire_next_image_index(
        &mut self,
        vkcontext: &VkContext,
        image_available_semaphore: vk::Semaphore
    ) -> Option<u32> {
        let result = unsafe {
            vkcontext.loaders.swapchain.acquire_next_image(
                self.handle,
                std::u64::MAX,
                image_available_semaphore,
                vk::Fence::null())
        };

        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                log::debug!("Swapchain out of date.");
                self.out_of_date = true;
                return None;
            },
            _ => return None
        };

        Some(image_index)
    }

    pub fn present(
        &mut self,
        vkcontext: &VkContext,
        render_complete_semaphore: vk::Semaphore,
        present_image_index: u32
    ) -> bool {
        let wait_semaphores = [render_complete_semaphore];
        let swapchains = [self.handle];
        let image_indices = [present_image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();

        let result = unsafe {
            vkcontext.loaders.swapchain.queue_present(vkcontext.present_queue, &present_info)
        };

        match result {
            Ok(true) => return true,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                log::debug!("Swapchain out of date.");
                self.out_of_date = true;
            },
            Err(error) => panic!("Failed to present swapchain: {}", error),
            _ => {}
        }

        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SwapchainProperties {
    pub format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn query(device: vk::PhysicalDevice, surface: &Surface, surface_khr: vk::SurfaceKHR) -> Self {
        let capabilities = unsafe {
            surface
                .get_physical_device_surface_capabilities(device, surface_khr)
                .unwrap()
        };

        let formats = unsafe {
            surface
                .get_physical_device_surface_formats(device, surface_khr)
                .unwrap()
        };

        let present_modes = unsafe {
            surface
                .get_physical_device_surface_present_modes(device, surface_khr)
                .unwrap()
        };

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }

    pub fn get_ideal_swapchain_properties(&self) -> SwapchainProperties {
        let format = Self::choose_swapchain_surface_format(&self.formats);
        let present_mode = Self::choose_swapchain_surface_present_mode(&self.present_modes);
        let extent = Self::choose_swapchain_extent(self.capabilities);

        SwapchainProperties {
            format,
            present_mode,
            extent,
        }
    }

    fn choose_swapchain_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        if available_formats.len() == 1 && available_formats[0].format == vk::Format::UNDEFINED {
            return vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            };
        }

        *available_formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&available_formats[0])
    }

    fn choose_swapchain_surface_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if available_present_modes.contains(&vk::PresentModeKHR::FIFO) {
            vk::PresentModeKHR::FIFO
        } else {
            vk::PresentModeKHR::IMMEDIATE
        }
    }

    fn choose_swapchain_extent(capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        capabilities.current_extent
    }

}
