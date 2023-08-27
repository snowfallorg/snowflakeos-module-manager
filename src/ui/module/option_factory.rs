use std::collections::HashMap;

use adw::prelude::*;
use log::debug;
use relm4::{
    factory::{FactoryVecDeque, FactoryView},
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    view, FactorySender,
};

use crate::modules::{ModuleOption, OptionData, OptionType};

use super::{
    list_option_factory::{ListOptionInit, ListOptionModel},
    page::ModulePageInput,
};

#[tracker::track]
pub struct ModuleOptionModel {
    data: OptionData,
    value: Option<ModuleOption>,

    label: String,
    subtitle: String,

    enumoptions: Option<HashMap<u32, ModuleOption>>,
    #[tracker::no_eq]
    list_option_factory: FactoryVecDeque<ListOptionModel>,
    entryerror: bool,
    entryinput: String,
}

#[derive(Debug)]
pub enum ModuleOptionInput {
    SetEnumOptions(HashMap<u32, ModuleOption>),
    SelectOption(u32),
    SetEntryError(bool),
    SetEntryInput(String),
    AddExpanderOption,
    RemoveExpanderOption(String, DynamicIndex),
}

#[derive(Debug)]
pub enum ModuleOptionOutput {
    SetOption(String, ModuleOption),
}

