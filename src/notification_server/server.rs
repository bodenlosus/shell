use crate::notification_server::notification::NotificationItem;
use gio::glib::object::{Cast, CastNone};
use gio::glib::property::PropertyGet;
use gio::glib::Variant;
use gio::prelude::ListModelExt;
use gtk::gio::{self};
use gtk::glib::{self};
use std::{
    error::Error,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

struct NotifyParams(
    String,
    u32,
    String,
    String,
    String,
    Vec<String>,
    glib::VariantDict,
    i32,
);

#[derive(Clone)]
pub struct Server {
    store: gio::ListStore,
    next_id: Arc<AtomicU32>,
}

const NOTIFICATION_DBUS_NAME: &str = "org.freedesktop.Notifications";
const NOTIFICATION_DBUS_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATION_DBUS_INTERFACE: &str = "org.freedesktop.Notifications";
const NOTIFICATION_INTROSPECTION_XML: &str = include_str!("notifications-introspect.xml");

impl Server {
    pub fn new() -> Self {
        Server {
            store: gio::ListStore::new::<NotificationItem>(),
            next_id: Arc::new(0.into()),
        }
    }

    pub fn get_store(&self) -> gio::ListStore {
        self.store.clone()
    }

    pub fn connect_to_dbus(&self) {
        let s = self.clone();
        gio::bus_own_name(
            gio::BusType::Session,
            NOTIFICATION_DBUS_NAME,
            gio::BusNameOwnerFlags::NONE,
            move |_, _| {
                println!("bus aquired");
            },
            move |conn, name| {
                println!("Name acquired {conn:?} {name}");
                Self::register_dbus_interace(&s, &conn).unwrap();
            },
            |x, y| {
                println!("Name lost {x:?} {y}");
            },
        );
    }
    fn next_notification_id(&self) -> u32 {
        loop {
            let current = self.next_id.load(Ordering::Relaxed);
            let new = current.wrapping_add(1).max(1);

            if let Ok(_) =
                self.next_id
                    .compare_exchange(current, new, Ordering::Relaxed, Ordering::Relaxed)
            {
                return new;
            }
        }
    }
    fn register_dbus_interace(&self, conn: &gio::DBusConnection) -> Result<(), Box<dyn Error>> {
        let node_info = gio::DBusNodeInfo::for_xml(NOTIFICATION_INTROSPECTION_XML)?;
        let interface_info = node_info
            .interfaces()
            .first()
            .ok_or("could not retrieve interface info")?;

        let s = self.clone();
        conn.register_object(NOTIFICATION_DBUS_PATH, interface_info)
            .method_call(
                move |_connection,
                      _sender,
                      _object_path,
                      _interface_name,
                      method_name,
                      parameters,
                      invocation| {
                    let method_name = method_name.to_string();
                    let s = s.clone();
                    glib::spawn_future_local(async move {
                        Self::handle_method_call(&s, &method_name, invocation, parameters).await;
                    });
                },
            )
            .build()?;

        Ok(())
    }
    async fn handle_method_call(
        &self,
        method_name: &str,
        invocation: gio::DBusMethodInvocation,
        parameters: glib::Variant,
    ) {
        println!("Method Name: {method_name}");
        match method_name.as_ref() {
            "GetServerInformation" => {
                Self::on_get_server_info(invocation);
            }
            "Notify" => {
                self.on_notify(&parameters, invocation).await;
            }
            "GetCapabilities" => {
                Self::on_get_capabilities(invocation);
            }
            "CloseNotification" => {
                self.on_close_notification(&parameters, invocation).await;
            }
            &_ => {
                invocation.return_error(
                    gio::DBusError::UnknownMethod,
                    &format!("Method {method_name} is not known to server"),
                );
            }
        };
    }
    fn on_get_server_info(invocation: gio::DBusMethodInvocation) {
        let info = ("ShellNotificationServer", "Shell", "1.0", "1.2");
        invocation.return_value(Some(&info.into()));
    }

    fn handle_insert_notification(&self, notification: &NotificationItem) -> u32 {
        let replaces_id = notification.replaces_id();
        // replaces id starts at 1 -> n_items == replaces_id -> last item
        if replaces_id == 0 {
            let id = self.next_notification_id();
            notification.set_id(id);
            self.store.append(notification);
            return id;
        }

        println!("{replaces_id}");

        let Some(old_store_id) = self
            .store
            .find_with_equal_func(|obj| Self::find_id(obj, replaces_id))
        else {
            let id = self.next_notification_id();
            notification.set_id(id);
            self.store.append(notification);
            return id;
        };

        self.store.splice(old_store_id, 1, &[notification.clone()]);


        replaces_id
    }

    async fn on_notify(&self, parameters: &glib::Variant, invocation: gio::DBusMethodInvocation) {
        let dt = glib::DateTime::now_local().ok();
        match NotificationItem::from_variant(None, parameters, dt) {
            Some(notification) => {
                let id = self.handle_insert_notification(&notification);
                invocation.return_value(Some(&(id,).into()));
            }
            None => {
                invocation.return_error(
                    gio::DBusError::InvalidArgs,
                    "Could not parse notification parameters",
                );
            }
        }
    }
    fn on_get_capabilities(invocation: gio::DBusMethodInvocation) {
        let arr: Variant = (vec!["actions", "body", "persistent"],).into();
        invocation.return_value(Some(&arr));
    }
    async fn on_close_notification(
        &self,
        parameters: &glib::Variant,
        invocation: gio::DBusMethodInvocation,
    ) {
        let Some(id) = parameters.get::<u32>() else {
            invocation.return_error(gio::DBusError::InvalidArgs, "Invalid Notification ID");
            return;
        };

        let Some(id) = self
            .store
            .find_with_equal_func(|obj| Self::find_id(obj, id))
        else {
            invocation.return_error(
                gio::DBusError::Failed,
                &format!("notification with id {id} not found"),
            );
            return;
        };

        self.store.remove(id);
        invocation.return_value(None);
    }

    fn find_id(obj: &glib::Object, id: u32) -> bool {
        obj.downcast_ref::<NotificationItem>()
            .map_or(false, |n| n.id() == id)
    }
}
