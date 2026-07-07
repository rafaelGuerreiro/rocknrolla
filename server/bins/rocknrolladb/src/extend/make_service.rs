//! The `{Domain}ReducerContext` + `{Domain}Services` boilerplate every domain
//! repeats: a trait giving `ReducerContext` a `.{domain}_services()` method,
//! and a thin `Deref<Target = ReducerContext>` wrapper struct so a domain's
//! own `impl {Domain}Services` block only has to write its actual methods.
macro_rules! make_service {
    ($trait_name:ident, $accessor:ident, $services:ident) => {
        pub trait $trait_name {
            fn $accessor(&self) -> $services<'_>;
        }

        impl $trait_name for spacetimedb::ReducerContext {
            fn $accessor(&self) -> $services<'_> {
                $services { ctx: self }
            }
        }

        pub struct $services<'a> {
            ctx: &'a spacetimedb::ReducerContext,
        }

        impl std::ops::Deref for $services<'_> {
            type Target = spacetimedb::ReducerContext;
            fn deref(&self) -> &Self::Target {
                self.ctx
            }
        }
    };
}

pub(crate) use make_service;
