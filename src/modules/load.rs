use anyhow::{Context, Result};
use log::debug;
use nix_data::config::configfile::NixDataConfig;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::modules::OptionType;

use super::{Module, ModuleOption};

pub fn loadmodules(basedir: &Path) -> Vec<Module> {
    // Iterate over all directories and subdirectories in the `basedir/modules` directory
    // and return a vector of `Module`s based on finding a `default.nix` file in the directory.

    let mut modules: Vec<Module> = Vec::new();
    let modulepath = basedir.join("modules");

    for entry in walkdir::WalkDir::new(modulepath).into_iter().flatten() {
        let path = entry.path();
        if path.is_dir() {
            let defaultnix = path.join("default.nix");
            if defaultnix.exists() {
                let moduleconfig = path.join("module.yml");
                let config = moduleconfig
                    .exists()
                    .then(|| {
                        fs::read_to_string(&moduleconfig)
                            .ok()
                            .and_then(|config_str| serde_yaml::from_str(&config_str).unwrap())
                    })
                    .flatten();
                debug!("Loading config: {:#?}", config);

                if let Some(config) = config {
                    let module = Module {
                        name: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        path: path.to_path_buf(),
                        config,
                    };
                    modules.push(module);
                }
            }
        }
    }
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    modules
}

/**
 * Load current module configuration in `modules.nix`, located next to the `default.nix` file.
 */
pub fn loadmoduleconfig(config: &NixDataConfig) -> Result<String> {
    let modulesnix = Path::new(&config.systemconfig.clone().context("systemconfig")?)
        .parent()
        .context("systemconfig parent")?
        .join("modules.nix");
    fs::read_to_string(modulesnix).context("modules.nix")
}

pub fn getmodulepath(config: &NixDataConfig) -> Result<PathBuf> {
    let modulesnix = Path::new(&config.systemconfig.clone().context("systemconfig")?)
        .parent()
        .context("systemconfig parent")?
        .join("modules.nix");
    Ok(modulesnix)
}

/**
 * Get the currently set module options
 * Should be a HashMap<String, String>?
 */
pub fn getcurrentoptions(
    config: &NixDataConfig,
    modules: &[Module],
) -> Result<HashMap<String, ModuleOption>> {
    let modulesnix = Path::new(&config.systemconfig.clone().context("systemconfig")?)
        .parent()
        .context("systemconfig parent")?
        .join("modules.nix");
    let moduletext = fs::read_to_string(modulesnix).context("modules.nix")?;

    let options = modules
        .iter()
        .map(|x| x.config.options.clone())
        .collect::<Vec<_>>()
        .concat();

    let mut output = HashMap::new();
    for option in options {
        let attribute = option.id;
        let string_value = nix_editor::read::readvalue(&moduletext, &attribute);

        if let Ok(string_value) = string_value {
            match option.op_type {
                OptionType::Switch { .. } => {
                    let value = match string_value.as_str() {
                        "true" => true,
                        "false" => false,
                        _ => continue,
                    };
                    output.insert(attribute, ModuleOption::Switch { value });
                }
                OptionType::Text { .. } => {
                    let value = string_value
                        .strip_prefix('"')
                        .and_then(|x| x.strip_suffix('"'));
                    if let Some(value) = value {
                        output.insert(
                            attribute,
                            ModuleOption::Text {
                                value: value.to_string(),
                            },
                        );
                    }
                }
                OptionType::Enum { .. } => {
                    output.insert(
                        attribute,
                        ModuleOption::Text {
                            value: string_value.to_string(),
                        },
                    );
                }
                OptionType::NumberList { .. } => {
                    if let Ok(arr) = nix_editor::read::getarrvals(&moduletext, &attribute) {
                        let numbers = arr
                            .iter()
                            .filter_map(|x| x.parse::<u32>().ok())
                            .collect::<Vec<_>>();
                        output.insert(attribute, ModuleOption::NumberList { value: numbers });
                    }
                }
            }
        }
    }
    Ok(output)
}
