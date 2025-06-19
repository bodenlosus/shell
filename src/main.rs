mod center;
mod notifications;
mod panel;
mod time;
mod notification;
mod utils;
use adw::prelude::*;
use gtk::{gio, glib::bitflags::Flags, CssProvider};

const APP_ID: &'static str = "io.github.johannes.shell";
const DATA_DIR: &'static str = "/home/johannes/bracket/data/";

fn main() -> () {
    load_resources();

    let app = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);
    app.connect_startup(|_| {
        load_css();
    });
    app.connect_activate(build_ui);

    gtk::glib::spawn_future_local(async move {
        let mut server = notifications::server::Server::new();
        let r = server.take_reciever();
        server.connect_to_dbus();
        if let Some(r) = r {
            while let Ok(event) = r.recv().await {
                match event {
                    notifications::server::NotificationEvent::Add(n) => {
                        println!("{n:?}");
                    }
                    notifications::server::NotificationEvent::Close(id) => {
                        println!("closed: {id}");
                    }
                }
            }
        }
    });
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
    // let ic = gtk::IconTheme::default();
    // println!("Theme Name: {}", ic.theme_name());
    // println!("Icons: {:#?}, ", ic.icon_names());
    // println!("HAS ICON: {}", ic.has_icon("hourglass-symbolic"));
    let panel = panel::Panel::new(&app);
    panel.present();
}

fn load_resources() {
    gio::resources_register_include!("shell.gresource").expect("failed to register resources ");
}
