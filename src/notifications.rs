use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::{
    glib::{self, clone, object::{IsA, ObjectExt}, Object},
    CompositeTemplate,
};

mod inner {

    use gtk::prelude::ObjectExt;
    use std::cell::RefCell;

    use gtk::glib::{self, derived_properties, Properties};
    use gtk::subclass::prelude::*;

    use super::*;
    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::TimeModule)]
    #[template(resource = "/shell/ui/time.ui")]
    pub struct TimeModule {
        #[template_child(id = "date-label")]
        pub date_label: TemplateChild<gtk::Label>,

        #[template_child(id = "weekday-label")]
        pub weekday_label: TemplateChild<gtk::Label>,
        
        #[property(get, set)]
        pub datetime: RefCell<Option<glib::DateTime>>,
    }

    impl TimeModule {
        fn setup_bindings(&self) {
            self.obj().bind_time_to(&self.date_label.get(), "label", "%x"); 
            self.obj().bind_time_to(&self.weekday_label.get(), "label", "%A"); 
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeModule {
        const NAME: &'static str = "TimeModule";
        type Type = super::TimeModule;
        type ParentType = gtk::Box;

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
    #[derived_properties]
    impl ObjectImpl for TimeModule {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = &self.obj();
            let imp = obj.imp();
            obj.setup_clock().expect("couldnt set datetime");
            self.setup_bindings();
        }
    }
    impl BoxImpl for TimeModule {}
    impl WidgetImpl for TimeModule {}
}

glib::wrapper! {
    pub struct TimeModule(ObjectSubclass<inner::TimeModule>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl TimeModule {
    pub fn new() -> Self {
        let obj = Object::new();
        obj
    }

    pub fn setup_clock(&self) -> Result<(), glib::BoolError> {
        let dt = glib::DateTime::now_local()?;
        self.set_datetime(dt);

        let c = clone!(
            #[strong(rename_to = tm)]
            self,
            move || {
                let Ok(dt) = glib::DateTime::now_local() else {
                    return glib::ControlFlow::Continue;
                };
                tm.set_datetime(dt);
                glib::ControlFlow::Continue
            }
        );
        glib::timeout_add_seconds_local(1, c);
        let imp = self.imp();
        
        Ok(())
    }
    pub fn bind_time_to<W: IsA<Object>>(&self, obj: &W, property: &'static str, fmt: &'static str) {
        let c = move |_, dt: Option<glib::DateTime>| {
            return dt.and_then(|dt| dt.format(&fmt).ok())
        };
        self.bind_property("datetime", obj, property)
        .sync_create()
        .transform_to(c)
        .build();
    }
}
