
use gtk::{glib};

use log::error;
use relm4::*;
use snowflakeos_module_manager::ui::{window::{AppModel, AppInit}, load::load};
fn main() {
    gtk::init().unwrap();
    pretty_env_logger::init();
	glib::set_application_name("Module Manager");
    // if let Ok(res) = gio::Resource::load(RESOURCES_FILE) {
    //     info!("Resource loaded: {}", RESOURCES_FILE);
    //     gio::resources_register(&res);
    // } else {
    //     error!("Failed to load resources");
    // }
    // gtk::Window::set_default_icon_name(nix_software_center::config::APP_ID);
    // let app = adw::Application::new(Some(nix_software_center::config::APP_ID), gio::ApplicationFlags::empty());
    // app.set_resource_base_path(Some("/dev/vlinkz/NixSoftwareCenter"));
    let app = RelmApp::new("org.snowflakeos.modulemanager");

    match load() {
        Ok(load) => app.run::<AppModel>(AppInit { load }),
        Err(e) => {
            error!("Failed to load: {}", e);
            std::process::exit(1);
        }
    }
}
