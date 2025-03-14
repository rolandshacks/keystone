use std::{ffi::CStr, ptr::null_mut};

use libloading::Library;
use vst3_com::{interfaces::iunknown::IID_IUNKNOWN, sys::GUID, *};
use vst3_sys::base::*;
use log::{*};

use crate::{error::Error, utils::{get_file_time, slashify_path}};

const VST_CATEGORY_AUDIO_EFFECT: &str = "Audio Module Class";
const VST_CATEGORY_COMPONENT_CONTROLLER: &str = "Component Controller Class";
const VST_CATEGORY_PLUGIN_COMPATIBILITY: &str = "Plugin Compatibility Class";

type FnInitDll = extern "system" fn() -> bool;
type FnExitDll = extern "system" fn() -> bool;
type FnGetPluginFactory = extern "system" fn() -> *mut *mut IPluginFactoryVTable;

#[derive(Clone, Debug)]
pub struct ClassInfo {
    pub name: String,
    pub category: String,
    pub cardinality: i32,
    pub cid: GUID
}

impl ClassInfo {
    pub fn from(ci: PClassInfo) -> Self {
        let category = unsafe { CStr::from_ptr(ci.category.as_ptr()).to_string_lossy().to_string() };
        let name = unsafe { CStr::from_ptr(ci.name.as_ptr()).to_string_lossy().to_string() };
        Self {
            name,
            category,
            cardinality: ci.cardinality,
            cid: ci.cid
        }
    }
}

#[derive(Clone, Debug)]
pub struct PluginInfo {
    pub id: String,
    pub path: String,
    pub classes: Vec<ClassInfo>,
    pub file_time: i64
}

impl Default for PluginInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            path: String::new(),
            classes: Vec::new(),
            file_time: 0
        }
    }
}

pub struct Plugin {
    info: PluginInfo,
    pub lib: Option<libloading::Library>,
    exit_fn: Option<FnExitDll>,
    factory: RawVstPtr<dyn IPluginFactory>,
}

impl Drop for Plugin {
    fn drop(&mut self) {
        trace!("drop");
    }
}

impl Plugin {

    pub fn new(filename: &str) -> Result<Self, Error> {
        trace!("new");

        let file_time = get_file_time(filename);

        let lib = match unsafe { libloading::Library::new(filename) } {
            Ok(lib) => lib,
            Err(_e) => {
                return Err(Error::from("failed to load plugin library: could not load dynamic library"))
            }
        };

        match unsafe { lib.get::<FnInitDll>(b"InitDll") } {
            Ok(init_fn) => {
                let result = init_fn();
                if !result {
                    return Err(Error::from("failed to initialize plugin"));
                }
            },
            Err(_e) => {}
        };

        let exit_fn = match unsafe { lib.get::<FnExitDll>(b"ExitDll") } {
            Ok(exit_fn) => Some(*exit_fn),
            Err(_e) => None
        };

        let factory = Self::get_factory(&lib)?;
        let classes = Self::get_classes(&factory)?;

        let plugin_info = PluginInfo {
            id: get_identifier_from_path(filename),
            path: slashify_path(filename),
            classes,
            file_time
        };

        Ok(Self {
            info: plugin_info,
            lib: Some(lib),
            exit_fn,
            factory
        })

    }

    pub fn dispose(&mut self) {
        trace!("dispose");

        match self.lib.take() {
            Some(lib) => {
                unsafe { self.factory.release(); }

                if self.exit_fn.is_some() {
                    self.exit_fn.unwrap()();
                }

                let _ = lib.close();
            },
            None => {}
        };
    }

    pub fn get_info(&self) -> &PluginInfo {
        &self.info
    }

