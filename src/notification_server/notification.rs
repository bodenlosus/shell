use std::collections::HashMap;

use gio::glib::{
    property::PropertyGet, types::StaticType, value::{FromValue, ToValue}, variant::StaticVariantType,
};
use gtk::glib::{self, Object};

mod inner {

    use glib::prelude::ObjectExt;
    use glib::subclass::object::{DerivedObjectProperties, ObjectImpl, ObjectImplExt};
    use glib::{self, Properties};
    use std::cell::RefCell;

    use super::*;
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::NotificationItem)]
    pub struct NotificationItem {
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

        #[property(get, set)]
        pub actions: RefCell<Vec<String>>,

        #[property(get, set)]
        pub hints: RefCell<glib::VariantDict>,

        #[property(get, set, minimum = i32::MIN, maximum = i32::MAX, default = -1i32)]
        pub expire_timeout: RefCell<i32>,

        #[property(get, set)]
        pub timestamp: RefCell<Option<glib::DateTime>>,
    }

    #[glib::object_subclass]
    impl glib::subclass::types::ObjectSubclass for NotificationItem {
        const NAME: &'static str = "NotificationItem";
        type Type = super::NotificationItem;
        type ParentType = glib::Object;

        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }
    }
    #[glib::derived_properties]
    impl ObjectImpl for NotificationItem {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}

glib::wrapper! {
    pub struct NotificationItem(ObjectSubclass<inner::NotificationItem>);
}

impl NotificationItem {
    pub fn new(
        id: Option<u32>,
        app_name: impl Into<String>,
        replaces_id: u32,
        app_icon: impl Into<String>,
        summary: impl Into<String>,
        body: impl Into<String>,
        actions: impl Into<Vec<String>>,
        hints: impl Into<glib::VariantDict>,
        expire_timeout: i32,
        timestamp: Option<glib::DateTime>,
    ) -> Self {
        let id = id.unwrap_or(replaces_id);
        let obj = Object::builder()
            .property("id", id)
            .property("app-name", app_name.into())
            .property("replaces-id", replaces_id)
            .property("app-icon", app_icon.into())
            .property("summary", summary.into())
            .property("body", body.into())
            .property("actions", actions.into())
            .property("hints", hints.into())
            .property("expire-timeout", expire_timeout)
            .property("timestamp", timestamp)
            .build();
        obj
    }

    pub fn overwrite(&self, notification: &Self) {
        self.set_id(notification.id());
        self.set_app_name(notification.app_name());
        self.set_replaces_id(notification.replaces_id());
        self.set_app_icon(notification.app_icon());
        self.set_summary(notification.summary());
        self.set_body(notification.body());
        self.set_actions(notification.actions());
        self.set_hints(notification.hints());
        self.set_expire_timeout(notification.expire_timeout());
        if let Some(dt) = notification.timestamp() {
            self.set_timestamp(dt);
        }
    }

    pub fn from_variant(id: Option<u32>, variant: &glib::Variant, datetime: Option<glib::DateTime>) -> Option<Self> {
        // println!("NVARIANT: {variant:?}");
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
            .map(
                |(
                    app_name,
                    replaces_id,
                    app_icon,
                    summary,
                    body,
                    actions,
                    hints,
                    expire_timeout,
                )| {
                    Self::new(
                        id,
                        app_name,
                        replaces_id,
                        app_icon,
                        summary,
                        body,
                        actions,
                        hints,
                        expire_timeout,
                        datetime,
                    )
                },
            )
    }
    pub fn get_hints(&self) -> NotificationHints {
        self.hints().into()
    }
}

#[derive(Debug)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

pub type NotificationImageData = (i32, i32, i32, bool, i32, i32, Vec<u8>);

#[derive(Debug)]
pub struct NotificationHints {
    pub urgency: Urgency,
    pub desktop_entry: Option<String>,
    pub category: Option<String>,
    pub action_icons: Option<bool>,
    pub image_data: Option<NotificationImageData>,
    pub image_path: Option<String>,
    pub icon_data: Option<NotificationImageData>,
}

impl From<glib::VariantDict> for NotificationHints {
    fn from(dict: glib::VariantDict) -> Self {
        let urgency = dict
            .lookup_value("urgency", None)
            .and_then(|v| v.get::<u8>());
        let urgency = match urgency {
            Some(0) => Urgency::Low,
            Some(1) => Urgency::Normal,
            Some(2) => Urgency::Critical,
            _ => Urgency::Normal, // Default to Normal if not specified
        };
        let desktop_entry = dict
            .lookup_value("desktop-entry", Some(&String::static_variant_type()))
            .and_then(|v| v.get::<String>());

        let category = dict
            .lookup_value("category", Some(&String::static_variant_type()))
            .and_then(|v| v.get::<String>());

        let action_icons = dict
            .lookup_value("action-icons", Some(&bool::static_variant_type()))
            .and_then(|v| v.get::<bool>());

        let image_data = dict
            .lookup_value("image-data", None)
            .and_then(|v| v.get::<NotificationImageData>());

        let image_path = dict
            .lookup_value("image-path", Some(&String::static_variant_type()))
            .and_then(|v| v.get::<String>());

        let icon_data = dict
            .lookup_value("icon-data", None)
            .and_then(|v| v.get::<NotificationImageData>());

        Self {
            urgency,
            desktop_entry,
            category,
            action_icons,
            image_data,
            image_path,
            icon_data: icon_data,
        }
    }
}
