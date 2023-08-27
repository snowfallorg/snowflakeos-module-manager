use anyhow::{Context, Result};
use log::debug;
use nix_data::config::configfile::NixDataConfig;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::modules::{ModuleData, OptionType};

use super::{Module, ModuleOption};

pub fn loadmodules(flakepath: &Path) -> Result<Vec<Module>> {
    // Iterate over all directories and subdirectories in the `basedir/modules` directory
    // and return a vector of `Module`s based on finding a `default.nix` file in the directory.

    let mut modules: Vec<Module> = Vec::new();
    let modulepath = Path::new("/etc/snowflakeos-modules");

    let flakefile = fs::read_to_string(flakepath)?;
    let installed_modules =
        nix_editor::read::getarrvals(&flakefile, "outputs.systems.modules.nixos")?;

    for entry in walkdir::WalkDir::new(modulepath).into_iter().flatten() {
        let path = entry.path();
        if path.is_dir() {
            let moduleconfig = path.join("module.yml");
            if moduleconfig.exists() {
                // Path from /etc/snowflakeos-modules joined by '/'
                let mut moduleid = path
                    .strip_prefix(modulepath)
                    .unwrap_or(path)
                    .iter()
                    .map(|x| x.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                if moduleid.contains("/") {
                    moduleid = format!("\"{}\"", moduleid);
                }

                if let Some(config_text) = fs::read_to_string(&moduleconfig)
                    .ok()
                    .and_then(|config_str| serde_yaml::from_str(&config_str).ok())
                {
                    let config: Option<ModuleData> =
                        moduleconfig.exists().then(|| config_text).flatten();
                    debug!("Loading config: {:#?}", config);

                    if let Some(config) = config {
                        let module = Module {
                            name: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            path: path.to_path_buf(),
                            config: config.clone(),
                        };
                        if installed_modules
                            .contains(&format!("{}.nixosModules.{}", config.flake, moduleid))
                        {
                            modules.push(module);
                        }
                    }
                }
            }
        }
    }
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(modules)
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
                OptionType::Enum { options, .. } => {
                    output.insert(
                        attribute,
                        ModuleOption::Enum {
                            value: string_value.to_string(),
                            pretty: if let Some(pretty) = options.get(&string_value) {
                                pretty.to_string()
                            } else {
                                string_value.to_string()
                            },
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