    fn get_factory(lib: &Library) -> Result<RawVstPtr<dyn IPluginFactory>, Error> {

        let factory = match unsafe { lib.get::<FnGetPluginFactory>(b"GetPluginFactory") } {
            Ok(fn_get_plugin_factory) => {
                let factory_ppvtable = fn_get_plugin_factory();
                if factory_ppvtable.is_null() {
                    return Err(Error::from("no plugin factory"));
                }

                match unsafe { RawVstPtr::<dyn IPluginFactory>::new(factory_ppvtable) } {
                    Some(plugin_factory) => plugin_factory,
                    None => {
                        return Err(Error::from("no plugin factory"));
                    }
                }

            },
            Err(_e) => {
                return Err(Error::from("failed to load plugin library: incompatible plugin interface"))
            }
        };

        Ok(factory)
    }

    fn get_classes(plugin_factory: &RawVstPtr<dyn IPluginFactory>) -> Result<Vec<ClassInfo>, Error> {

        trace!("get classes");

        let mut classes =  Vec::<ClassInfo>::new();

        unsafe {

            let class_count = plugin_factory.count_classes();

            for i in 0..class_count {
                let mut p_class_info = PClassInfo {
                    cid: GUID { data: [0u8; 16] },
                    cardinality: 0,
                    category: [0; 32],
                    name: [0; 64]
                };

                let result = plugin_factory.get_class_info(i, &mut p_class_info);
                if result != kResultOk {
                    continue;
                }

                let class_info = ClassInfo::from(p_class_info);

                trace!(" - class info: name: \"{}\", category: \"{}\", cardinality: {:#x}", class_info.name, class_info.category, class_info.cardinality);

                classes.push(class_info);
            }

        }

        Ok(classes)

    }

    fn get_class_info(&self, name: &str) -> Option<&ClassInfo> {

        trace!("get class info");

        for class_info in &self.info.classes {
            if class_info.name == name {
                return Some(class_info);
            }
        }

        return None
    }

    pub fn create_class_instance_by_name(&self, name: &str) -> Result<VstPtr<dyn IUnknown>, Error> {

        trace!("create class instance by name");

        let class_info = match self.get_class_info(name) {
            Some(class_info) => class_info,
            None => {
                return Err(Error::from("failed to create instance"));
            }
        };

        self.create_class_instance(&class_info.cid)
    }

    pub fn create_class_instance(&self, cid: &GUID) -> Result<VstPtr<dyn IUnknown>, Error> {

        trace!("create class instance");

        let mut instance_ptr: *mut c_void = null_mut();
        unsafe {
            let result = self.factory.create_instance(cid, &IID_IUNKNOWN,  &mut instance_ptr);
            if result != kResultOk {
                return Err(Error::from("failed to create instance"));
            }

            // owned -> no auto-release
            let instance = match VstPtr::<dyn IUnknown>::owned(instance_ptr as *mut _) {
                Some(instance) => {
                    instance
                },
                None => {
                    return Err(Error::from("failed to create instance"));
                }
            };

            crate::utils::trace_ref(&instance);

            Ok(instance)
        }
    }

}

pub fn get_identifier_from_class(class_info: &ClassInfo, prefix: Option<&str>) -> String {
    let mut s = String::new();

    if prefix.is_some() {
        s.push_str(prefix.unwrap());
        s.push('_');
    }

    for c in class_info.name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else {
            s.push('_');
        }
    }

    if class_info.category == VST_CATEGORY_AUDIO_EFFECT {
        // nothing
    } else if class_info.category == VST_CATEGORY_COMPONENT_CONTROLLER {
        s.push_str("_controller");
    } else if class_info.category == VST_CATEGORY_PLUGIN_COMPATIBILITY {
        s.push_str("_compat");
    } else {
        s.push_str("_unknown");
    }

    s
}


pub fn get_identifier_from_path(path: &str) -> String {
    let mut s = String::new();

    let mut last_ignored = false;

    for c in path.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
            last_ignored = false;
        } else {
            if !last_ignored {
                s.push('_');
            }
            last_ignored = true;
        }
    }

    s
}
