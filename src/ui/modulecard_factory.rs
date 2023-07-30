use super::window::AppInput;
use crate::modules::{IconType, Module, ModuleData};
use adw::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use log::debug;
use relm4::{
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

pub struct ModuleCardModel {
    module: Module,
}

#[derive(Debug)]
pub enum ModuleCardInput {}

#[derive(Debug)]
pub enum ModuleCardOutput {
    Clicked(ModuleData),
}

pub struct ModuleCardInit {
    pub module: Module,
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleCardModel {
    type ParentWidget = gtk::Box;
    type ParentInput = AppInput;
    type Input = ModuleCardInput;
    type Output = ModuleCardOutput;
    type Init = ModuleCardInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Button {
            add_css_class: "card",
            connect_clicked[sender, data = self.module.config.clone()] => move |_| {
                sender.output(ModuleCardOutput::Clicked(data.clone()))
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 15,
                set_margin_all: 15,
                #[name(image)]
                gtk::Image {
                    set_pixel_size: 64,
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    set_hexpand: true,
                    gtk::Label {
                        add_css_class: "title-3",
                        set_halign: gtk::Align::Start,
                        set_label: &self.module.config.name,
                    },
                    gtk::Label {
                        add_css_class: "dim-label",
                        set_halign: gtk::Align::Start,
                        set_label: &self.module.config.version,
                    },
                    gtk::Label {
                        add_css_class: "heading",
                        set_halign: gtk::Align::Start,
                        set_label: self.module.config.description.as_deref().unwrap_or_default(),
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            module: init.module,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        if let Some(icondata) = &self.module.config.icon {
            match icondata.icon_type {
                IconType::File => {
                    debug!(
                        "icondata.path: {}/{}",
                        self.module.path.to_string_lossy(),
                        icondata.path
                    );
                    widgets.image.set_file(Some(
                        format!("{}/{}", self.module.path.to_string_lossy(), icondata.path)
                            .as_str(),
                    ));
                }
                IconType::System => {
                    widgets.image.set_icon_name(Some(icondata.path.as_str()));
                }
            }
        } else {
            widgets
                .image
                .set_icon_name(Some("application-x-executable"));
        }
        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {}
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        let output = match output {
            ModuleCardOutput::Clicked(data) => AppInput::OpenModulePage(data),
        };
        Some(output)
    }
}
