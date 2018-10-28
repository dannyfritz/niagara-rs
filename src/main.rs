use ash::extensions::{DebugReport, Surface, Swapchain};
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::{WaylandSurface, XlibSurface};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::vk::Image;
use ash::vk::PhysicalDevice;
use ash::vk::Semaphore;
use ash::vk::SwapchainKHR;
use ash::Entry;
use ash::{vk, vk_make_version, Device, Instance};
use std::ffi::{CStr, CString};
use std::ptr;

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 600.0;

fn main() {
    let entry = create_entry();
    let instance = create_instance(&entry);
    let physical_device = pick_physical_device(&instance);
    let props = instance.get_physical_device_properties(physical_device);
    println!("GPU chosen: {:?}", &unsafe {
        CStr::from_ptr(&props.device_name[0])
    });
    let device = create_device(&instance, &physical_device);
    let queue_family_index: u32 = 0;
    let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_title("Ash - Example")
        .with_dimensions(winit::dpi::LogicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&events_loop)
        .unwrap();
    let (swapchain_loader, swapchain) =
        unsafe { create_swapchain(&entry, &instance, &window, physical_device, &device).unwrap() };
    let present_images = unsafe { get_present_images(&swapchain_loader, swapchain).unwrap() };
    let present_complete_semaphore = unsafe { create_semaphore(&device).unwrap() };
    let rendering_complete_semaphore = unsafe { create_semaphore(&device).unwrap() };
    let mut closed = false;
    while !closed {
        events_loop.poll_events(|event| match event {
            winit::Event::WindowEvent { event, .. } => match event {
                winit::WindowEvent::CloseRequested => closed = true,
                _ => {}
            },
            _ => {}
        });
        let present_index = unsafe {
            swapchain_loader
                .acquire_next_image_khr(
                    swapchain,
                    std::u64::MAX,
                    present_complete_semaphore,
                    vk::Fence::null(),
                )
                .unwrap()
        };
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PresentInfoKhr,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            // p_wait_semaphores: &rendering_complete_semaphore,
            p_wait_semaphores: ptr::null(),
            swapchain_count: 1,
            p_swapchains: &swapchain,
            p_image_indices: &present_index,
            p_results: ptr::null_mut(),
        };
        unsafe {
            swapchain_loader
                .queue_present_khr(present_queue, &present_info)
                .unwrap();
        }
    }
}

fn create_entry() -> Entry<V1_0> {
    Entry::new().unwrap()
}

fn create_instance(entry: &Entry<V1_0>) -> Instance<V1_0> {
    let app_name = CString::new("Niagara-rs").unwrap();
    let raw_name = app_name.as_ptr();
    let appinfo = vk::ApplicationInfo {
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
            DebugReport::new(entry, &instance).expect("Unable to load debug report");
        let _debug_call_back = debug_report_loader
            .create_debug_report_callback_ext(&debug_info, None)
            .unwrap();
        return instance;
    }
}

fn pick_physical_device(instance: &Instance<V1_0>) -> vk::PhysicalDevice {
    let physical_devices = instance
        .enumerate_physical_devices()
        .expect("Physical device error");
    if physical_devices.len() == 0 {
        panic!("No GPU found!");
    }
    let physical_device = physical_devices
        .iter()
        .max_by_key(|physical_device| {
            let props = instance.get_physical_device_properties(**physical_device);
            match props.device_type {
                vk::PhysicalDeviceType::DiscreteGpu => 2,
                vk::PhysicalDeviceType::IntegratedGpu => 1,
                _ => 0,
            }
        })
        .expect("No suitable device found!");
    return *physical_device;
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

fn create_device(instance: &Instance<V1_0>, physical_device: &vk::PhysicalDevice) -> Device<V1_0> {
    let queue_family_index = 0 as u32;
    let priorities = [1.0];
    let queue_info = vk::types::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DeviceQueueCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        queue_family_index: queue_family_index as u32,
        p_queue_priorities: priorities.as_ptr(),
        queue_count: priorities.len() as u32,
    };
    let device_extension_names_raw = [Swapchain::name().as_ptr()];
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DeviceCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        p_queue_create_infos: &queue_info,
        queue_create_info_count: 1,
        pp_enabled_layer_names: ptr::null(),
        enabled_layer_count: 0,
        pp_enabled_extension_names: device_extension_names_raw.as_ptr(),
        enabled_extension_count: device_extension_names_raw.len() as u32,
        p_enabled_features: &features,
    };
    unsafe {
        let device: Device<V1_0> = instance
            .create_device(*physical_device, &device_create_info, None)
            .unwrap();
        return device;
    }
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::unix::WindowExt;
    let x11_display = window.get_xlib_display().unwrap();
    let x11_window = window.get_xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XlibSurfaceCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };
    let xlib_surface_loader =
        XlibSurface::new(entry, instance).expect("Unable to load xlib surface");
    xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None)
}

