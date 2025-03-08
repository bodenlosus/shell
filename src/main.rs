mod niri;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use niri::Niri;
use niri_ipc::Response;
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};
use gtk4_layer_shell::{self, LayerShell, Edge};

struct AppModel {
    counter: u8,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = u8;

    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 30,
            init_layer_shell: (),
            set_anchor: (Edge::Bottom, true),
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, true),
            auto_exclusive_zone_enable: (),

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked => AppMsg::Increment
                },

                gtk::Button::with_label("Decrement") {
                    connect_clicked => AppMsg::Decrement
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                    set_margin_all: 5,
                }
            }
        }
    }

    // Initialize the UI.
    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel { counter };

        // Insert the macro code generation here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

// fn main() {
//     let app = RelmApp::new("relm4.test.simple");
//     app.run::<AppModel>(0);
// }

fn main() {
    let i = Niri::connect().unwrap();
    i.stream(|event| println!("{event:?}"));
}