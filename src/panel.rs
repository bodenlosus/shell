use gtk::{gio, glib::{self, Object}, prelude::GtkWindowExt};

mod inner { 
    use crate::center::Center;

    use super::*;

    use adw::subclass::{application_window::AdwApplicationWindowImpl, prelude::ObjectImplExt};
    use gtk::subclass::{prelude::*, widget::WidgetImpl, window::WindowImpl};
    use gtk::CompositeTemplate;
    use gtk4_layer_shell::{Edge, Layer, LayerShell};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/shell/ui/panel.ui")]
    pub struct Panel {
        #[template_child(id="center")]
        center: TemplateChild<Center>
    }


    #[glib::object_subclass]
    impl ObjectSubclass for Panel {
        const NAME: &'static str = "Panel";
        type Type = super::Panel;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Panel {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.init_layer_shell();

            obj.set_anchor(Edge::Bottom, true);
            obj.set_anchor(Edge::Left, true);
            obj.set_anchor(Edge::Right, true);
            obj.set_layer(Layer::Top);
            obj.auto_exclusive_zone_enable();

        }
    }
    impl WidgetImpl for Panel {}
    impl WindowImpl for Panel {}
    impl ApplicationWindowImpl for Panel {}
    impl AdwApplicationWindowImpl for Panel {}
}

glib::wrapper! {
    pub struct Panel(ObjectSubclass<inner::Panel>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Panel {
    pub fn new(app: &adw::Application) -> Self {
        let obj: Panel = Object::new();
        obj.set_application(Some(app));
        obj
    }
}