#[macro_export]
macro_rules! merge_modules {
    (mod $ident:ident { $($module:path;)* }) => {
        fn $ident<T>() -> $crate::module::Module<T> {
            let mut module = $crate::module::Module::new();

            $(
                module.merge_module($module());
            )*

            module
        }
    };

    (pub mod $ident:ident { $($module:path;)* }) => {
        pub fn $ident<T>() -> $crate::module::Module<T> {
            let mut module = $crate::module::Module::new();

            $(
                module.merge_module($module());
            )*

            module
        }
    };
}

#[macro_export]
macro_rules! def_module {
    (mod $ident:ident { $($items:tt)* }) => {
        fn $ident<T>() -> $crate::module::Module<T> {
            let mut module = $crate::module::Module::new();

            module_items!($($items)*)(&mut module);

            module
        }
    };

    (pub mod $ident:ident { $($items:tt)* }) => {
        pub fn $ident<T>() -> $crate::module::Module<T> {
            let mut module = $crate::module::Module::new();

            module_items!($($items)*)(&mut module);

            module
        }
    };
}

#[macro_export]
macro_rules! module_items {
    (mod $ident:ident { $($items:tt)* } $($rest:tt)*) => {
        |module: &mut $crate::module::Module<_>| {
            let sub_module = {
                let mut module = $crate::module::Module::new();

                module_items!($($items)*)(&mut module);

                module
            };

            module.register_sub_module(stringify!($ident), sub_module);

            module_items!($($rest)*)(module);
        }
    };

    (fn $ident:literal ($($param_ident:tt : $param_ty:ty),*) $block:block $($rest:tt)*) => {
        |module: &mut $crate::module::Module<_>| {
            module.register_fn($ident, #[allow(unused_mut)]|$(mut $param_ident: $param_ty),*| $block).unwrap();

            module_items!($($rest)*)(module);
        }
    };

    () => {
        |_module: &mut $crate::module::Module<_>| {}
    };
}
