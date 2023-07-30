pub mod changes_factory;
pub mod confirm_dialog;
pub mod rebuild_dialog;

#[derive(Debug)]
pub struct OptionModification {
    pub label: String,
    pub mod_type: ModificationType,
}

#[derive(Debug)]
pub enum ModificationType {
    New { value: String },
    Update { old: String, new: String },
}

impl ModificationType {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            ModificationType::New { value } => value.to_string(),
            ModificationType::Update { old, new } => format!("{} → {}", old, new),
        }
    }
}
