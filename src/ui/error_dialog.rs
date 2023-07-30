use relm4::{SimpleComponent, ComponentSender, ComponentParts};
use adw::{prelude::*, gtk};

use super::window::AppInput;

#[tracker::track]
pub struct ErrorDialogModel {
    visible: bool,
    title: String,
    body: String,
}

#[derive(Debug)]
pub enum ErrorDialogInput {
    Show(String, String),
    Close
}

#[relm4::component(pub)]
impl SimpleComponent for ErrorDialogModel {
    type Input = ErrorDialogInput;
    type Output = AppInput;
    type Init = ();

    view! {
        #[root]
        #[name(dialog)]
        adw::MessageDialog {
            #[track(model.changed(ErrorDialogModel::visible()))]
            set_visible: model.visible,
            set_modal: true,
            #[track(model.changed(ErrorDialogModel::title()))]
            set_heading: Some(model.title.as_str()),
            #[track(model.changed(ErrorDialogModel::body()))]
            set_body: model.body.as_str(),
            add_response: ("reset", "Reset"),
            add_response: ("quit", "Quit"),
            connect_close_request => |_| {
                gtk::Inhibit(true)
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ErrorDialogModel {
            visible: false,
            title: String::new(),
            body: String::new(),
            tracker: 0,
        };
        let widgets = view_output!();
        widgets
        .dialog
        .connect_response(None, move |_, resp| match resp {
            "quit" => relm4::main_application().quit(),
            "reset" => {
                sender.input(ErrorDialogInput::Close);
                let _ = sender.output(AppInput::Reload);
            }
            _ => unreachable!(),
        });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match message {
            ErrorDialogInput::Show(title, body) => {
                self.set_visible(true);
                self.set_title(title);
                self.set_body(body);
            },
            ErrorDialogInput::Close => self.set_visible(false),
        }
    }
}
