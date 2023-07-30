use super::{
    error_dialog::{ErrorDialogInput, ErrorDialogModel},
    load::{reload, ReloadOutput},
    module::page::{ModulePageInput, ModulePageModel},
    modulecard_factory::ModuleCardModel,
    rebuild::{
        confirm_dialog::ConfirmDialogModel,
        rebuild_dialog::{RebuildInput, RebuildModel},
    },
};
use crate::{
    modules::{ModuleData, ModuleOption},
    ui::{
        load::LoadOutput,
        module::page::ModulePageInit,
        modulecard_factory::ModuleCardInit,
        rebuild::{
            confirm_dialog::{ConfirmDialogInit, ConfirmDialogInput},
            rebuild_dialog::RebuildInit,
        }, about::AboutPageModel,
    },
};
use adw::{gtk, prelude::*};
use nix_data::config::configfile::NixDataConfig;
use relm4::{
    adw, factory::FactoryVecDeque, Component, ComponentController, ComponentParts, ComponentSender,
    Controller, RelmWidgetExt, SimpleComponent, actions::{RelmActionGroup, RelmAction},
};
use std::{collections::HashMap, convert::identity};

pub struct AppModel {
    config: NixDataConfig,
    modulecardsfactory: FactoryVecDeque<ModuleCardModel>,
    modulepage: Controller<ModulePageModel>,
    aboutpage: Controller<AboutPageModel>,
    main_leaflet: adw::Leaflet,
    main_box: gtk::Box,

    confirm_dialog: Controller<ConfirmDialogModel>,
    rebuild_dialog: Controller<RebuildModel>,
    error_dialog: Controller<ErrorDialogModel>,

    moduleconfig: String,

    current_config: HashMap<String, ModuleOption>,
    modified_config: HashMap<String, ModuleOption>,
}

#[derive(Debug)]
pub enum AppInput {
    OpenModulePage(ModuleData),
    CloseModulePage,
    SetModuleOption(String, ModuleOption),
    ApplyChanges,
    Rebuild,
    Reload,
}

#[derive(Debug)]
pub enum AppOutput {}

pub struct AppInit {
    pub load: LoadOutput,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Input = AppInput;
    type Output = AppOutput;
    type Init = AppInit;