unsafe fn create_swapchain(
    entry: &Entry<V1_0>,
    instance: &Instance<V1_0>,
    window: &winit::Window,
    physical_device: PhysicalDevice,
    device: &Device<V1_0>,
) -> Result<(Swapchain, SwapchainKHR), vk::Result> {
    let surface = create_surface(entry, instance, window).unwrap();
    let surface_loader =
        Surface::new(entry, instance).expect("Unable to load the Surface extension");
    let surface_formats = surface_loader
        .get_physical_device_surface_formats_khr(physical_device, surface)
        .unwrap();
    let surface_format = surface_formats
        .iter()
        .map(|sfmt| match sfmt.format {
            vk::Format::Undefined => vk::SurfaceFormatKHR {
                format: vk::Format::B8g8r8Unorm,
                color_space: sfmt.color_space,
            },
            _ => sfmt.clone(),
        })
        .nth(0)
        .expect("Unable to find suitable surface format.");
    let surface_capabilities = surface_loader
        .get_physical_device_surface_capabilities_khr(physical_device, surface)
        .unwrap();
    let mut desired_image_count = surface_capabilities.min_image_count + 1;
    if surface_capabilities.max_image_count > 0
        && desired_image_count > surface_capabilities.max_image_count
    {
        desired_image_count = surface_capabilities.max_image_count;
    }
    let surface_resolution = match surface_capabilities.current_extent.width {
        std::u32::MAX => vk::Extent2D {
            width: WIDTH as u32,
            height: HEIGHT as u32,
        },
        _ => surface_capabilities.current_extent,
    };
    let pre_transform = if surface_capabilities
        .supported_transforms
        .subset(vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR)
    {
        vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR
    } else {
        surface_capabilities.current_transform
    };
    let present_modes = surface_loader
        .get_physical_device_surface_present_modes_khr(physical_device, surface)
        .unwrap();
    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&mode| mode == vk::PresentModeKHR::Mailbox)
        .unwrap_or(vk::PresentModeKHR::Fifo);
    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SwapchainCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        surface: surface,
        min_image_count: desired_image_count,
        image_color_space: surface_format.color_space,
        image_format: surface_format.format,
        image_extent: surface_resolution.clone(),
        image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        image_sharing_mode: vk::SharingMode::Exclusive,
        pre_transform: pre_transform,
        composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        present_mode: present_mode,
        clipped: 1,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1,
        p_queue_family_indices: ptr::null(),
        queue_family_index_count: 0,
    };
    let swapchain_loader = Swapchain::new(instance, device).expect("Unable to load swapchain");
    let swapchain = swapchain_loader
        .create_swapchain_khr(&swapchain_create_info, None)
        .unwrap();
    Ok((swapchain_loader, swapchain))
}

unsafe fn get_present_images(
    swapchain_loader: &Swapchain,
    swapchain: SwapchainKHR,
) -> Result<Vec<Image>, vk::Result> {
    swapchain_loader.get_swapchain_images_khr(swapchain)
}

unsafe fn create_semaphore(device: &Device<V1_0>) -> Result<Semaphore, vk::Result> {
    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SemaphoreCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
    };
    device.create_semaphore(&semaphore_create_info, None)
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
