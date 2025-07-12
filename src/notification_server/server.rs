use crate::notification_server::notification::NotificationItem;
use crate::notification_server::store::IDStore;
use gio::glib::object::{Cast, CastNone};
use gio::glib::property::PropertyGet;
use gio::glib::Variant;
use gio::prelude::ListModelExt;
use gtk::gio::{self};
use gtk::glib::{self};
use std::cell::OnceCell;
use std::ffi::os_str::Display;
use std::fmt::write;
use std::str::FromStr;
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
    connection: OnceCell<gio::DBusConnection>,
}

const NOTIFICATION_DBUS_NAME: &str = "org.freedesktop.Notifications";
const NOTIFICATION_DBUS_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATION_DBUS_INTERFACE: &str = "org.freedesktop.Notifications";
const NOTIFICATION_INTROSPECTION_XML: &str = include_str!("notifications-introspect.xml");

pub enum CloseReason {
    Expired = 1,
    Dismissed = 2,
    Call = 3,
    Undefined = 4,
}

#[derive(Debug)]
pub enum ServerError {
    ConnectionUninitialised,
    GLibError(glib::Error),
    UnspecifiedError(String),
    NoInterfaceInfo,
}

impl From<glib::Error> for ServerError {
    fn from(value: glib::Error) -> Self {
        Self::GLibError(value)
    }
}

impl From<String> for ServerError {
    fn from(value: String) -> Self {
        Self::UnspecifiedError(value)
    }
}
impl From<&str> for ServerError {
    fn from(value: &str) -> Self {
        Self::UnspecifiedError(value.to_string())
    }
}
impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ConnectionUninitialised => "ConnectionError",
            Self::GLibError(_) => "GLibError",
            Self::UnspecifiedError(s) => &format!("UnspecifiedError: {s}"),
            Self::NoInterfaceInfo => "NoInterfaceInfo",
        };

        write!(f, "{s}")
    }
}

impl std::error::Error for ServerError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::GLibError(e) => Some(e),
            _ => None,
        }
    }
}

impl Server {
    pub fn new() -> Self {
        Server {
            store: IDStore::new::<NotificationItem>(),
            connection: OnceCell::new(),
        }
    }

    pub fn get_store(&self) -> IDStore {
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
                Self::register_dbus_interface(&s, &conn).unwrap();
                let _ = s.connection.set(conn);
            },
            |x, y| {
                println!("Name lost {x:?} {y}");
            },
        );
    }
    fn register_dbus_interface(&self, conn: &gio::DBusConnection) -> Result<(), ServerError> {
        let node_info = gio::DBusNodeInfo::for_xml(NOTIFICATION_INTROSPECTION_XML)?;
        let interface_info = node_info
            .interfaces()
            .first()
            .ok_or(ServerError::NoInterfaceInfo)?;

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

        println!("handling insert");
        if replaces_id == 0 {
            println!("trying to push");
            let (id, prev) = self.store.push(notification.clone());
            notification.set_id(id);
            return id;
        }
        
        println!("trying to insert");
        self.store.set(replaces_id, notification.clone());

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

    pub fn send_closed(&self, id: u32, reason: CloseReason) -> Result<(), ServerError> {
        self.send_signal("NotificationClosed", &(id, reason as u32).into())?;
        Ok(())
    }
    fn on_get_capabilities(invocation: gio::DBusMethodInvocation) {
        let arr: Variant = (vec![
            "action-icons",
            "actions",
            "body",
            "body-hyperlinks",
            "persistent",
        ],)
            .into();
        invocation.return_value(Some(&arr));
    }
    fn send_signal(
        &self,
        signal: impl AsRef<str>,
        args: &glib::Variant,
    ) -> Result<(), ServerError> {
        let conn = self
            .connection
            .get()
            .ok_or(ServerError::ConnectionUninitialised)?;
        conn.emit_signal(
            Some(NOTIFICATION_DBUS_NAME),
            NOTIFICATION_DBUS_PATH,
            NOTIFICATION_DBUS_INTERFACE,
            signal.as_ref(),
            Some(args),
        )?;
        Ok(())
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

        let prev = self.store.remove(id);

        if prev.is_none() {
            invocation.return_error(
                gio::DBusError::Failed,
                &format!("notification with id {id} not found"),
            );
            return;
        };

        invocation.return_value(None);

        if let Err(e) = self.send_closed(id, CloseReason::Call) {
            eprintln!("Error occured sending close signal for notification: {e}")
        };
    }
}
