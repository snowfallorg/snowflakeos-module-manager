use super::{changes_factory::ModuleChangesModel, ModificationType};
use crate::{
    modules::{Module, ModuleOption},
    ui::{
        rebuild::{changes_factory::ModuleChangesInit, OptionModification},
        window::AppInput,
    },
};
use adw::{prelude::*, traits::MessageDialogExt};
use relm4::{factory::FactoryVecDeque, gtk, ComponentParts, ComponentSender, SimpleComponent};
use std::collections::HashMap;

pub struct ConfirmDialogModel {
    modules: Vec<Module>,
    visible: bool,
    changes_factory: FactoryVecDeque<ModuleChangesModel>,
}

#[derive(Debug)]
pub enum ConfirmDialogInput {
    Open(HashMap<String, ModuleOption>, HashMap<String, ModuleOption>),
    Close,
}

pub struct ConfirmDialogInit {
    pub modules: Vec<Module>,
}

#[relm4::component(pub)]
impl SimpleComponent for ConfirmDialogModel {
    type Input = ConfirmDialogInput;
    type Output = AppInput;
    type Init = ConfirmDialogInit;

    view! {
        #[root]
        dialog = adw::MessageDialog {
            #[watch]
            set_visible: model.visible,
            set_modal: true,
            set_heading: Some("Apply changes?"),
            set_body: "The following changes will be applied. This may take some time.",
            #[wrap(Some)]
            #[local_ref]
            set_extra_child = changes_factory_box -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
            },
            add_response: ("cancel", "Cancel"),
            add_response: ("continue", "Continue"),
            set_response_appearance: ("continue", adw::ResponseAppearance::Suggested),
            connect_close_request => |_| {
                gtk::Inhibit(true)
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let changes_factory =
            FactoryVecDeque::new(gtk::Box::builder().build(), sender.input_sender());
        let model = ConfirmDialogModel {
            modules: init.modules,
            visible: false,
            changes_factory,
        };
        let changes_factory_box = model.changes_factory.widget();
        let widgets = view_output!();
        widgets
            .dialog
            .connect_response(None, move |_, resp| match resp {
                "cancel" => sender.input(ConfirmDialogInput::Close),
                "continue" => {
                    sender.input(ConfirmDialogInput::Close);
                    let _ = sender.output(AppInput::Rebuild);
                }
                _ => unreachable!(),
            });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ConfirmDialogInput::Open(current_config, modified_config) => {
                self.visible = true;
                let mut changes_factory_guard = self.changes_factory.guard();
                changes_factory_guard.clear();
                for module in &self.modules {
                    let options = &module.config.options;
                    let matches = options
                        .iter()
                        .filter_map(|x| {
                            modified_config
                                .get(&x.id)
                                .map(|modified_value| OptionModification {
                                    label: x.label.to_string(),
                                    mod_type: if let Some(current_value) = current_config.get(&x.id)
                                    {
                                        ModificationType::Update {
                                            new: modified_value.to_string(),
                                            old: current_value.to_string(),
                                        }
                                    } else {
                                        ModificationType::New {
                                            value: modified_value.to_string(),
                                        }
                                    },
                                })
                        })
                        .collect::<Vec<_>>();
                    if !matches.is_empty() {
                        changes_factory_guard.push_back(ModuleChangesInit {
                            modifications: matches,
                            label: module.config.name.to_string(),
                        });
                    }
                }
            }
            ConfirmDialogInput::Close => self.visible = false,
        }
    }
}
