use adw::subclass::prelude::{ObjectSubclassExt, ObjectSubclassIsExt};
use gio::{glib::variant::ToVariant, prelude::{AppInfoExt, FileExt, IconExt}};
use gtk::{
    gdk::{self, prelude::TextureExt}, glib::{
        self, clone,
        object::{IsA, ObjectExt},
        Object,
    }, prelude::{BoxExt, WidgetExt}, CompositeTemplate, IconLookupFlags
};

use crate::notification_server;

mod inner {

    use gtk::prelude::ObjectExt;
    use std::cell::RefCell;

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

        #[template_child(id = "content")]
        pub content_box: TemplateChild<gtk::Box>
    }

    #[template_callbacks]
    impl NotificationDisplay {
        #[template_callback]
        fn on_close(&self) {
            
        }
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

        
        // imp.app_icon.set_icon_name(Some(&notification.app_name()));
        println!("{}", notification.summary());
        imp.title_label.set_label(&notification.summary());
        imp.body_label.set_label(&notification.body());

        let hints = notification.get_hints();

        match hints.urgency {
            notification_server::Urgency::Critical => {
                self.add_css_class("critical");
            }
            _ => {}
        };


        let image = pic_pixbuf_from_hints(hints);

        if let Some(image) = image {
            let texture = gtk::gdk::Texture::for_pixbuf(&image);
            let ratio = texture.width() as f32 / texture.height() as f32;
            let picture = gtk::Picture::builder()
                .paintable(&texture)
                .vexpand(true)
                .hexpand(true)
                .content_fit(gtk::ContentFit::Fill)
                .css_classes(["rounded-sm"])
                .build();
            
            let aspect_frame = gtk::AspectFrame::builder()
                .child(&picture)
                .halign(gtk::Align::Start)
                // .vexpand(true)
                // .width_request(50)
                .height_request(50)
                .ratio(1.0)
                .css_classes(["rounded-sm"])
                .overflow(gtk::Overflow::Hidden)
                .build();
            imp.content_box.prepend(&aspect_frame);
        }
        if let Some(datestr) = notification.timestamp().and_then(|dt| dt.format("%X").ok()) {
            imp.date_label.set_label(&datestr);
        }
    }
}

pub fn pic_pixbuf_from_hints(
    hints: notification_server::NotificationHints,
) -> Option<gdk_pixbuf::Pixbuf> {
    let notification_server::NotificationHints {
        image_data,
        image_path,
        icon_data,
        ..
    } = hints;
    image_data
        .and_then(pixbuf_from_image_bytes)
        .or_else(|| image_path.and_then(|path| gdk_pixbuf::Pixbuf::from_file(path).ok()))
        .or_else(|| icon_data.and_then(pixbuf_from_image_bytes))
}

fn pixbuf_from_image_bytes(
    bytes: notification_server::NotificationImageData,
) -> Option<gdk_pixbuf::Pixbuf> {
    let (width, height, rowstride, has_alpha, bits_per_sample, channels, data) = bytes;
    let data = glib::Bytes::from_owned(data);
    let colorspace = gdk_pixbuf::Colorspace::Rgb;
    Some(gdk_pixbuf::Pixbuf::from_bytes(
        &data,
        colorspace,
        has_alpha,
        bits_per_sample,
        width,
        height,
        rowstride,
    ))
}

fn gicon_from_desktop_entry(entry: impl AsRef<str>) -> Option<gio::Icon> {
    println!("{}", entry.as_ref());
    let info = gio::DesktopAppInfo::new(&format!("{}.desktop", entry.as_ref()))?;
    info.icon()
}