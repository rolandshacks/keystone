//! FUnknown's integer error codes
use crate::base::tresult;
pub use vst3_com::interfaces::iunknown::IUnknown;

#[cfg(not(target_os = "windows"))]
pub const kNoInterface: tresult = -1;
#[cfg(not(target_os = "windows"))]
pub const kResultOk: tresult = 0;
#[cfg(not(target_os = "windows"))]
pub const kResultTrue: tresult = kResultOk;
#[cfg(not(target_os = "windows"))]
pub const kResultFalse: tresult = 1;
#[cfg(not(target_os = "windows"))]
pub const kInvalidArgument: tresult = 2;
#[cfg(not(target_os = "windows"))]
pub const kNotImplemented: tresult = 3;
#[cfg(not(target_os = "windows"))]
pub const kInternalError: tresult = 4;
#[cfg(not(target_os = "windows"))]
pub const kNotInitialized: tresult = 5;
#[cfg(not(target_os = "windows"))]
pub const kOutOfMemory: tresult = 6;

#[cfg(target_os = "windows")]
pub const kNoInterface: tresult = -2_147_467_262;
#[cfg(target_os = "windows")]
pub const kResultOk: tresult = 0;
#[cfg(target_os = "windows")]
pub const kResultTrue: tresult = kResultOk;
#[cfg(target_os = "windows")]
pub const kResultFalse: tresult = 1;
#[cfg(target_os = "windows")]
pub const kInvalidArgument: tresult = -2_147_024_809;
#[cfg(target_os = "windows")]
pub const kNotImplemented: tresult = -2_147_467_263;
#[cfg(target_os = "windows")]
pub const kInternalError: tresult = -2_147_467_259;
#[cfg(target_os = "windows")]
pub const kNotInitialized: tresult = -2_147_418_113;
#[cfg(target_os = "windows")]
pub const kOutOfMemory: tresult = -2_147_024_882;
