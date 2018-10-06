#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::WaylandSurface;
use ash::extensions::{DebugReport, Surface, Swapchain};
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::Entry;
use ash::{vk, vk_make_version, Instance};
use std::ffi::{CStr, CString};
use std::ptr;

fn main() {
    let instance = create_instance();
    let physical_device = pick_physical_device(&instance);
    println!("Hello, Vulkan!");
}

fn create_instance() -> Instance<V1_0> {
    let app_name = CString::new("Niagara-rs").unwrap();
    let raw_name = app_name.as_ptr();
    let entry = Entry::new().unwrap();
    let appinfo = vk::types::ApplicationInfo {
        s_type: vk::StructureType::ApplicationInfo,
        api_version: vk_make_version!(1, 0, 36),
        p_application_name: raw_name,
        p_engine_name: raw_name,
        application_version: 0,
        engine_version: 0,
        p_next: ptr::null(),
    };
    let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();
    let extension_names_raw = extension_names();
    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::InstanceCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        p_application_info: &appinfo,
        pp_enabled_layer_names: layers_names_raw.as_ptr(),
        enabled_layer_count: layers_names_raw.len() as u32,
        pp_enabled_extension_names: extension_names_raw.as_ptr(),
        enabled_extension_count: extension_names_raw.len() as u32,
    };
    unsafe {
        let instance = entry
            .create_instance(&create_info, None)
            .expect("Instance creation error");
        let debug_info = vk::DebugReportCallbackCreateInfoEXT {
            s_type: vk::StructureType::DebugReportCallbackCreateInfoExt,
            p_next: ptr::null(),
            flags: vk::DEBUG_REPORT_ERROR_BIT_EXT
                | vk::DEBUG_REPORT_WARNING_BIT_EXT
                | vk::DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT,
            pfn_callback: vulkan_debug_callback,
            p_user_data: ptr::null_mut(),
        };
        let debug_report_loader =
            DebugReport::new(&entry, &instance).expect("Unable to load debug report");
        let _debug_call_back = debug_report_loader
            .create_debug_report_callback_ext(&debug_info, None)
            .unwrap();
        return instance;
    }
}

fn pick_physical_device(instance: &Instance<V1_0>) {
    println!("hello?");
    let physical_devices = instance
        .enumerate_physical_devices()
        .expect("Physical device error");
    println!("hello?");
    if physical_devices.len() == 0 {
        panic!("No GPU found!");
    }
    physical_devices.iter().for_each(|physical_device| {
        println!("{:?}", physical_device);
    });
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        WaylandSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: vk::uint64_t,
    _: vk::size_t,
    _: vk::int32_t,
    _: *const vk::c_char,
    p_message: *const vk::c_char,
    _: *mut vk::c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    vk::VK_FALSE
}
