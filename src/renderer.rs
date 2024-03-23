mod debug;
mod swapchain;
mod utility;
mod vkcontext;

use swapchain::Swapchain;
use vkcontext::VkContext;

use winit::window::Window;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub struct Renderer {
    swapchain: Swapchain,
    vk_context: VkContext,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        // Create context.
        
        let vk_context = VkContext::new(window);

        let swapchain = Swapchain::new(&vk_context, vk_context.queue_family_indices, [WIDTH, HEIGHT]);

        Renderer {
            swapchain,
            vk_context,
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.swapchain.free(&self.vk_context);
        self.vk_context.free();
    }
}
