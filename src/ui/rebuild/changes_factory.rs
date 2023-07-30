use super::{confirm_dialog::ConfirmDialogInput, ModificationType, OptionModification};
use adw::traits::{ActionRowExt, PreferencesGroupExt, PreferencesRowExt};
use relm4::{
    factory::{FactoryVecDeque, FactoryView},
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender,
};

pub struct ModuleChangesModel {
    modificationfactory: FactoryVecDeque<OptionChangesModel>,
    label: String,
}

#[derive(Debug)]
pub enum ModuleChangesInput {}

#[derive(Debug)]
pub enum ModuleChangesOutput {}

pub struct ModuleChangesInit {
    pub modifications: Vec<OptionModification>,
    pub label: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleChangesModel {
    type ParentWidget = gtk::Box;
    type ParentInput = ConfirmDialogInput;
    type Input = ModuleChangesInput;
    type Output = ModuleChangesOutput;
    type Init = ModuleChangesInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            #[local_ref]
            preferences_group -> adw::PreferencesGroup {
                set_title: &self.label,
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut modificationfactory =
            FactoryVecDeque::new(adw::PreferencesGroup::new(), sender.input_sender());
        let mut modificationfactory_guard = modificationfactory.guard();
        for change in init.modifications {
            modificationfactory_guard.push_back(OptionChangesInit {
                modification: change,
            });
        }
        modificationfactory_guard.drop();
        Self {
            modificationfactory,
            label: init.label,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let preferences_group = self.modificationfactory.widget();
        let widgets = view_output!();
        widgets
    }
}

pub struct OptionChangesModel {
    label: String,
    mod_type: ModificationType,
}

#[derive(Debug)]
pub enum OptionChangesInput {}

#[derive(Debug)]
pub enum OptionChangesOutput {}

pub struct OptionChangesInit {
    pub modification: OptionModification,
}

#[relm4::factory(pub)]
impl FactoryComponent for OptionChangesModel {
    type ParentWidget = adw::PreferencesGroup;
    type ParentInput = ModuleChangesInput;
    type Input = OptionChangesInput;
    type Output = OptionChangesOutput;
    type Init = OptionChangesInit;
    type CommandOutput = ();

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.label,
            add_suffix = &gtk::Label {
                set_label: &self.mod_type.to_string()
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            label: init.modification.label,
            mod_type: init.modification.mod_type,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        widgets
    }
}
