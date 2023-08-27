use crate::{
    modules::ModuleOption,
    ui::window::AppInput, config::LIBEXECDIR,
};
use adw::{gio, glib};
use log::{info, warn};
use relm4::{
    gtk::{
        self,
        prelude::{ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
    },
    ComponentParts, ComponentSender, SimpleComponent,
};
use std::{collections::HashMap, path::PathBuf};
use vte::{TerminalExt, TerminalExtManual};

#[tracker::track]
pub struct RebuildModel {
    visible: bool,
    status: RebuildStatus,
    terminal: vte::Terminal,

    flakepath: PathBuf,
    modulepath: PathBuf,
    generations: Option<u32>,
}

#[derive(Debug)]
pub enum RebuildInput {
    Rebuild(HashMap<String, ModuleOption>, String),
    Close,
    SetStatus(RebuildStatus),
}

pub struct RebuildInit {
    pub flakepath: PathBuf,
    pub modulepath: PathBuf,
    pub generations: Option<u32>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RebuildStatus {
    Building,
    Success,
    Error,
}

#[relm4::component(pub)]
impl SimpleComponent for RebuildModel {
    type Input = RebuildInput;
    type Output = AppInput;
    type Init = RebuildInit;

    view! {
        #[root]
        dialog = adw::Window {
            add_css_class: "csd",
            add_css_class: "messagedialog",
            #[track(model.changed(RebuildModel::visible()))]
            set_visible: model.visible,
            set_modal: true,
            set_resizable: true,
            set_default_width: 500,
            set_default_height: 300,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                gtk::Box {
                    add_css_class: "message-area",
                    set_orientation: gtk::Orientation::Vertical,
                    match model.status {
                        RebuildStatus::Building => {
                            gtk::Spinner {
                                set_spinning: true,
                                set_height_request: 60,    
                            }
                        },
                        RebuildStatus::Success => {
                            gtk::Image {
                                add_css_class: "success",
                                set_icon_name: Some("object-select-symbolic"),
                                set_pixel_size: 128,
                            }
                        },
                        RebuildStatus::Error => {
                            gtk::Image {
                                add_css_class: "error",
                                set_icon_name: Some("dialog-error-symbolic"),
                                set_pixel_size: 128,
                            }
                        }
                    },
                    gtk::Label {
                        add_css_class: "title-2",
                        #[track(model.changed(RebuildModel::status()))]
                        set_text: match model.status {
                            RebuildStatus::Building => "Rebuilding",
                            RebuildStatus::Success => "Done!",
                            RebuildStatus::Error => "Error!",
                        }
                    },
                    gtk::Label {
                        #[track(model.changed(RebuildModel::status()))]
                        set_text: match model.status {
                            RebuildStatus::Building => "This may take a few minutes.",
                            RebuildStatus::Success => "All changes have applied!",
                            RebuildStatus::Error => "Error encountered during rebuild process."
                        },
                    }
                },
                gtk::Frame {
                    set_margin_start: 15,
                    set_margin_end: 15,
                    set_margin_bottom: 15,
                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        #[local_ref]
                        terminal -> vte::Terminal {
                            set_vexpand: true,
                            set_hexpand: true,
                            set_input_enabled: false,
                            connect_child_exited[sender, rebuild_status = model.status.clone()] => move |_term, status| {
                                if status == 0 {
                                    info!("Rebuild finished successfully");
                                    if rebuild_status == RebuildStatus::Building {
                                        sender.input(RebuildInput::SetStatus(RebuildStatus::Success));
                                    }
                                } else {
                                    warn!("Rebuild failed with status {}", status);
                                    sender.input(RebuildInput::SetStatus(RebuildStatus::Error));
                                }
                            }
                        }
                    }
                },
                gtk::Separator {
                    #[track(model.changed(RebuildModel::status()))]
                    set_visible: model.status != RebuildStatus::Building,
                    set_valign: gtk::Align::End,
                },
                gtk::Box {
                    #[track(model.changed(RebuildModel::status()))]
                    set_visible: model.status != RebuildStatus::Building,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_valign: gtk::Align::End,
                    add_css_class: "response-area",
                    gtk::Button {
                        add_css_class: "flat",
                        set_hexpand: true,
                        set_label: "Close",
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildInput::Close);
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RebuildModel {
            visible: false,
            status: RebuildStatus::Building,
            terminal: vte::Terminal::new(),
            flakepath: init.flakepath,
            modulepath: init.modulepath,
            generations: init.generations,
            tracker: 0,
        };
        let terminal = &model.terminal;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match message {
            RebuildInput::Rebuild(modified_config, moduleconfig) => {
                self.set_visible(true);
                sender.input(RebuildInput::SetStatus(RebuildStatus::Building));
                let mut output = moduleconfig;
                for (attribute, value) in modified_config {
                    output = nix_editor::write::write(&output, &attribute, &value.value())
                        .unwrap()
                        .to_string();
                }
                output = nixpkgs_fmt::reformat_string(&output);
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &[
                        "/usr/bin/env",
                        "pkexec",
                        &format!("{}/smm-helper", LIBEXECDIR),
                        "write-rebuild",
                        "--content",
                        &output,
                        "--path",
                        &self.modulepath.to_string_lossy(),
                        "--",
                        "switch",
                        "--flake",
                        &self.flakepath.to_string_lossy(),
                    ],
                    &[],
                    glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |_| (),
                );
            }
            RebuildInput::Close => {
                self.terminal.reset(true, true);
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &["/usr/bin/env", "clear"],
                    &[],
                    glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |_| (),
                );
                self.set_visible(false);
                let _ = sender.output(AppInput::Reload);
            }
            RebuildInput::SetStatus(status) => {
                self.set_status(status);
            }
        }
    }
}
