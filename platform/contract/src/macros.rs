//! Two small macros so every wire type carries the same derives.

/// Structs and field carrying enums on the wire. Enum items must add `#[non_exhaustive]`.
macro_rules! wire {
    ($($(#[$meta:meta])* $vis:vis $word:ident $name:ident $body:tt)*) => {
        $(
            #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
            $(#[$meta])*
            $vis $word $name $body
        )*
    };
}

/// An enum of plain words. A word this build does not know reads as `Unknown` and never fails.
macro_rules! words {
    ($(#[$meta:meta])* pub enum $name:ident as $case:literal { $($(#[$vmeta:meta])* $variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(rename_all = $case)]
        #[non_exhaustive]
        $(#[$meta])*
        pub enum $name {
            $($(#[$vmeta])* $variant,)+
            /// A word this build does not know.
            #[serde(other)]
            #[cfg_attr(feature = "schema", schemars(skip))]
            Unknown,
        }
    };
    ($(#[$meta:meta])* pub enum $name:ident { $($body:tt)* }) => {
        words! { $(#[$meta])* pub enum $name as "snake_case" { $($body)* } }
    };
}
