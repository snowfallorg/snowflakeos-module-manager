use crate::modules::{
    self,
    load::{getcurrentoptions, getmodulepath, loadmoduleconfig},
    ModuleOption, Module,
};
use anyhow::Result;
use nix_data::config::configfile::NixDataConfig;
use std::{collections::HashMap, path::PathBuf};

pub struct LoadOutput {
    pub config: nix_data::config::configfile::NixDataConfig,
    pub moduleconfig: String,
    pub modulepath: PathBuf,
    pub flakepath: PathBuf,
    pub modules: Vec<modules::Module>,
    pub current_config: HashMap<String, ModuleOption>,
}

pub fn load() -> Result<LoadOutput> {
    let config = nix_data::config::configfile::getconfig().expect("Failed to load config");
    let moduleconfig = loadmoduleconfig(&config).expect("Failed to load module config");
    let modulepath = getmodulepath(&config).expect("Failed to get module path");
    let flakepath = config
        .flake
        .as_ref()
        .map(PathBuf::from)
        .and_then(|x| x.parent().map(|x| x.to_path_buf()))
        .expect("Failed to get flake path");
    let modules = modules::load::loadmodules(&flakepath);
    let current_config =
        getcurrentoptions(&config, &modules).expect("Failed to load current module configuration");
    Ok(LoadOutput {
        config,
        moduleconfig,
        modulepath,
        flakepath,
        modules,
        current_config,
    })
}

pub struct ReloadOutput {
    pub current_config: HashMap<String, ModuleOption>,
    pub moduleconfig: String,
    pub modules: Vec<Module>,
}

pub fn reload(config: &NixDataConfig) -> Result<ReloadOutput> {
    let flakepath = config
        .flake
        .as_ref()
        .map(PathBuf::from)
        .and_then(|x| x.parent().map(|x| x.to_path_buf()))
        .expect("Failed to get flake path");
    let modules = modules::load::loadmodules(&flakepath);
    let current_config = getcurrentoptions(config, &modules)
        .expect("Failed to load current module configuration");
    let moduleconfig = loadmoduleconfig(config).expect("Failed to load module config");
    Ok(
        ReloadOutput {
            current_config,
            moduleconfig,
            modules,
        }
    )
}
