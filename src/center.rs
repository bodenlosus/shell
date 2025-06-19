use gtk::glib::{self, Object};
use gtk::CompositeTemplate;

mod inner {

    use adw::subclass::{bin::BinImpl, prelude::ObjectImplExt};
    use gtk::{
        prelude::PopoverExt,
        subclass::{prelude::*, widget::WidgetImpl},
    };

    use crate::{notification_server, time};

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/shell/ui/center.ui")]
    pub struct Center {
        #[template_child(id = "time-label")]
        pub time_label: TemplateChild<gtk::Label>,
        #[template_child(id = "popover")]
        pub popover: TemplateChild<gtk::Popover>,

        #[template_child(id="time-module")]
        pub time_mod: TemplateChild<time::TimeModule>,

        server: Option<notification_server::NotificationServer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Center {
        const NAME: &'static str = "Center";
        type Type = super::Center;
        type ParentType = adw::Bin;

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

    impl ObjectImpl for Center {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = &self.obj();
            let imp = obj.imp();
            imp.popover.set_offset(0, -10);
            imp.time_mod.bind_time_to(&imp.time_label.get(), "label", "%x %X");
        }
    }
    impl BinImpl for Center {}
    impl WidgetImpl for Center {}
}

glib::wrapper! {
    pub struct Center(ObjectSubclass<inner::Center>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Center {
    pub fn new() -> Self {
        let obj = Object::new();
        obj
    }
}
