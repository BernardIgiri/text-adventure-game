use super::Database;

/*
    Omitted fields are in the second bracket and will not be automatically added to the proxy.
*/
#[macro_export]
macro_rules! proxied_entity {
    (
        $proxy:ident,
        $entity:ident,
        $handle:ident {
            $(
                $(#[$meta:meta])*
                $field:ident : $type:ty
            ),* $(,)?
        }, {
            $(
                $(#[$meta_omitted:meta])*
                $field_omitted:ident : $type_omitted:ty
            ),* $(,)?
        }
    ) => {
        $crate::naked_entity!(
            $entity,
            $handle {
                $(
                    $(#[$meta])*
                    $field: $type,
                )*
                $(
                    $(#[$meta_omitted])*
                    $field_omitted: $type_omitted,
                )*
            }
        );
        #[derive(new)]
        pub struct $proxy<'a, T: Database> {
            handle: $handle,
            db: &'a T,
        }
        impl<'a, T: Database> $proxy<'a, T> {
            #[allow(dead_code)]
            pub fn into_handle(self) -> $handle {
                self.into()
            }
        }
        impl<'a, T: Database> $proxy<'a, T> {
            #[allow(dead_code)]
            pub fn handle_clone(&self) -> $handle {
                self.handle.clone()
            }
        }
        impl<'a, T: Database> $proxy<'a, T> {
            $(
                #[allow(dead_code)]
                pub fn $field(&self) -> &'_ $type {
                    &self.handle.0.$field
                }
            )*
        }
        impl<'a, T: Database> From<$proxy<'a, T>> for $handle {
            fn from(proxy: $proxy<'a, T>) -> Self {
                proxy.handle
            }
        }
        impl<T: Database> ToProxy<T> for $handle {
            type Proxy<'a> = $proxy<'a, T> where Self: 'a, T: 'a;

            fn to_proxy<'a>(self, db: &'a T) -> Self::Proxy<'a> {
                $proxy::new(self, db)
            }
        }

        impl<T: Database> ToProxy<T> for Rc<$entity> {
            type Proxy<'a> = $proxy<'a, T> where Self: 'a, T: 'a;

            fn to_proxy<'a>(self, db: &'a T) -> Self::Proxy<'a> {
                let handle = $handle::from(self);
                $proxy::new(handle, db)
            }
        }
    };
}

#[macro_export]
macro_rules! naked_entity {
    (
        $entity:ident,
        $handle:ident {
            $(
                $(#[$meta:meta])*
                $field:ident : $type:ty
            ),* $(,)?
        }
    ) => {
        #[allow(dead_code)]
        #[derive(Getters, Builder, Debug, PartialEq, Eq)]
        pub struct $entity {
            $(
                $(#[$meta])*
                $field: $type,
            )*
        }
        impl From<Rc<$entity>> for $handle {
            fn from(entity: Rc<$entity>) -> Self {
                $handle::new(entity)
            }
        }
        impl From<&Rc<$entity>> for $handle {
            fn from(entity: &Rc<$entity>) -> Self {
                $handle::new(entity.clone())
            }
        }
        #[derive(new, Clone, Debug, PartialEq, Eq)]
        pub struct $handle(Rc<$entity>);
    };
}

pub trait ToProxy<T: Database> {
    type Proxy<'a>
    where
        Self: 'a,
        T: 'a;

    fn to_proxy<'a>(self, db: &'a T) -> Self::Proxy<'a>;
}
