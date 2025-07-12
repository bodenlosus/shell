use gtk::glib::{self, Object};
use gtk::CompositeTemplate;

mod inner {

    use adw::subclass::{bin::BinImpl, prelude::ObjectImplExt};
    use gtk::{
        prelude::PopoverExt,
        subclass::{prelude::*, widget::WidgetImpl},
    };

    use crate::{notification_server, notifications, time};

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/shell/ui/bento.ui")]
    pub struct BentoGrid {
        #[template_child(id="time-module")]
        pub time_mod: TemplateChild<time::TimeModule>,

        #[template_child(id = "notifications-module")]
        pub notifications_module: TemplateChild<notifications::NotificationsModule>,

        #[template_child(id = "grid")]
        pub grid: TemplateChild<gtk::Grid>,

        server: Option<notification_server::NotificationServer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BentoGrid {
        const NAME: &'static str = "BentoGrid";
        type Type = super::BentoGrid;
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

    impl ObjectImpl for BentoGrid {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = &self.obj();
            let imp = obj.imp();
        }
    }
    impl BinImpl for BentoGrid {}
    impl WidgetImpl for BentoGrid {}
}

glib::wrapper! {
    pub struct BentoGrid(ObjectSubclass<inner::BentoGrid>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl BentoGrid {
    pub fn new() -> Self {
        let obj = Object::new();
        obj
    }
}
