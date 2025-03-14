//!
//! Configuration
//!

// registry settings
pub const REGISTRY_CACHE_DISABLE: bool = false;
pub const REGISTRY_CACHE_FILENAME: &str = ".plugin_cache.toml";
pub const VST_DEFAULT_DIR: &str = "C:/Program Files/Common Files/VST3";
pub const VST_DIRS: &[&str] = &[
    "D:/Work/vsthacks/megasynth/build/VST3/Debug/megasynth.vst3/Contents/x86_64-win",
    "C:/Tools/sdk/VST_SDK/vst3sdk/build/VST3/Debug/host-checker.vst3/Contents/x86_64-win",
    VST_DEFAULT_DIR
];

// choosing the ASIO device
pub const DEVICE_ASIO4ALL: &str = "ASIO4ALL v2";
pub const DEVICE_YAMAHA: &str = "Yamaha Steinberg USB ASIO";
pub const DEVICE_UMC: &str = "UMC ASIO Driver";
pub const DEVICE_REALTEK: &str = "Realtek ASIO";

// general ASIO output
pub const ASIO_DEVICE_NAME: &str = DEVICE_ASIO4ALL;
pub const ASIO_BUFFER_SIZE: usize = 0; // 0 to use default
pub const ASIO_SAMPLE_RATE: f64 = 44100.0;

// choosing the VST plugin
pub const FM8_CLASS_ID: &str = "4E545356666966386D38000000000000"; // FM8
pub const MEGASYNTH_CLASS_ID: &str = "3C2A31A836CE1F5088C9096932F9A0EA"; // MEGASYNTH
pub const ANALOG_LAB: &str = "7574724156415349416C617650726F63"; // Analog Lab
pub const HOST_CHECKER: &str = "0E19FC23DD029944A8D2230E50617DA3"; // Host Checker
pub const OTHER_CLASS_ID: &str = "4E5453564B696B386F6E74616B742038"; // some thing...
pub const VST_CLSID: &str = FM8_CLASS_ID;

// debugging settings
pub const ENABLE_COMPONENT_HANDLER: bool = false;
pub const ENABLE_VIEW_RESIZE: bool = true;
