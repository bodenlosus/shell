use adw::subclass::prelude::ObjectSubclassIsExt;
use gdk_pixbuf::Pixbuf;
use gio::{
    glib::{object::IsA, property::PropertyGet, variant::StaticVariantType, UriFlags},
    prelude::FileExt,
};
use gtk::glib::{self, Object};

mod inner {

    use super::*;
    use crate::notification_server::server::ServerError;
    use crate::notification_server::NotificationItem;
    use adw::subclass::prelude::ObjectSubclassExt;
    use gio::glib::object::IsA;
    use gio::glib::subclass::types::ObjectSubclass;
    use gio::glib::subclass::{Signal, SignalId};
    use gio::glib::types::StaticType;
    use gio::prelude::ListModelExt;
    use gio::subclass::prelude::ListModelImpl;
    use glib::prelude::ObjectExt;
    use glib::subclass::object::{DerivedObjectProperties, ObjectImpl, ObjectImplExt};
    use glib::{self, Properties};
    use std::cell::{Cell, OnceCell, RefCell};
    use std::collections::{HashMap, HashSet};
    use std::hash::Hash;
    use std::num::{NonZero, NonZeroU32};
    use std::ops::Div;
    use std::rc::Rc;
    use thiserror::Error;

    trait Key: Eq + Hash + Copy {
        const START: Self;
        fn next(&mut self) -> Self;
    }

    macro_rules! implement_next_for_nonzero {
        ($t:ty) => {
            impl Key for $t {
                const START: Self = Self::new(1).unwrap();
                fn next(&mut self) -> Self {
                    let curr = *self;
                    *self = self.checked_add(1).unwrap_or(Self::new(1).unwrap());
                    curr
                }
            }
        };
    }

    implement_next_for_nonzero!(NonZeroU32);

    #[derive(Clone)]
    struct Node<K, T: Clone> {
        prev: Option<K>,
        next: Option<K>,
        value: T,
    }

    #[derive(Error, Debug)]
    enum StoreError {
        #[error("store is malformed")]
        Malformed,
        #[error("previous node not found")]
        PrevNodeNotFound,
        #[error("next node not found")]
        NextNodeNotFound,
    }

    struct Store<K: Key, T: Clone> {
        items: HashMap<K, Node<K, T>>,
        free_ids: Vec<K>,
        id: K,
        head: Option<K>,
        tail: Option<K>,
        cache: Option<(usize, K)>,
    }

    impl<K: Key, T: Clone> Store<K, T> {
        pub fn new() -> Self {
            Self {
                items: HashMap::new(),
                free_ids: Vec::new(),
                id: K::START,
                head: None,
                tail: None,
                cache: None,
            }
        }
        pub fn next_id(&mut self) -> K {
            self.free_ids.pop().unwrap_or_else(|| self.id.next())
        }
        fn initialize(&mut self, item: T, id: K) {
            self.items.insert(
                id,
                Node {
                    prev: None,
                    next: None,
                    value: item,
                },
            );
            self.head = Some(id);
            self.tail = Some(id);
        }
        pub fn push_tail(&mut self, item: T) -> K {
            let id = self.next_id();
            let (Some(_), Some(tail)) = (self.head, self.tail) else {
                // TODO: Handle rewrap
                self.initialize(item, id);
                return id;
            };

            self.items.insert(
                id,
                Node {
                    prev: Some(tail),
                    next: None,
                    value: item,
                },
            );
            self.tail = Some(id);
            id
        }
        pub fn push_head(&mut self, item: T) -> K {
            let id = self.next_id();
            let (Some(head), Some(_)) = (self.head, self.tail) else {
                // TODO: Handle rewrap
                self.initialize(item, id);
                return id;
            };
            self.items.insert(
                id,
                Node {
                    prev: None,
                    next: Some(head),
                    value: item,
                },
            );
            self.head = Some(id);
            id
        }
        pub fn replace(&mut self, id: &K, item: T) -> Option<T> {
            let node = self.items.get_mut(id)?;
            let prev = std::mem::replace(&mut node.value, item);
            Some(prev)
        }

        pub fn remove(&mut self, id: &K) -> Result<Option<T>, StoreError> {
            let Some(node) = self.items.remove(id) else {
                return Ok(None);
            };

            match node {
                Node {
                    next: Some(next),
                    prev: Some(prev),
                    ..
                } => {
                    let [pn, nn] = self.items.get_disjoint_mut([&prev, &next]);
                    let pn = pn.ok_or(StoreError::PrevNodeNotFound)?;
                    let nn = nn.ok_or(StoreError::NextNodeNotFound)?;

                    pn.next = Some(next);
                    nn.prev = Some(prev);
                }
                // Node is tail
                Node {
                    prev: Some(prev),
                    next: None,
                    ..
                } => {
                    if Some(*id) != self.tail {
                        return Err(StoreError::Malformed);
                    }

                    let pn = self
                        .items
                        .get_mut(&prev)
                        .ok_or(StoreError::PrevNodeNotFound)?;
                    pn.next = None;

                    self.tail = Some(prev);
                }
                // node is head
                Node {
                    prev: None,
                    next: Some(next),
                    ..
                } => {
                    if Some(*id) != self.head {
                        return Err(StoreError::Malformed);
                    }

                    let nn = self
                        .items
                        .get_mut(&next)
                        .ok_or(StoreError::NextNodeNotFound)?;
                    nn.prev = None;

                    self.head = Some(next);
                }

                Node {
                    prev: None,
                    next: None,
                    ..
                } => {
                    return Err(StoreError::Malformed);
                }
            };

            self.free_ids.push(*id);

            Ok(Some(node.value))
        }

