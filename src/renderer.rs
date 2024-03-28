mod command_buffer;
mod debug;
mod pipeline;
mod shader;
mod swapchain;
mod utility;
mod vkcontext;

use ash::{vk, Device};

use swapchain::Swapchain;
use vkcontext::VkContext;

use winit::window::Window;

use self::shader::VoxelShader;

const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub struct Renderer {
    voxel_shader: VoxelShader,

    current_frame: u64,

    sync_objects: Vec<SyncObject>,
    command_pool: vk::CommandPool,
    swapchain: Swapchain,
    vk_context: VkContext,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        // Create context.
        let vk_context = VkContext::new(window);

        let swapchain = Swapchain::new(&vk_context, vk_context.queue_family_indices);

        // Command pool.
        let command_pool = {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(vk_context.queue_family_indices.graphics_index)
                .build();

            unsafe { vk_context.device.create_command_pool(&create_info, None).unwrap() }
        };

        // Sync objects.
        let mut sync_objects = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_available_semaphore = {
                let create_info = vk::SemaphoreCreateInfo::builder().build();
                unsafe { vk_context.device.create_semaphore(&create_info, None).unwrap() }
            };

            let queue_complete_semaphore = {
                let create_info = vk::SemaphoreCreateInfo::builder().build();
                unsafe { vk_context.device.create_semaphore(&create_info, None).unwrap() }
            };

            let in_flight_fence = {
                let create_info = vk::FenceCreateInfo::builder()
                    .flags(vk::FenceCreateFlags::SIGNALED)
                    .build();
                unsafe { vk_context.device.create_fence(&create_info, None).unwrap() }
            };

            sync_objects.push(SyncObject {
                image_available_semaphore,
                queue_complete_semaphore,
                in_flight_fence,
            });
        }

        let voxel_shader =
            VoxelShader::new(&vk_context, swapchain.images.len() as u32);

        Renderer {
            voxel_shader,
            current_frame: 0,
            sync_objects,
            command_pool,
            swapchain,
            vk_context,
        }
    }
}

impl Renderer {
    pub fn begin_frame(&mut self) -> bool {
        let sync_object = self.next_sync_object();

        let wait_fences = [sync_object.in_flight_fence];

        // Wait for current frame to finish rendering.
        unsafe {
            self.vk_context.device.wait_for_fences(&wait_fences, true, std::u64::MAX).unwrap();
        }

        let next_image_index =
            match self.swapchain.acquire_next_image_index(&self.vk_context, sync_object.image_available_semaphore) {
                Some(next_index) => next_index,
                None => return true,
        };

        unsafe { self.vk_context.device.reset_fences(&wait_fences).unwrap() };

        true
    }

    pub fn end_frame(&self) {

    }
}

impl Renderer {
    fn next_sync_object(&mut self) -> SyncObject {
        let next = self.sync_objects[self.current_frame as usize];

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT as u64;

        next
    }

    fn recreate_swapchain(&mut self) {
        log::debug!("Recreating swapchain.");

        self.vk_context.wait_gpu_idle();

        self.swapchain.destroy(&self.vk_context);

        let swapchain = Swapchain::new(&self.vk_context, self.vk_context.queue_family_indices);

        self.swapchain = swapchain;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::debug!("Dropping renderer.");

        let device = &self.vk_context.device;

        unsafe {
            self.voxel_shader.destroy(&self.vk_context);

            for sync_object in self.sync_objects.iter() {
                sync_object.destroy(device);
            }

            device.destroy_command_pool(self.command_pool, None);
        }
        
        self.swapchain.destroy(&self.vk_context);
        self.vk_context.destroy();
    }
}

#[derive(Clone, Copy)]
struct SyncObject {
    image_available_semaphore: vk::Semaphore,
    queue_complete_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

impl SyncObject {
    fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.queue_complete_semaphore, None);
            device.destroy_fence(self.in_flight_fence, None);
        }
    }
}
