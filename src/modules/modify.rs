use std::fs;

use crate::{MODULES_CONFIG, modules::OptionType};

use super::{Module, OptionData};
use anyhow::{Result, Context};

fn write_module_option(opt: &str, value: &str) -> Result<()> {
    let moduleconfig = fs::read_to_string(MODULES_CONFIG)?;
    let out = nixpkgs_fmt::reformat_string(&nix_editor::write::write(&moduleconfig, opt, value)?);
    fs::write(MODULES_CONFIG, out)?;
    Ok(())
}

fn deref_module_option(opt: &str) -> Result<()> {
    let moduleconfig = fs::read_to_string(MODULES_CONFIG)?;
    let out = nixpkgs_fmt::reformat_string(&nix_editor::write::deref(&moduleconfig, opt)?);
    fs::write(MODULES_CONFIG, out)?;
    Ok(())
}

impl Module {
    pub fn setoption(&self, option: &OptionData, value: &str) -> Result<()>{
        let config = &self.config;
        if !config.options.iter().any(|o| o.id == option.id) {
            anyhow::bail!("Option {} not found in module {}", option.id, self.name);
        }
        // Verification
        match &option.op_type {
            OptionType::Switch { .. } => {
                if value == "true" || value == "false" {
                    // Do nothing
                } else {
                    anyhow::bail!("Invalid value for switch option: {}", value);
                }
            },
            OptionType::Text { .. } => {
                if value.starts_with('"') && value.ends_with('"') {
                    // Do nothing
                } else {
                    anyhow::bail!("Invalid value for text option: {}", value);
                }
            },
            OptionType::Enum { options, .. } => {
                if !options.values().any(|x| x == value) {
                    anyhow::bail!("Invalid value for enum option: {}", value);
                }
            },
            OptionType::NumberList { .. } => {
                if value.starts_with('[') && value.ends_with(']') {
                    // Do nothing
                } else {
                    anyhow::bail!("Invalid value for numberlist option: {}", value);
                }
            }
        }
    
        write_module_option(&option.id, value)?;

        Ok(())
    }

    pub fn deref_option(&self, option: &OptionData) -> Result<()> {
        let config = &self.config;
        if !config.options.iter().any(|o| o.id == option.id) {
            anyhow::bail!("Option {} not found in module {}", option.id, self.name);
        }
        deref_module_option(&option.id)?;
        Ok(())
    }
    

    pub fn enable(&self, enable: bool) -> Result<()> {
        let config = &self.config;
        let options = &config.options;
        let enableoption = options.iter().find(|option| option.id.split('.').last() == Some("enable")).context("No enable option found")?;
        write_module_option(&enableoption.id, &enable.to_string())?;
        Ok(())  
    }

    pub fn remove(self) -> Result<()> {
        let config = &self.config;
        for option in &config.options {
            self.deref_option(option)?;
        }
        fs::remove_dir_all(self.path)?;
        Ok(())
    }
}
