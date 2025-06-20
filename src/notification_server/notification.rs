use gtk::glib::{self, Object};

mod inner {

    use glib::prelude::ObjectExt;
    use glib::subclass::object::{DerivedObjectProperties, ObjectImpl, ObjectImplExt};
    use glib::{self, Properties};
    use std::cell::RefCell;

    use super::*;
    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::Notification)]
    pub struct Notification {
        #[property(get, set, minimum = 0u32, maximum = u32::MAX, default = 0u32)]
        pub id: RefCell<u32>,

        #[property(get, set)]
        pub app_name: RefCell<String>,

        #[property(get, set, minimum = 0u32, maximum = u32::MAX, default = 0u32)]
        pub replaces_id: RefCell<u32>,

        #[property(get, set)]
        pub app_icon: RefCell<String>,

        #[property(get, set)]
        pub summary: RefCell<String>,

        #[property(get, set)]
        pub body: RefCell<String>,

        # [property(get, set)]
        pub actions: RefCell<Vec<String>>,

        #[property(get, set, minimum = i32::MIN, maximum = i32::MAX, default = -1i32)]
        pub expire_timeout: RefCell<i32>,
    }

    #[glib::object_subclass]
    impl glib::subclass::types::ObjectSubclass for Notification {
        const NAME: &'static str = "Notification";
        type Type = super::Notification;
        type ParentType = glib::Object;

        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }
    }
    #[glib::derived_properties]
    impl ObjectImpl for Notification {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}

glib::wrapper! {
    pub struct Notification(ObjectSubclass<inner::Notification>);
}

impl Notification {
    pub fn new(
        id: u32,
        app_name: impl Into<String>,
        replaces_id: u32,
        app_icon: impl Into<String>,
        summary: impl Into<String>,
        body: impl Into<String>,
        actions: impl Into<Vec<String>>,
        expire_timeout: i32,
    ) -> Self {
        let obj = Object::builder()
            .property("id", id)
            .property("app-name", app_name.into())
            .property("replaces-id", replaces_id)
            .property("app-icon", app_icon.into())
            .property("summary", summary.into())
            .property("body", body.into())
            .property("actions", actions.into())
            .property("expire-timeout", expire_timeout)
            .build();
        obj
    }

    pub fn from_variant(id: u32, variant: &glib::Variant) -> Option<Self> {
        variant
            .get::<(
                String,
                u32,
                String,
                String,
                String,
                Vec<String>,
                glib::VariantDict,
                i32,
            )>()
            .map(|(app_name, replaces_id, app_icon, summary, body, actions, _, expire_timeout)| {
                Self::new(id, app_name, replaces_id, app_icon, summary, body, actions, expire_timeout)
            })
    }
}
