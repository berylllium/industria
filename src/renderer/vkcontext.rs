use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    vk, Device, Entry, Instance,
};

pub struct VkContext {
    entry: Entry,
    instance: Instance,
    debug_report_callback: Option<(DebugUtils, vk::DebugUtilsMessengerEXT)>,
    surface: Surface,
    surface_khr: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    device: Device,
}

impl VkContext {
    pub fn new(
        entry: Entry,
        instance: Instance,
        debug_report_callback: Option<(DebugUtils, vk::DebugUtilsMessengerEXT)>,
        surface: Surface,
        surface_khr: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        device: Device,
    ) -> Self {
        VkContext {
            entry,
            instance,
            debug_report_callback,
            surface,
            surface_khr,
            physical_device,
            device,
        }
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            if let Some((utils, messenger)) = self.debug_report_callback.take() {
                utils.destroy_debug_utils_messenger(messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
