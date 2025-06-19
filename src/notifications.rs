use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::property::PropertySet;
use gtk::glib::{self, clone, Object, WeakRef};
use gtk::{CompositeTemplate, Label};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::notification::Notification;
pub mod server {
    use async_channel::{unbounded, Receiver, Sender};
    use gtk::{
        gio::{self, prelude::ListModelExt},
        glib::{self, object::ObjectExt, variant::FromVariant, Boxed},
    };
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        error::Error,
        ops::Not,
        rc::Rc,
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc, Mutex,
        },
    };

    use crate::notification::Notification;


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
            let (sender, reciever) = unbounded();
            Server {
                store: gio::ListStore::new::<Notification>(),
                next_id: Arc::new(0.into()),
            }
        }

        pub fn connect_to_dbus(self) {
            let s = Arc::new(self);
            gio::bus_own_name(
                gio::BusType::Session,
                NOTIFICATION_DBUS_NAME,
                gio::BusNameOwnerFlags::NONE,
                move |_, _| {
                    println!("bus aquired");
                },
                move |conn, name| {
                    println!("Name acquired {conn:?} {name}");
                    let s = s.clone();
                    Self::register_dbus_interace(s, &conn).unwrap();
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

                if let Ok(_) = self.next_id.compare_exchange(
                    current,
                    new,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    return new;
                }
            }
        }
        fn register_dbus_interace(
            s: Arc<Self>,
            conn: &gio::DBusConnection,
        ) -> Result<(), Box<dyn Error>> {
            let node_info = gio::DBusNodeInfo::for_xml(NOTIFICATION_INTROSPECTION_XML)?;
            let interface_info = node_info
                .interfaces()
                .first()
                .ok_or("could not retrieve interface info")?;

            conn.register_object(NOTIFICATION_DBUS_PATH, interface_info)
                .method_call(
                    move |_connection,
                          _sender,
                          _object_path,
                          _interface_name,
                          method_name,
                          parameters,
                          invocation| {
                        println!("Recieved method call: {} {:#?}", method_name, parameters);
                        let method_name = method_name.to_string();
                        let s = s.clone();
                        glib::spawn_future_local(async move {
                            Self::handle_method_call(&s, &method_name, invocation, parameters)
                                .await;
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

        async fn on_notify(
            &self,
            parameters: &glib::Variant,
            invocation: gio::DBusMethodInvocation,
        ) {
            let id = self.next_notification_id();

            match Notification::from_variant(id, parameters) {
                Some(notification) => {
                    self.store.append(&notification);
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
            invocation.return_value(Some(&vec!["actions", "body", "persistent"].into()));
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

            
            

            if let Err(e) = fut.await {
                invocation.return_error(
                    gio::DBusError::Failed,
                    &format!("Failed to send close request: {e}"),
                );
            } else {
                invocation.return_value(None);
            }
        }
    }
}
