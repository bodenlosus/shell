use gtk::{
    glib::{
        self, Object},
    CompositeTemplate,
};

use crate::notification_server;

mod inner {

    use adw::subclass::bin::BinImpl;
    use gio::glib::object::CastNone;
    use gtk::prelude::ListItemExt;
    

    use gtk::glib::{self};
    use gtk::subclass::prelude::*;
    use gtk::template_callbacks;

    use crate::notification_display;

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/shell/ui/notifications.ui")]
    pub struct NotificationsModule {
        #[template_child(id = "view")]
        pub view: TemplateChild<gtk::ListView>,
    }

    #[template_callbacks]
    impl NotificationsModule {
        #[template_callback]
        fn on_setup(&self, item: &gtk::ListItem) {
            let child = notification_display::NotificationDisplay::new();
            // let child = adw::Clamp::builder()
            //     .child(&display)
            //     .maximum_size(400)
            //     .build();
            item.set_child(Some(&child));
        }
        #[template_callback]
        fn on_bind(&self, item: &gtk::ListItem) {

            // let Some(child) = item.child().and_downcast::<adw::Clamp>() else {return;};
            let Some(child) = item.child().and_downcast::<notification_display::NotificationDisplay>() else {return;};
            let Some(item) = item.item().and_downcast::<notification_server::NotificationItem>() else {
                return;
            };

            child.set_from_notification(&item); 
        }
        #[template_callback]
        fn on_activate(listview: gtk::ListView, position: u32) {
            println!("{:?} {}", listview, position)
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationsModule {
        const NAME: &'static str = "NotificationsModule";
        type Type = super::NotificationsModule;
        type ParentType = adw::Bin;

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



    impl ObjectImpl for NotificationsModule {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = &self.obj();
            let imp = obj.imp();
            let server = notification_server::NotificationServer::new();
            let store = server.get_store();
            // let model = gtk::NoSelection::new(Some(store));
            // imp.view.set_model(Some(&model));
            server.connect_to_dbus();
        }
    }
    impl BinImpl for NotificationsModule {}
    impl WidgetImpl for NotificationsModule {}
}

glib::wrapper! {
    pub struct NotificationsModule(ObjectSubclass<inner::NotificationsModule>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl NotificationsModule {
    pub fn new() -> Self {
        let obj = Object::new();
        obj

    }
}
