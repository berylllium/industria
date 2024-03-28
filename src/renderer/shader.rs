use ash::vk;
use std::ffi::CString;
use super::{command_buffer::CommandBuffer, pipeline::Pipeline, swapchain::{self, Swapchain}, vkcontext::VkContext};
use crate::container::FreeList;

pub struct VoxelShader {
    max_instance_count: u32,
    instances: FreeList<VoxelShaderInstance>,

    global_sets: Vec<vk::DescriptorSet>,

    global_set_layout: vk::DescriptorSetLayout,
    instance_set_layout: vk::DescriptorSetLayout,

    global_descriptor_pool: vk::DescriptorPool,
    instance_descriptor_pool: vk::DescriptorPool,

    pipeline: Pipeline,
}

impl VoxelShader {
    pub fn new(vkcontext: &VkContext, swapchain_image_count: u32) -> Self {
        let max_instance_count = 1000u32;

        let stage = ShaderStage::new(vkcontext, "shaders/voxel.spv", vk::ShaderStageFlags::COMPUTE);

        // Global set layout.
        let global_set_layout = {
            let bindings = [
                // Color buffer binding.
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .build(),
                // Environment buffer binding.
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .build(),
            ];

            let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&bindings)
                .build();

            unsafe { vkcontext.device.create_descriptor_set_layout(&create_info, None).unwrap() }
        };

        // Instance set layout.
        let instance_set_layout = {
            let bindings = [
                // Octree Nodes.
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .build(),
                // Voxels.
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .build(),
            ];

            let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&bindings)
                .build();

            unsafe { vkcontext.device.create_descriptor_set_layout(&create_info, None).unwrap() }
        };

        let global_descriptor_pool = {
            let sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: swapchain_image_count,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: swapchain_image_count,
                },
            ];

            let create_info = vk::DescriptorPoolCreateInfo::builder()
                .max_sets(swapchain_image_count)
                .pool_sizes(&sizes)
                .build();

            unsafe { vkcontext.device.create_descriptor_pool(&create_info, None).unwrap() }
        };

        let instance_descriptor_pool = {
            let sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 2 * swapchain_image_count,
                },
            ];

            let create_info = vk::DescriptorPoolCreateInfo::builder()
                .max_sets(swapchain_image_count * max_instance_count)
                .pool_sizes(&sizes)
                .build();

            unsafe { vkcontext.device.create_descriptor_pool(&create_info, None).unwrap() }
        };

        let global_sets = {
            let global_set_layouts = vec![global_set_layout; swapchain_image_count as usize];

            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(global_descriptor_pool)
                .set_layouts(&global_set_layouts)
                .build();

            unsafe { vkcontext.device.allocate_descriptor_sets(&allocate_info).unwrap() }
        };

        let pipeline = Pipeline::new_compute(
            vkcontext,
            &[global_set_layout, instance_set_layout],
            stage.shader_stage_create_info
        );

        stage.destroy(vkcontext);

        Self {
            max_instance_count,
            instances: FreeList::<VoxelShaderInstance>::with_capacity(3),
            global_sets,
            global_set_layout,
            instance_set_layout,
            global_descriptor_pool,
            instance_descriptor_pool,
            pipeline,
        }
    }

    pub fn destroy(&mut self, vkcontext: &VkContext) {
        unsafe {
            self.pipeline.destroy(vkcontext);

            vkcontext.device.destroy_descriptor_pool(self.global_descriptor_pool, None);
            vkcontext.device.destroy_descriptor_pool(self.instance_descriptor_pool, None);

            vkcontext.device.destroy_descriptor_set_layout(self.global_set_layout, None);
            vkcontext.device.destroy_descriptor_set_layout(self.instance_set_layout, None);
        }
    }
}

impl VoxelShader {
    pub fn allocate_instance(&mut self, vkcontext: &VkContext) -> u32 {
        let descriptor_sets = {
            let set_layouts = vec![self.instance_set_layout; self.global_sets.len()];

            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(self.instance_descriptor_pool)
                .set_layouts(&set_layouts)
                .build();

            unsafe { vkcontext.device.allocate_descriptor_sets(&allocate_info).unwrap() }
        };

        let instance = VoxelShaderInstance {
            descriptor_sets,
            id: 0,
        };

        let index = self.instances.push_first(instance);
        self.instances.as_slice_mut()[index].id = index as u32;

        index as u32
    }

    pub fn bind(&self, vkcontext: &VkContext, command_buffer: &CommandBuffer, image_index: u32) {
        unsafe {
            vkcontext.device.cmd_bind_pipeline(
                command_buffer.handle,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline.handle,
            );
        }

        unsafe {
            let null = [];
            let image_index = image_index as usize;
            vkcontext.device.cmd_bind_descriptor_sets(
                command_buffer.handle, 
                vk::PipelineBindPoint::COMPUTE, 
                self.pipeline.layout, 
                0, 
                &self.global_sets[image_index..=image_index],
                &null
            );
        }
    }

    pub fn update_color_buffer_descriptors(&self, vkcontext: &VkContext, swapchain: &Swapchain) {

    }
}

pub struct VoxelShaderInstance {
    id: u32,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

struct ShaderStage {
    module: vk::ShaderModule,
    shader_stage_create_info: vk::PipelineShaderStageCreateInfo,
    stage_entry_point_name: CString,
}

impl ShaderStage {
    fn new<P: AsRef<std::path::Path>>(vkcontext: &VkContext, path: P, shader_stage: vk::ShaderStageFlags) -> Self {
        let compute_code = read_shader_from_file(path);

        let module = {
            let create_info = vk::ShaderModuleCreateInfo::builder()
                .code(&compute_code)
                .build();

            unsafe { vkcontext.device.create_shader_module(&create_info, None).unwrap() }
        };

        let entry_point_name = CString::new("main").unwrap();

        let shader_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(shader_stage)
            .module(module)
            .name(&entry_point_name)
            .build();

        Self {
            module,
            shader_stage_create_info,
            stage_entry_point_name: entry_point_name,
        }
    }

    fn destroy(&self, vkcontext: &VkContext) {
        unsafe {
            vkcontext.device.destroy_shader_module(self.module, None);
        }
    }
}

fn read_shader_from_file<P: AsRef<std::path::Path>>(path: P) -> Vec<u32> {
    use crate::utility::fs;

    log::debug!("Reading shader file: {}", path.as_ref().to_str().unwrap());

    let mut cursor = fs::load(path);

    ash::util::read_spv(&mut cursor).unwrap()
}
