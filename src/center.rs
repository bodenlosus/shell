use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::property::PropertySet;
use gtk::glib::{self, clone, Object, WeakRef};
use gtk::{CompositeTemplate, Label};
use std::time::{SystemTime, Duration, UNIX_EPOCH};

mod inner {

    use adw::subclass::{bin::BinImpl, prelude::ObjectImplExt};
    use gtk::{
        subclass::{prelude::*, widget::WidgetImpl},
    };

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/shell/ui/center.ui")]
    pub struct Center {
        #[template_child(id="time-label")]
        pub time_label: TemplateChild<gtk::Label>,
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
            super::Clock::attach_to_label(&obj.imp().time_label);

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

// TODO: use systime
struct Clock;

impl Clock {
    pub fn new() -> Self {
        Self
    }

    pub fn attach_to_label(label: &gtk::Label) {
        Self::update_display(label);

        glib::timeout_add_local(Duration::from_secs(1), clone!(
            #[weak] label,
            #[upgrade_or]
            glib::ControlFlow::Continue,
            move || {
                Self::update_display(&label);
                glib::ControlFlow::Continue
            }
        ));
    }

    pub fn update_display(label: &gtk::Label) {
        let Ok(now) = glib::DateTime::now_local() else {return;};

        let Ok(time_str) = now.format("%x | %X") else {return;};

        label.set_text(&time_str);
    }
}