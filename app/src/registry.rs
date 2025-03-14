use std::{collections::HashMap, fs::{self, File}, io::{Read, Write}, sync::Arc};

use chrono::Utc;
use vst3_com::sys::GUID;
use log::{*};

use crate::{config::{REGISTRY_CACHE_FILENAME, VST_DIRS}, error::Error, instance::Instance, plugin::{get_identifier_from_class, get_identifier_from_path, ClassInfo, Plugin, PluginInfo}, utils::get_file_time};

pub struct ClassMapEntry {
    plugin_info: PluginInfo,
    class_info: ClassInfo
}

pub struct PluginRef {
    plugin: Arc<Plugin>,
    ref_counter: usize
}

pub struct Registry {
    cache: HashMap<String, PluginInfo>,
    loaded_plugins: HashMap<String, PluginRef>,
    class_map: HashMap<String, ClassMapEntry>
}

impl Registry {

    pub fn new() -> Self {
        trace!("new");
        Self {
            cache: HashMap::new(),
            loaded_plugins: HashMap::new(),
            class_map: HashMap::new()
        }
    }

    pub fn dispose(&mut self) {
        self.cache.clear();
        self.loaded_plugins.clear();
        self.class_map.clear();
    }

    pub fn init(&mut self) -> Result<(), Error> {
        trace!("init");

        self.update_cache()?;

        for plugin_info in self.cache.values() {
            for class_info in &plugin_info.classes {
                let entry = ClassMapEntry {
                    plugin_info: plugin_info.clone(),
                    class_info: class_info.clone()
                };

                let uid = class_info.cid.to_string();
                self.class_map.insert(uid, entry);
            }
        }

        Ok(())
    }

    fn ref_plugin(&mut self, plugin_id: &str) -> Result<Arc<Plugin>, Error> {

        let plugin_ref = self.loaded_plugins.get_mut(plugin_id);
        if plugin_ref.is_none() {

            let plugin_info = match self.cache.get(plugin_id) {
                Some(plugin_info) => plugin_info,
                None => {
                    return Err(Error::from("invalid plugin identifier"));
                }
            };

            let plugin = Arc::new(Plugin::new(&plugin_info.path)?);

            let plugin_ref = PluginRef {
                plugin: plugin.clone(),
                ref_counter: 1
            };

            self.loaded_plugins.insert(plugin_id.to_string(), plugin_ref);

            Ok(plugin)
        } else {
            Ok(plugin_ref.unwrap().plugin.clone())
        }

    }

    fn unref_plugin(&mut self, plugin_id: &str) {
        match self.loaded_plugins.get_mut(plugin_id) {
            Some(plugin_ref) => {
                if plugin_ref.ref_counter > 1 {
                    plugin_ref.ref_counter -= 1;
                } else {
                    match self.loaded_plugins.remove(plugin_id) {
                        Some(mut removed_plugin) => {
                            match Arc::get_mut(&mut removed_plugin.plugin) {
                                Some(plugin) => {
                                    plugin.dispose();
                                },
                                None => {}
                            };
                        },
                        None => {}
                    }
                }
            },
            None => {}
        }
    }

    fn get_class(&self, class_id: &str) -> Result<&ClassMapEntry, Error> {
        match self.class_map.get(class_id) {
            Some(entry) => Ok(entry),
            None => {
                Err(Error::from("failed to create class instance"))
            }
        }
    }

    pub fn create_class_instance(&mut self, class_id: &str) -> Result<Instance, Error> {

        trace!("create class instance");

        let plugin_id;
        let class_guid: GUID;

        {
            let entry = match self.get_class(class_id) {
                Ok(entry) => entry,
                Err(e) => {
                    return Err(e);
                }
            };

            plugin_id = entry.plugin_info.id.clone();
            class_guid = entry.class_info.cid.clone();
        }

        let plugin = self.ref_plugin(&plugin_id)?;

        let instance = match plugin.create_class_instance(&class_guid) {
            Ok(instance) => instance,
            Err(e) => {
                self.unref_plugin(&plugin_id);
                return Err(e);
            }
        };

        Instance::new(&plugin, instance, class_id)

    }