#[derive(Debug)]
pub struct ModuleOptionInit {
    pub data: OptionData,
    pub value: Option<ModuleOption>,
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleOptionModel {
    type ParentWidget = gtk::Box;
    type ParentInput = ModulePageInput;
    type Input = ModuleOptionInput;
    type Output = ModuleOptionOutput;
    type Init = ModuleOptionInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            #[local_ref]
            prefgroup -> adw::PreferencesGroup {
                set_hexpand: true,
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut list_option_factory =
            FactoryVecDeque::new(adw::ExpanderRow::new(), sender.input_sender());
        if let OptionType::NumberList { default } = &init.data.op_type {
            let mut list_option_factory_guard = list_option_factory.guard();
            if let Some(ModuleOption::NumberList { value }) = &init.value {
                for number in value {
                    list_option_factory_guard.push_back(ListOptionInit {
                        value: number.to_string(),
                    });
                }
            } else {
                for number in default {
                    list_option_factory_guard.push_back(ListOptionInit {
                        value: number.to_string(),
                    });
                }
            }
            list_option_factory_guard.drop();
        }
        Self {
            data: init.data.clone(),
            value: init.value,
            label: init.data.label.to_string(),
            subtitle: init.data.description.clone().unwrap_or_default(),
            enumoptions: None,
            list_option_factory,
            entryerror: false,
            entryinput: String::new(),
            tracker: 0,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let list_option_exapnder = self.list_option_factory.widget();

        let prefgroup = adw::PreferencesGroup::new();

        match &self.data.op_type {
            OptionType::Switch { default } => {
                view! {
                    switch = adw::ActionRow {
                        set_hexpand: true,
                        set_title: &self.label,
                        #[watch]
                        set_subtitle: &self.subtitle,
                        add_suffix = & gtk::Switch {
                            set_halign: gtk::Align::End,
                            set_valign: gtk::Align::Center,
                            set_active:  {
                                if let Some(ModuleOption::Switch { value }) = self.value {
                                    value
                                } else {
                                    *default
                                }
                            },
                            connect_state_set[sender, id = self.data.id.to_string()] => move |_, value| {
                                sender.output(ModuleOptionOutput::SetOption(id.to_string(), ModuleOption::Switch { value }));
                                gtk::Inhibit(false)
                            }
                        }
                    }
                };
                prefgroup.add(&switch);
            }
            OptionType::Text { default } => {
                view! {
                    text = adw::ActionRow {
                        set_hexpand: true,
                        set_title: &self.label,
                        #[watch]
                        set_subtitle: &self.subtitle,
                        add_suffix = &gtk::Entry {
                            set_halign: gtk::Align::End,
                            set_valign: gtk::Align::Center,
                            set_text: {
                                if let Some(ModuleOption::Text { value }) = &self.value {
                                    value
                                } else {
                                    default
                                }
                            },
                            connect_changed[sender, id = self.data.id.to_string()] => move |x| {
                                sender.output(ModuleOptionOutput::SetOption(id.to_string(), ModuleOption::Text { value: x.text().to_string() }));
                            }
                        }
                    }
                };
                prefgroup.add(&text);
            }
            OptionType::Enum { options, default } => {
                view! {
                    string_enum = adw::ComboRow {
                        set_title: &self.label,
                        #[watch]
                        set_subtitle: &self.subtitle,
                        set_model: ({
                            let mut model = vec![];
                            for value in options.values() {
                                model.push(value.as_str());
                            }
                            model.sort();
                            let mut map = HashMap::new();
                            for (key, value) in options {
                                if let Some(index) = model.iter().position(|x| x == value) {
                                    map.insert(index as u32, ModuleOption::Enum { value: key.to_string(), pretty: value.to_string() });
                                }
                            }
                            sender.input(ModuleOptionInput::SetEnumOptions(map));
                            Some(gtk::StringList::new(&model))
                        }).as_ref(),
                        set_selected: {
                            let mut model = vec![];
                            for (k, v) in options {
                                model.push((k.as_str(), v.as_str()));
                            }
                            model.sort_by(|a, b| a.1.cmp(b.1));
                            if let Some(ModuleOption::Enum { value, .. }) = &self.value {
                                model.iter().position(|(k, _)| k == value).map(|x| x as u32).unwrap_or(0)
                            } else {
                                model.iter().position(|(k, _)| k == default).map(|x| x as u32).unwrap_or(0)
                            }
                        },
                        connect_selected_notify[sender] => move |row| {
                            sender.input(ModuleOptionInput::SelectOption(row.selected()));
                        }
                    }
                }
                prefgroup.add(&string_enum);
            }
            OptionType::NumberList { .. } => {
                view! {
                    #[local_ref]
                    list_option_exapnder -> adw::ExpanderRow {
                        set_title: &self.label,
                        #[watch]
                        set_subtitle: &self.subtitle,
                        set_expanded: true,
                        add_action = &gtk::Box {
                            add_css_class: "linked",
                            gtk::Entry {
                                set_valign: gtk::Align::Center,
                                #[track(self.changed(ModuleOptionModel::entryerror()))]
                                set_css_classes: if self.entryerror {
                                    &["error"]
                                } else {
                                    &[]
                                },
                                connect_changed[sender] => move |x| {
                                    let error = !x.text().is_empty() && x.text().parse::<u32>().is_err();
                                    sender.input(ModuleOptionInput::SetEntryError(error));
                                    if !error {
                                        sender.input(ModuleOptionInput::SetEntryInput(x.text().to_string()));
                                    }
                                }
                            },
                            gtk::Button {
                                set_icon_name: "list-add-symbolic",
                                set_valign: gtk::Align::Center,
                                connect_clicked[sender] => move |_| {
                                    sender.input(ModuleOptionInput::AddExpanderOption);
                                }
                            }
                        }
                    }
                }
                prefgroup.add(list_option_exapnder);
            }
        }

        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        self.reset();
        match message {
            ModuleOptionInput::SetEnumOptions(options) => {
                self.set_enumoptions(Some(options));
            }
            ModuleOptionInput::SelectOption(index) => {
                debug!("Selecting option {}", index);
                if let Some(options) = self.enumoptions.as_ref() {
                    if let Some(value) = options.get(&index) {
                        sender.output(ModuleOptionOutput::SetOption(
                            self.data.id.to_string(),
                            value.clone(),
                        ));
                    }
                }
            }
            ModuleOptionInput::SetEntryError(error) => {
                self.set_entryerror(error);
            }
            ModuleOptionInput::SetEntryInput(input) => {
                self.set_entryinput(input);
            }
            ModuleOptionInput::AddExpanderOption => {
                let mut list_option_factory_guard = self.list_option_factory.guard();
                let entry = &self.entryinput;
                if let Ok(number) = entry.parse::<u32>() {
                    if let Some(ModuleOption::NumberList { mut value }) = self.value.clone() {
                        if !value.contains(&number) {
                            value.push(entry.parse::<u32>().unwrap());
                            value.sort();
                            sender.output(ModuleOptionOutput::SetOption(
                                self.data.id.to_string(),
                                ModuleOption::NumberList {
                                    value: value.to_vec(),
                                },
                            ));
                            list_option_factory_guard.push_back(ListOptionInit {
                                value: entry.to_string(),
                            });
                            list_option_factory_guard.drop();
                            self.set_value(Some(ModuleOption::NumberList {
                                value: value.to_vec(),
                            }));
                        }
                    } else if let OptionType::NumberList { default } = &self.data.op_type {
                        let mut value = default.clone();
                        value.push(number);
                        value.sort();
                        sender.output(ModuleOptionOutput::SetOption(
                            self.data.id.to_string(),
                            ModuleOption::NumberList {
                                value: value.to_vec(),
                            },
                        ));
                        list_option_factory_guard.push_back(ListOptionInit {
                            value: entry.to_string(),
                        });
                        list_option_factory_guard.drop();
                        self.set_value(Some(ModuleOption::NumberList {
                            value: value.to_vec(),
                        }));
                    }
                }
            }
            ModuleOptionInput::RemoveExpanderOption(entry, index) => {
                let mut list_option_factory_guard = self.list_option_factory.guard();
                if let Some(ModuleOption::NumberList { mut value }) = self.value.clone() {
                    value.retain(|x| x.to_string() != entry);
                    sender.output(ModuleOptionOutput::SetOption(
                        self.data.id.to_string(),
                        ModuleOption::NumberList {
                            value: value.to_vec(),
                        },
                    ));
                    list_option_factory_guard.remove(index.current_index());
                    list_option_factory_guard.drop();
                    self.set_value(Some(ModuleOption::NumberList {
                        value: value.to_vec(),
                    }));
                } else if let OptionType::NumberList { default } = &self.data.op_type {
                    let mut value = default.clone();
                    value.retain(|x| x.to_string() != entry);
                    sender.output(ModuleOptionOutput::SetOption(
                        self.data.id.to_string(),
                        ModuleOption::NumberList {
                            value: value.to_vec(),
                        },
                    ));
                    list_option_factory_guard.remove(index.current_index());
                    list_option_factory_guard.drop();
                    self.set_value(Some(ModuleOption::NumberList {
                        value: value.to_vec(),
                    }));
                }
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        let output = match output {
            ModuleOptionOutput::SetOption(id, value) => ModulePageInput::SetModuleOption(id, value),
        };
        Some(output)
    }
}
