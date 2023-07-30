use std::{path::PathBuf, collections::HashMap};

use serde::{Deserialize, Serialize};

pub mod load;
pub mod modify;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub config: ModuleData,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModuleData {
    pub name: String,
    pub id: String,
    pub description: Option<String>,
    pub version: String,
    pub options: Vec<OptionData>,
    pub icon: Option<IconData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IconData {
    #[serde(rename = "type")]
    pub icon_type: IconType,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IconType {
    File,
    System,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OptionData {
    pub label: String,
    pub id: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub op_type: OptionType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OptionType {
    Switch {
        default: bool,
    },
    Text {
        default: String,
    },
    Enum {
        default: String,
        options: HashMap<String, String>,
    },
    NumberList {
        default: Vec<u32>,
    },
}

impl OptionType {
    pub fn is_switch(&self) -> bool {
        matches!(self, OptionType::Switch { .. })
    }
    pub fn is_text(&self) -> bool {
        matches!(self, OptionType::Text { .. })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigDetails {
    pub description: String,
    #[serde(rename = "type")]
    pub config_type: String,
    #[serde(default)]
    pub default: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleOption {
    Switch {
        value: bool
    },
    Text {
        value: String
    },
    Enum {
        value: String,
        pretty: String
    },
    NumberList {
        value: Vec<u32>
    }
}

impl ModuleOption {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            ModuleOption::Switch { value } => if *value { String::from("Enabled") } else { String::from("Disabled") },
            ModuleOption::Text { value } => format!("\"{}\"", value),
            ModuleOption::Enum { pretty, .. } => pretty.to_string(),
            ModuleOption::NumberList { value } => value.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
        }
    }
    pub fn value(&self) -> String {
        match self {
            ModuleOption::Switch { value } => value.to_string(),
            ModuleOption::Text { value } => format!("\"{}\"", value),
            ModuleOption::Enum { value, .. } => value.to_string(),
            ModuleOption::NumberList { value } => format!("[ {} ]", value.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
        }
    }
}