    pub fn unref_class_instance(&mut self, instance: Instance) -> Result<(), Error> {
        trace!("unref class instance");

        crate::utils::trace_ref(&instance.instance);

        let plugin_id;

        {
            let entry = match self.get_class(instance.class_id()) {
                Ok(entry) => entry,
                Err(e) => {
                    return Err(e);
                }
            };

            plugin_id = entry.plugin_info.id.clone();
        }

        crate::utils::trace_ref(&instance.instance);

        trace!("drop instance");
        drop(instance);

        trace!("unref plugin");
        self.unref_plugin(&plugin_id);

        Ok(())
    }

    fn update_cache(&mut self) -> Result<(), Error> {

        trace!("update cache");

        let mut cache = Self::read_cache().unwrap_or(HashMap::<String, PluginInfo>::new());

        if crate::config::REGISTRY_CACHE_DISABLE {
            self.cache = cache;
            return Ok(())
        }

        let mut new_cache = HashMap::<String, PluginInfo>::new();

        let path_list = Self::find_libraries()?;

        let mut dirty = false;

        for filename in &path_list {

            let file_time = get_file_time(filename);
            if 0 == file_time {
                continue; // failed to access file
            }

            let identifier = get_identifier_from_path(filename);

            let opt_plugin_info = cache.remove(&identifier);
            if opt_plugin_info.is_some() && opt_plugin_info.as_ref().unwrap().file_time == file_time {
                // no change, just move entry to new cache
                new_cache.insert(identifier, opt_plugin_info.unwrap());
            } else {
                dirty = true; // from now on, cache must be updated
                match Plugin::new(filename) {
                    Ok(mut plugin) => {
                        let plugin_info = plugin.get_info().clone();
                        new_cache.insert(identifier, plugin_info);
                        plugin.dispose();
                    },
                    Err(_) => {
                        // failed to load plugin, ignore, no update to new cache
                    }
                };
            }

            // keep updating cache while processing
            if dirty {
                let _ = Self::write_cache(&new_cache);
                dirty = false;
            }
        }

        if !cache.is_empty() {
            dirty = true;
        }

        if dirty {
            let _ = Self::write_cache(&new_cache);
        }

        self.cache = new_cache;

        Ok(())
    }

    fn read_cache() -> Result<HashMap<String, PluginInfo>, Error> {
        let mut file = match File::open(REGISTRY_CACHE_FILENAME) {
            Ok(f) => f,
            Err(_) => { return Err(Error::from("failed to open plugin cache")); }
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Ok(_) => {},
            Err(_) => {
                return Err(Error::from("failed to open plugin cache"));
            }
        }

        let cache: toml::Table = toml::from_str(&s).unwrap();
        match cache.get("cache") {
            Some(_) => {},
            None => { return Err(Error::from("incomplete plugin cache")); }
        };

        let mut plugins = HashMap::<String, PluginInfo>::new();

        for cache_section in &cache {
            let section_name = cache_section.0;
            if section_name == "cache" {
                continue; // skip
            }
            let cached_plugin_id = cache_section.0;
            let cached_plugin_info = cache_section.1;
            if cached_plugin_info.is_table() {
                let plugin_attributes = cached_plugin_info.as_table().unwrap();

                let mut plugin_info = PluginInfo::default();
                plugin_info.id = cached_plugin_id.clone();

                for attribute in plugin_attributes {
                    let key = attribute.0;
                    let value = attribute.1;

                    if key == "path" {
                        plugin_info.path = String::from(value.as_str().unwrap());
                        //trace!("class name: {}", &plugin_info.path);
                    } else if key == "file_time" {
                        plugin_info.file_time = value.as_integer().unwrap() as i64;
                    } else if value.is_table() {
                        let info = value.as_table().unwrap();
                        let name = info.get("name").unwrap().as_str().unwrap();
                        let category = info.get("category").unwrap().as_str().unwrap();
                        let cardinality = info.get("cardinality").unwrap().as_integer().unwrap() as i32;
                        let cid_str = info.get("cid").unwrap().as_str().unwrap();
                        let cid = GUID::from_string(cid_str);

                        /*
                        trace!(
                            "class: name=\"{}\", category=\"{}\", cardinality={}, cid=\"{}\"",
                            name, category, cardinality, cid_str
                        );
                        */

                        let class_info = ClassInfo {
                            name: name.to_string(),
                            category: category.to_string(),
                            cardinality,
                            cid
                        };

                        plugin_info.classes.push(class_info);
                    }
                }

                //plugins.push(plugin_info);
                plugins.insert(cached_plugin_id.clone(), plugin_info);

            }
        }

        Ok(plugins)
    }

