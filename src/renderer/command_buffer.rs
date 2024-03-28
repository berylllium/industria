use ash::vk;
use super::vkcontext::VkContext;

pub struct CommandBuffer {
    pub handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn new(vkcontext: &VkContext, command_pool: vk::CommandPool, is_primary: bool) -> Self {
        let handle = {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(if is_primary {vk::CommandBufferLevel::PRIMARY} else {vk::CommandBufferLevel::SECONDARY})
                .command_buffer_count(1)
                .build();

            unsafe { vkcontext.device.allocate_command_buffers(&allocate_info).unwrap()[0] }
        };

        Self {
            handle,
        }
    }

    pub fn destroy(&mut self, vkcontext: &VkContext, command_pool: vk::CommandPool) {
        unsafe { vkcontext.device.free_command_buffers(command_pool, &[self.handle]) }
    }
}

impl CommandBuffer {
    pub fn begin(
        &self,
        vkcontext: &VkContext,
        is_single_use: bool,
        is_render_pass_continue: bool,
        is_simultaneous_use: bool
    ) {
        let mut flags = vk::CommandBufferUsageFlags::default();

        if is_single_use { flags |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT; }
        if is_render_pass_continue { flags |= vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE; }
        if is_simultaneous_use { flags |= vk::CommandBufferUsageFlags::SIMULTANEOUS_USE; }

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(flags)
            .build();

        unsafe { vkcontext.device.begin_command_buffer(self.handle, &begin_info).unwrap() }
    }

    pub fn end(&self, vkcontext: &VkContext) {
        unsafe { vkcontext.device.end_command_buffer(self.handle).unwrap() }
    }

    pub fn end_and_submit_single_use(&self, vkcontext: &VkContext, queue: vk::Queue) {
        let buffers = [self.handle];

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&buffers)
            .build();

        unsafe { vkcontext.device.queue_submit(queue, &[submit_info], vk::Fence::null()).unwrap() }
    }
}
