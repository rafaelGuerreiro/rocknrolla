//! The `{Accessor}Trait` + `{Accessor}Impl` boilerplate every domain repeats:
//! a trait giving `ReducerContext` a `.{accessor}()` method, and a thin
//! `Deref<Target = ReducerContext>` wrapper struct so a domain's own
//! `impl {Accessor}Impl` block only has to write its actual methods.
macro_rules! make_service {
    ($accessor:ident) => {
        paste::paste! {
            pub trait [<$accessor:camel Trait>] {
                fn $accessor(&self) -> [<$accessor:camel Impl>]<'_>;
            }

            impl [<$accessor:camel Trait>] for spacetimedb::ReducerContext {
                fn $accessor(&self) -> [<$accessor:camel Impl>]<'_> {
                    [<$accessor:camel Impl>] { ctx: self }
                }
            }

            pub struct [<$accessor:camel Impl>]<'a> {
                ctx: &'a spacetimedb::ReducerContext,
            }

            impl std::ops::Deref for [<$accessor:camel Impl>]<'_> {
                type Target = spacetimedb::ReducerContext;
                fn deref(&self) -> &Self::Target {
                    self.ctx
                }
            }
        }
    };
}

pub(crate) use make_service;
