mod center;
mod notification_server;
mod panel;
mod time;
mod utils;
use adw::prelude::*;
use gtk::{gio, CssProvider};
use notification_server::NotificationServer;
const APP_ID: &'static str = "io.github.johannes.shell";
const DATA_DIR: &'static str = "/home/johannes/bracket/data/";

fn main() -> () {
    load_resources();

    let app = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);
    app.connect_startup(|_| {
        load_css();
    });
    app.connect_activate(build_ui);
    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("styles/style.css");
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &adw::Application) {
    let server = NotificationServer::new();
    let store = server.get_store();
    store.connect_notify(name, |x, p| {});
    let panel = panel::Panel::new(&app, Some(server.get_store()));
    
    server.connect_to_dbus();
    
    panel.present();
}

fn load_resources() {
    gio::resources_register_include!("shell.gresource").expect("failed to register resources ");
}
