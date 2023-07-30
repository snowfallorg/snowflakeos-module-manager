use std::collections::HashMap;

use adw::prelude::{OrientableExt, WidgetExt, ButtonExt, BoxExt};
use log::error;
use relm4::{gtk, SimpleComponent, ComponentSender, ComponentParts, RelmWidgetExt, factory::FactoryVecDeque};

use crate::{modules::{ModuleData, ModuleOption}, ui::{window::AppInput, module::option_factory::ModuleOptionInit}};

use super::option_factory::ModuleOptionModel;

#[tracker::track]
pub struct ModulePageModel {
    data: Option<ModuleData>,
    #[tracker::no_eq]
    optionfactory: FactoryVecDeque<ModuleOptionModel>,
    show_apply: bool,
}

#[derive(Debug)]
pub enum ModulePageInput {
    OpenModulePage(ModuleData, HashMap<String, ModuleOption>, HashMap<String, ModuleOption>),
    SetModuleOption(String, ModuleOption),
    ShowApply(bool),
}


pub struct ModulePageInit {}

#[relm4::component(pub)]
impl SimpleComponent for ModulePageModel {
    type Input = ModulePageInput;
    type Output = AppInput;
    type Init = ModulePageInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            adw::HeaderBar {
                #[wrap(Some)]
                set_title_widget = &gtk::Label {
                    #[track(model.changed(ModulePageModel::data()))]
                    set_label: model.data.as_ref().map(|data| data.name.as_str()).unwrap_or("Unknown"),
                },
                pack_start = &gtk::Button {
                    add_css_class: "flat",
                    set_icon_name: "go-previous-symbolic",
                    connect_clicked[sender] => move |_| {
                        let _ = sender.output(AppInput::CloseModulePage);
                    },
                },
                pack_end = &gtk::Button {
                    #[track(model.changed(ModulePageModel::show_apply()))]
                    set_visible: model.show_apply,
                    add_css_class: "suggested-action",
                    set_label: "Apply",
                    connect_clicked[sender] => move |_| {
                        if sender.output(AppInput::ApplyChanges).is_err() { error!("Error sender AppInput::Applychanges") }
                    }
                }
            },
            gtk::ScrolledWindow {
                adw::Clamp {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,
                        set_margin_all: 15,
                        gtk::Label {
                            add_css_class: "title-1",
                            set_halign: gtk::Align::Start,
                            #[track(model.changed(ModulePageModel::data()))]
                            set_label: model.data.as_ref().map(|data| data.name.as_str()).unwrap_or("Unknown"),
                        },
                        gtk::Label {
                            add_css_class: "dim-label",
                            set_halign: gtk::Align::Start,
                            #[track(model.changed(ModulePageModel::data()))]
                            set_visible: model.data.as_ref().and_then(|data| data.description.as_deref()).is_some(),
                            #[track(model.changed(ModulePageModel::data()))]
                            set_label: model.data.as_ref().and_then(|data| data.description.as_deref()).unwrap_or_default(),
                        },
                        #[local_ref]
                        optionfactory_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 15,
                            set_margin_top: 15,
                            set_margin_bottom: 15,
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let optionfactory = FactoryVecDeque::new(gtk::Box::new(gtk::Orientation::Vertical, 0), sender.input_sender());
        let model = ModulePageModel {
            data: None,
            optionfactory,
            show_apply: false,
            tracker: 0,
        };
        let optionfactory_box = model.optionfactory.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match message {
            ModulePageInput::OpenModulePage(data, current_config, modified_config) => {
                self.set_data(Some(data));
                let mut optionfactory_guard = self.optionfactory.guard();
                optionfactory_guard.clear();
                if let Some(options) = self.data.as_ref().map(|x| x.options.to_vec()) {
                    for option in options {
                        let modified_value = modified_config.get(&option.id);
                        let value = current_config.get(&option.id);
                        optionfactory_guard.push_back(ModuleOptionInit {
                            data: option,
                            value: modified_value.cloned().or(value.cloned())
                        });
                    }
                }
            },
            ModulePageInput::SetModuleOption(id, value) => {
                if sender.output(AppInput::SetModuleOption(id, value)).is_err() { error!("Error sending: AppInput::SetModuleOption") }
            },
            ModulePageInput::ShowApply(show) => {
                self.set_show_apply(show)
            }
        }
    }
}
