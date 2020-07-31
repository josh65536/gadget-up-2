//! Specialized for the case where every field
//! is 1 bit long, and bitwise operations are needed

/// Construct a bitfield where every field is 1 bit.
/// Includes some bitwise operators for convenience.
#[macro_export]
macro_rules! bitfield {
    ($(#[$attr:meta])* pub struct $name:ident($prim:ident) { $($fields:tt)* }) => {
        bitfield!(decl $([$attr])* pub struct $name as $prim);

        bitfield!{impl $name as $prim { $($fields)* }}

        bitfield!{bit_ops $name}
    };

    (decl $([$attr:meta])* pub struct $name:ident as $prim:ident) => {
        $(#[$attr])*
        pub struct $name($prim);
    };

    (impl $name:ident as $prim:ident { $($fields:tt)* }) => {
        impl $name {
            #[allow(dead_code)]
            pub fn new() -> Self {
                Self(0)
            }

            #[allow(dead_code)]
            pub fn zero() -> Self {
                Self(0)
            }

            #[allow(dead_code)]
            pub fn is_zero(self) -> bool {
                self.0 == 0
            }

            bitfield!{fields $name as $prim { $($fields)* }}
        }
    };

    (fields $name:ident as $prim:ident { $field:ident, $get:ident, $set:ident: $bit:literal, $($rest:tt)* }) => {
        bitfield!{field $name as $prim { $field, $get, $set: $bit }}

        bitfield!{fields $name as $prim { $($rest)* }}
    };

    (fields $name:ident as $prim:ident {} ) => {};

    (field $name:ident as $prim:ident { $field:ident, $get:ident, $set:ident: $bit:literal }) => {
        #[allow(dead_code)]
        pub fn $field(self) -> Self {
            Self(self.0 | ((1 as $prim) << $bit))
        }

        #[allow(dead_code)]
        pub fn $get(self) -> bool {
            (self.0 >> $bit & 1) != 0
        }

        #[allow(dead_code)]
        pub fn $set(&mut self, value: bool) {
            self.0 = self.0 & !((1 as $prim) << $bit) | ((value as $prim) << $bit)
        }
    };

    (bit_ops $name:ident) => {
        impl std::ops::BitAnd for $name {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self {
                $name(self.0 & rhs.0)
            }
        }

        impl std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 = self.0 & rhs.0
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self {
                $name(self.0 | rhs.0)
            }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 = self.0 | rhs.0
            }
        }

        impl std::ops::BitXor for $name {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self {
                $name(self.0 ^ rhs.0)
            }
        }

        impl std::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 = self.0 ^ rhs.0
            }
        }
    };
}