    view! {
        #[root]
        #[name(main_window)]
        adw::ApplicationWindow {
            set_default_size: (800, 600),
            #[local_ref]
            main_leaflet -> adw::Leaflet {
                set_can_unfold: false,
                set_homogeneous: false,
                set_transition_type: adw::LeafletTransitionType::Over,
                set_can_navigate_back: true,
                #[local_ref]
                main_box -> gtk::Box {
                    set_vexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Fill,
                    set_orientation: gtk::Orientation::Vertical,
                    adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &gtk::Label {
                            set_label: "SnowflakeOS Module Manager"
                        },
                        pack_end = &gtk::Button {
                            #[watch]
                            set_visible: !model.modified_config.is_empty(),
                            add_css_class: "suggested-action",
                            set_label: "Apply",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppInput::ApplyChanges)
                            }
                        },
                        pack_end: menu = &gtk::MenuButton {
                            add_css_class: "flat",
                            set_icon_name: "open-menu-symbolic",
                            #[wrap(Some)]
                            set_popover = &gtk::PopoverMenu::from_model(Some(&mainmenu)) {
                                add_css_class: "menu"
                            }
                        }
                    },
                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_halign: gtk::Align::Fill,
                        set_valign: gtk::Align::Fill,
                        adw::Clamp {
                            #[local_ref]
                            modulecardsbox -> gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 15,
                                set_margin_all: 15,
                            }
                        }
                    }
                },
                append = model.modulepage.widget(),
            }

        }
    }

    menu! {
        mainmenu: {
            "About" => AboutAction,
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut modulecardsfactory = FactoryVecDeque::new(
            gtk::Box::new(gtk::Orientation::Vertical, 0),
            sender.input_sender(),
        );

        let LoadOutput {
            config,
            moduleconfig,
            modulepath,
            flakepath,
            modules,
            current_config,
        } = init.load;

        let mut modulecardsfactory_guard = modulecardsfactory.guard();
        for module in modules.iter() {
            modulecardsfactory_guard.push_back(ModuleCardInit {
                module: module.clone(),
            });
        }
        modulecardsfactory_guard.drop();

        let modulepage = ModulePageModel::builder()
            .launch(ModulePageInit {})
            .forward(sender.input_sender(), identity);
        let confirm_dialog = ConfirmDialogModel::builder()
            .transient_for(root)
            .launch(ConfirmDialogInit { modules })
            .forward(sender.input_sender(), identity);
        let rebuild_dialog = RebuildModel::builder()
            .transient_for(root)
            .launch(RebuildInit {
                flakepath,
                modulepath,
                generations: config.generations,
            })
            .forward(sender.input_sender(), identity);
        let error_dialog = ErrorDialogModel::builder()
            .transient_for(root)
            .launch(())
            .forward(sender.input_sender(), identity);
        let aboutpage = AboutPageModel::builder()
            .launch(root.clone().upcast())
            .detach();

        let model = AppModel {
            config,
            modulecardsfactory,
            modulepage,
            aboutpage,
            main_leaflet: adw::Leaflet::new(),
            main_box: gtk::Box::new(gtk::Orientation::Vertical, 0),
            moduleconfig,
            confirm_dialog,
            rebuild_dialog,
            error_dialog,
            current_config,
            modified_config: HashMap::new(),
        };
        let modulecardsbox = model.modulecardsfactory.widget();
        let main_leaflet = &model.main_leaflet;
        let main_box = &model.main_box;
        
        let widgets = view_output!();

        let mut group = RelmActionGroup::<MenuActionGroup>::new();
        let aboutpage: RelmAction<AboutAction> = {
            let sender = model.aboutpage.sender().clone();
            RelmAction::new_stateless(move |_| {
                sender.send(()).unwrap();
            })
        };
        group.add_action(aboutpage);
        let actions = group.into_action_group();
        widgets
            .main_window
            .insert_action_group("menu", Some(&actions));


        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::OpenModulePage(data) => {
                self.modulepage.emit(ModulePageInput::OpenModulePage(
                    data,
                    self.current_config.clone(),
                    self.modified_config.clone(),
                ));
                self.main_leaflet
                    .set_visible_child(self.modulepage.widget());
            }
            AppInput::CloseModulePage => {
                self.main_leaflet.set_visible_child(&self.main_box);
            }
            AppInput::SetModuleOption(id, value) => {
                // TODO: Decied what to do about default values. Allow user to deref/set to default?
                // Or always save config once set once?
                if self.current_config.get(&id) == Some(&value) {
                    self.modified_config.remove(&id);
                } else {
                    self.modified_config.insert(id, value);
                }
                self.modulepage
                    .emit(ModulePageInput::ShowApply(!self.modified_config.is_empty()))
            }
            AppInput::ApplyChanges => self.confirm_dialog.emit(ConfirmDialogInput::Open(
                self.current_config.clone(),
                self.modified_config.clone(),
            )),
            AppInput::Rebuild => self.rebuild_dialog.emit(RebuildInput::Rebuild(
                self.modified_config.clone(),
                self.moduleconfig.clone(),
            )),
            AppInput::Reload => match reload(&self.config) {
                Ok(ReloadOutput {
                    modules,
                    current_config,
                    moduleconfig,
                }) => {
                    self.current_config = current_config;
                    self.moduleconfig = moduleconfig;
                    self.modified_config.clear();
                    let mut modulecardsfactory_guard = self.modulecardsfactory.guard();
                    modulecardsfactory_guard.clear();
                    for module in modules.iter() {
                        modulecardsfactory_guard.push_back(ModuleCardInit {
                            module: module.clone(),
                        });
                    }
                    modulecardsfactory_guard.drop();
                    self.main_leaflet.set_visible_child(&self.main_box);
                }
                Err(e) => {
                    self.error_dialog.emit(ErrorDialogInput::Show(
                        "Failed to reload current module configuration".to_string(),
                        e.to_string(),
                    ));
                }
            },
        }
    }
}

relm4::new_action_group!(MenuActionGroup, "menu");
relm4::new_stateless_action!(AboutAction, MenuActionGroup, "about");