        fn try_cache_lookup(&self, pos: usize) -> Option<&Node<K, T>> {
            let (cache_pos, cache_id) = self.cache?;
            let cached_node = self.items.get(&cache_id);

            if pos == cache_pos {
                return cached_node;
            }
            if pos > cache_pos {
                return self.items.get(&cached_node?.next?);
            }
            if pos < cache_pos {
                return self.items.get(&cached_node?.prev?);
            }

            None
        }

        pub fn nth_item(&self, pos: usize) -> Option<&T> {
            //TODO: use caching adapted to the lookup pattern of the SelectionModel, use end if nearer

            if let Some(node) = self.try_cache_lookup(pos) {
                return Some(&node.value);
            }

            let len = self.items.len();

            if pos >= len {
                return None;
            }

            if pos < len / 2 {
                return self.nth_item_from_head(pos);
            }

            return self.nth_item_from_tail(pos, len);
        }
        fn nth_item_from_head(&self, pos: usize) -> Option<&T> {
            let mut id = self.head?;

            for _ in 0..pos {
                id = self.items.get(&id).and_then(|n| n.next)?;
            }

            self.items.get(&id).map(|n| &n.value)
        }
        fn nth_item_from_tail(&self, pos: usize, len: usize) -> Option<&T> {
            let mut id = self.tail?;

            for _ in pos..len {
                id = self.items.get(&id).and_then(|n| n.next)?;
            }

            self.items.get(&id).map(|n| &n.value)
        }
    }

    // #[derive(Properties)]
    // #[properties(wrapper_type = super::IDStore)]
    // pub struct IDStore {
    //     items: RefCell<LinkedList<Object>>,
    //     item_type: Cell<glib::Type>,
    //     id_to_lli: RefCell<HashMap<>>
    // }

    // #[glib::object_subclass]
    // impl ObjectSubclass for IDStore {
    //     const NAME: &'static str = "IDStore";
    //     type Type = super::IDStore;
    //     type Interfaces = (gio::ListModel,);
    //     type ParentType = Object;

    //     fn new() -> Self {
    //         Self {
    //             items: RefCell::new(LinkedList::new()),

    //             item_type: Cell::new(glib::Type::OBJECT)
    //         }
    //     }
    // }
    // // #[glib::derived_properties]
    // impl ObjectImpl for IDStore {

    //     fn constructed(&self) {
    //         self.parent_constructed();
    //     }
    // }

    // impl ListModelImpl for IDStore {
    //     fn item(&self, position: u32) -> Option<Object> {
    //         self.get_obj_by_pos(position as usize)
    //     }
    //     fn item_type(&self) -> glib::Type {
    //         self.item_type.get()
    //     }
    //     fn n_items(&self) -> u32 {
    //         let items = self.items.borrow();
    //         items.len() as u32
    //     }
    // }

    // impl IDStore {
    //     pub fn push(&self, obj: Object) -> (u32, Option<Object>) {
    //         let mut items = self.items.borrow_mut();
    //         let id = items.push_front(obj);
    //         let x = id.data
    //     }

    //     fn notify_items_changed(&self, position: u32, removed: u32, added: u32) {
    //         println!("attempting items changed");
    //         println!("added: {added}, deleted: {removed} at {position}");
    //         self.obj().items_changed(position, removed, added);
    //         println!("items_changed succeded");
    //     }

    //     pub fn remove(&self, position: usize) -> Option<Object> {
    //         let id  = self.get_id_by_pos(position);
    //         let mut items = self.items.borrow_mut();
    //         let prev = items.remove(&id);
    //         drop(items);

    //         if prev.is_some() {

    //             self.notify_items_changed(id, 1, 1);
    //         }

    //         prev
    //     }

    //     pub fn set(&self, id: u32, obj: Object) -> Option<Object> {
    //         let mut items = self.items.borrow_mut();
    //         let prev = items.insert(id, obj.into());
    //         println!("items: {:?}", items);
    //         drop(items);
    //         let removed = prev.is_some() as u32;
    //         self.notify_items_changed(id, removed, 1);
    //         prev
    //     }
    //     pub fn set_type(&self, t: glib::Type) {
    //         self.item_type.set(t);
    //     }
    //     pub fn mep(&self) {
    //         println!("a");
    //     }
    // }
}

// glib::wrapper! {
//     pub struct IDStore(ObjectSubclass<inner::IDStore>)
//     @implements gio::ListModel;
// }

// impl IDStore {
//     pub fn new<T: IsA<glib::Object>>() -> Self {
//         let obj: IDStore = Object::new();
//         obj.imp().set_type(T::static_type());
//         obj
//     }

//     pub fn push(&self, obj: impl IsA<Object>) -> (u32, Option<Object>) {
//         let obj: glib::Object = obj.into();
//         self.imp().push(obj)
//     }
//     pub fn set(&self, id: u32, obj: impl IsA<Object>) -> Option<Object> {
//         self.imp().set(id, obj.into())
//     }
//     pub fn remove(&self, id: u32) -> Option<Object> {
//         self.imp().remove(id)
//     }
// }
