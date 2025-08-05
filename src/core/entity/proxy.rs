use super::database::Lookup;

/*

Database <- entity store and update
Raw <- from disk, is naked
Entity <- runtime variant, is naked
Proxy <- proxy
Id <- typed index into graph

Proxy -> Id
proxy.id()

Id -> Proxy
id.into_proxy(&db)
Proxy::new(id, &db)

Id -> Entity
db.look_up(&id)

No path from Entity to id or proxy

*/

pub trait IntoProxy<'a, Id, DB: Lookup, ProxyT> {
    fn into_proxy(self, db: &'a DB) -> ProxyT;
}

#[macro_export]
macro_rules! define_id {
    ($id:ident) => {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $id(usize);

        impl From<usize> for $id {
            fn from(v: usize) -> Self {
                $id(v)
            }
        }

        impl From<&usize> for $id {
            fn from(v: &usize) -> Self {
                $id(*v)
            }
        }

        impl From<$id> for usize {
            fn from(v: $id) -> Self {
                v.0
            }
        }

        impl From<&$id> for usize {
            fn from(v: &$id) -> Self {
                v.0
            }
        }
    };
}

#[macro_export]
macro_rules! define_id_and_proxy {
    ($id:ident, $proxy:ident) => {
        define_id!($id);
        pub struct $proxy<'a, DB: Lookup> {
            id: $id,
            db: &'a DB,
        }
        impl<'a, DB: Lookup> $proxy<'a, DB> {
            pub const fn new(id: $id, db: &'a DB) -> Self {
                Self { id, db }
            }
            #[allow(dead_code)]
            pub const fn into_id(self) -> $id {
                self.id
            }
            pub fn id(&self) -> $id {
                self.id.clone()
            }
        }

        impl<'a, DB: Lookup> IntoProxy<'a, $id, DB, $proxy<'a, DB>> for $id {
            fn into_proxy(self, db: &'a DB) -> $proxy<'a, DB> {
                $proxy::new(self, db)
            }
        }
    };
}