    fn write_cache(plugins: &HashMap<String, PluginInfo>) -> Result<(), Error> {

        let timestamp = Utc::now();
        let timestamp_str = timestamp.to_string();

        let mut file = match File::create(REGISTRY_CACHE_FILENAME) {
            Ok(f) => f,
            Err(_) => { return Err(Error::from("failed to create plugin cache")); }
        };

        match file.write_all(format!("[cache]\ntimestamp = \"{}\"\n\n", timestamp_str).as_bytes()) {
            Ok(_) => {},
            Err(_) => { return Err(Error::from("failed to create plugin cache")); }
        }

        for (plugin_identifier, plugin) in plugins.iter() {

            //trace!("plugin: {}", plugin.path);
            let plugin_path = &plugin.path;

            match file.write_all(format!(
                "[{}]\npath = \"{}\"\nfile_time = {}\n",
                plugin_identifier,
                plugin_path,
                plugin.file_time
            ).as_bytes()) {
                Ok(_) => {},
                Err(_) => { return Err(Error::from("failed to create plugin cache")); }
            }

            for class_info in &plugin.classes {
                match file.write_all(
                    format!("{} = {{ name=\"{}\", category=\"{}\", cardinality={}, cid=\"{}\" }}\n",
                    get_identifier_from_class(class_info, Some("class")),
                    class_info.name,
                    class_info.category,
                    class_info.cardinality,
                    class_info.cid.to_string()
                ).as_bytes()) {
                    Ok(_) => {},
                    Err(_) => { return Err(Error::from("failed to create plugin cache")); }
                }
            }

            let _ = file.write_all("\n".as_bytes());
        }

        let _ = file.sync_all();

        Ok(())
    }

    pub fn find_libraries() -> Result<Vec<String>, Error> {
        let mut path_list = Vec::<String>::new();

        for vst_dir in VST_DIRS {
            let _ = Self::find_libraries_in_path(vst_dir, &mut path_list);
        }

        Ok(path_list)
    }

    fn find_libraries_in_path(path: &str, path_list: &mut Vec<String>) -> Result<(), Error> {
        let paths = match fs::read_dir(path) {
            Ok(paths) => paths,
            Err(_) => {
                return Err(Error::from("failed to read directory contents"));
            }
        };

        for path in paths {
            match path {
                Ok(dir_entry) => {
                    let path = dir_entry.path();
                    match path.extension() {
                        Some(ext) => {
                            if ext.to_ascii_lowercase() == "vst3" {
                                let vst_path = path.to_str().unwrap().to_string();
                                path_list.push(vst_path)
                            }
                        },
                        None => {}
                    }
                }
                Err(_) => {}
            }
        }

        Ok(())
    }

}

trait GuidStringify {
    fn to_string(&self) -> String;
    fn from_string(s: &str) -> GUID;
}

impl GuidStringify for GUID {

    fn to_string(&self) -> String {
        self.data.iter().map(|n| format!("{:02X?}", n) ).collect::<Vec<String>>().join("")
    }

    fn from_string(s: &str) -> Self {

        let mut data: [u8; 16] = [0; 16];

        if s.len() > 0 && s.len() <= 32 {
            let mut low_nibble = false;
            let mut byte_index = 0;
            let mut accumulator: u8 = 0x0;
            for c in s.chars() {

                accumulator <<= 4;
                accumulator += c.to_digit(16).unwrap_or(0) as u8;

                if low_nibble {
                    data[byte_index] = accumulator;
                    byte_index += 1;
                    accumulator = 0x0;
                }
                low_nibble = !low_nibble;
            }
        }

        Self {
            data
        }

    }

}
