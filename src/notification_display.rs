use adw::subclass::prelude::{ObjectSubclassExt, ObjectSubclassIsExt};
use gio::prelude::AppInfoExt;
use gtk::{
    gdk::prelude::TextureExt,
    glib::{
        self,
        object::ObjectExt,
        Object,
    },
    prelude::{BoxExt, WidgetExt},
    CompositeTemplate,
};

use crate::notification_server;

mod inner {

    use gtk::prelude::ObjectExt;
    

    use super::*;
    use gtk::glib::{self, derived_properties, Properties};
    use gtk::subclass::prelude::*;
    use gtk::template_callbacks;
    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::NotificationDisplay)]
    #[template(resource = "/shell/ui/notification.ui")]
    pub struct NotificationDisplay {
        #[template_child(id = "app-label")]
        pub app_label: TemplateChild<gtk::Label>,

        // #[template_child(id = "app-icon")]
        // pub app_icon: TemplateChild<gtk::Image>,
        #[template_child(id = "time-label")]
        pub date_label: TemplateChild<gtk::Label>,

        #[template_child(id = "title-label")]
        pub title_label: TemplateChild<gtk::Label>,

        #[template_child(id = "body-label")]
        pub body_label: TemplateChild<gtk::Label>,
    }

    #[template_callbacks]
    impl NotificationDisplay {
        #[template_callback]
        fn on_close(&self) {}
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationDisplay {
        const NAME: &'static str = "NotificationDisplay";
        type Type = super::NotificationDisplay;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    #[derived_properties]
    impl ObjectImpl for NotificationDisplay {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = &self.obj();
            let imp = obj.imp();
        }
    }
    impl BoxImpl for NotificationDisplay {}
    impl WidgetImpl for NotificationDisplay {}
}

glib::wrapper! {
    pub struct NotificationDisplay(ObjectSubclass<inner::NotificationDisplay>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl NotificationDisplay {
    pub fn new() -> Self {
        let obj = Object::new();
        obj
    }

    pub fn set_from_notification(&self, notification: &notification_server::NotificationItem) {
        let imp = self.imp();

        imp.app_label.set_label(&notification.app_name());

        imp.title_label.set_label(&notification.summary());
        imp.body_label.set_label(&notification.body());

        let hints = notification.get_hints();

        match hints.urgency {
            notification_server::Urgency::Critical => {
                self.add_css_class("critical");
            }
            _ => {}
        };

        let image = notification.get_image_square();

        if let Some(image) = image {
            let texture = gtk::gdk::Texture::for_pixbuf(&image);
            let picture = gtk::Picture::builder()
                .paintable(&texture)
                .vexpand(true)
                .css_classes(["rounded-sm"])
                .build();

            self.append(&picture);
        }
        if let Some(datestr) = notification.timestamp().and_then(|dt| dt.format("%X").ok()) {
            imp.date_label.set_label(&datestr);
        }
    }
}
