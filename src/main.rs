mod bento;
mod notification_server;
mod notification_display;
mod panel;
mod time;
mod utils;
mod notifications;
use adw::prelude::*;
use gtk::{gio, CssProvider};
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
    let panel = panel::Panel::new(&app, None);
    
    panel.present();
}

fn load_resources() {
    gio::resources_register_include!("shell.gresource").expect("failed to register resources ");
}
