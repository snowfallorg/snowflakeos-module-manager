use super::option_factory::ModuleOptionInput;
use adw::prelude::*;
use relm4::{
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender,
};

pub struct ListOptionModel {
    value: String,
    index: DynamicIndex,
}

#[derive(Debug)]
pub enum ListOptionInput {}

#[derive(Debug)]
pub enum ListOptionOutput {
    Remove(String, DynamicIndex),
}

pub struct ListOptionInit {
    pub value: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for ListOptionModel {
    type ParentWidget = adw::ExpanderRow;
    type ParentInput = ModuleOptionInput;
    type Input = ListOptionInput;
    type Output = ListOptionOutput;
    type Init = ListOptionInit;
    type CommandOutput = ();

    view! {
        #[root]
        adw::ActionRow {
            #[watch]
            set_title: &self.value,
            add_suffix = &gtk::Button {
                set_valign: gtk::Align::Center,
                set_icon_name: "user-trash-symbolic",
                connect_clicked[sender, value = self.value.clone(), index = self.index.clone()] => move |_| {
                    sender.output(ListOptionOutput::Remove(value.to_string(), index.clone()))
                }
            }
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            value: init.value,
            index: index.clone(),
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
        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {}
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        let output = match output {
            ListOptionOutput::Remove(value, index) => {
                ModuleOptionInput::RemoveExpanderOption(value, index)
            }
        };
        Some(output)
    }
}
