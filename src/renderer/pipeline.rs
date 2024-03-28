use ash::vk;
use super::vkcontext::VkContext;

pub struct Pipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

impl Pipeline {
    pub fn new_compute(
        vkcontext: &VkContext,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
        compute_stage_create_info: vk::PipelineShaderStageCreateInfo,
    ) -> Self {


        let layout = { 
            let create_info = vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(descriptor_set_layouts)
                .build();

            unsafe { vkcontext.device.create_pipeline_layout(&create_info, None).unwrap() }
        };


        let handle = {
            let create_info = vk::ComputePipelineCreateInfo::builder()
                .stage(compute_stage_create_info)
                .layout(layout)
                .build();

            let create_infos = [create_info];
            
            unsafe {
                vkcontext.device
                .create_compute_pipelines(vk::PipelineCache::null(), &create_infos, None)
                .unwrap()[0]
            }
        };

        Self {
            handle,
            layout,
        }
    }

    pub fn destroy(&self, vkcontext: &VkContext){
        unsafe {
            vkcontext.device.destroy_pipeline_layout(self.layout, None);
            vkcontext.device.destroy_pipeline(self.handle, None);
        }
    }
}
