//! # packed_bits
//!
//! Memory-efficient bit packing library. Define a `packed_bits` struct that
//! stores multiple fields in a single integer, using only as many bits as each
//! field needs.
//!
//! # Usage
//!
//! ```rust
//! use packed_bits::packed_bits;
//!
//! // LC-3 ADD instruction (16-bit). Fields map directly to the ISA layout:
//! //   ADD DR, SR1, SR2   -> 0001 DR SR1 0 000 SR2 00
//! //   ADD DR, SR1, imm5  -> 0001 DR SR1 1 imm5
//! packed_bits! {
//!     struct Lc3Add(u16) {
//!         value: 5,   // SR2 (register mode) or imm5 (immediate mode)
//!         imm: 1,     // 0 = register mode, 1 = immediate mode
//!         sr1: 3,
//!         dr: 3,
//!         opcode: 4,  // 0b0001 for ADD
//!     }
//! }
//!
//! // ADD R2, R1, R3 (register mode) -> 0x144C
//! let add_reg = Lc3Add::from(0x144C);
//! assert_eq!((0b01100, 0, 1, 2, 0b0001),
//!     (add_reg.value(), add_reg.imm(), add_reg.sr1(), add_reg.dr(), add_reg.opcode()));
//!
//! // ADD R0, R1, #5 (immediate mode) -> 0x1065
//! let add_imm = Lc3Add::new(0b00101, 1, 1, 0, 0b0001);
//! assert_eq!(0x1065, add_imm.get_raw());
//!
//! // read a single field
//! assert_eq!(0b00101, add_imm.value());
//!
//! // update a field (chainable, returns &mut Self)
//! let mut add = add_imm;
//! add.set_dr(1).set_sr1(2);
//! assert_eq!(1, add.dr());
//!
//! // raw bit access
//! let mut flags = add;
//! flags.set_bit(5, true).toggle_bit(9);
//! assert!(flags.get_bit(5));
//! flags.clear_bit(5);
//! assert!(!flags.get_bit(5));
//!
//! // const-compatible creation
//! const ADD_R0_R1_5: Lc3Add = Lc3Add::new(0b00101, 1, 1, 0, 0b0001);
//! assert_eq!(0x1065, ADD_R0_R1_5.get_raw());
//! ```
//!
//! # Typed fields
//!
//! Fields can also have a type, backed by the [`PackedField`] trait. Fieldless
//! enums get an implementation via the `derive` feature:
//!
//! ```rust
//! # #[cfg(feature = "derive")]
//! # mod example {
//! use packed_bits::{packed_bits, PackedField};
//!
//! #[derive(PackedField, Debug, Clone, Copy, PartialEq, Eq)]
//! enum Color {
//!     Red = 0,
//!     Green = 1,
//!     Blue = 2,
//! }
//!
//! packed_bits! {
//!     struct Pixel(u16) {
//!         color: Color = 2,
//!         alpha: u8 = 8,
//!     }
//! }
//!
//! pub fn run() {
//!     let pixel = Pixel::new(Color::Blue, 200);
//!     assert_eq!(Some(Color::Blue), pixel.color());
//!     assert_eq!(2, pixel.color_raw());
//!     assert_eq!(Some(200), pixel.alpha());
//!
//!     // construction from raw bits fails if any field has no valid value
//!     assert!(Pixel::try_from(3).is_err());
//! }
//! # }
//! ```
//!
//! # no_std
//!
//! `packed_bits` is `no_std` compatible. Disable default features to build
//! without `std`:
//!
//! ```toml
//! packed_bits = { version = "0.1", default-features = false }
//! ```
//!
//! # Example: memory savings
//!
//! Eight boolean flags fit in a single byte instead of eight:
//!
//! Without packing: `fin` + `syn` + `ack` + ... (1 byte each) = 8 bytes
//! With packing: everything fits in just 1 byte!
//!
//! # Macro parameters
//!
//! - `name`: The name of the generated struct
//! - `storage`: The underlying data type/size (`u8`, `u16`, `u32`, `u64`)
//! - `field`: Field name (will also be used as a getter method)
//! - `bits`: Number of bits allocated for this field
//!
//! # Important notes
//! - Make sure your bit counts add up to fit in your storage type
//! - u16 can hold 16 bits total, u32 can hold 32 bits, etc.
//! - Each field gets a method with the same name to read its value, plus `set_<field>` to update it
//! - Values are stored from lowest bits to highest bits in declaration order
//! - Passing an out-of-range value to `new`/setters panics; use `set_bit`/`get_bit` for raw bit access
#![cfg_attr(not(feature = "std"), no_std)]

extern crate self as packed_bits;

pub use paste::paste;
pub use static_assertions;

#[cfg(feature = "derive")]
pub use packed_bits_derive::PackedField;

/// A type that can be packed into a bit field.
///
/// Implemented automatically by the `#[derive(packed_bits::PackedField)]`
/// derive macro for fieldless enums, and by blanket impls for `u8`, `u16`,
/// `u32`, and `u64`. Advanced users can implement this manually to pack
/// custom types.
///
/// # Safety
///
/// `unpack_unchecked` is safe to call only when `raw` is a value that `unpack`
/// would return `Some` for.
pub trait PackedField {
    /// The raw integer type this field is stored as.
    type Raw;

    /// Converts a value into its raw integer representation.
    fn pack(self) -> Self::Raw;

    /// Converts a raw integer back into a value, returning `None` if the raw
    /// value has no valid representation.
    fn unpack(raw: Self::Raw) -> Option<Self>
    where
        Self: Sized;

    /// Converts a raw integer back into a value without validating it.
    ///
    /// # Safety
    ///
    /// `raw` must be a value that `unpack` would return `Some` for.
    unsafe fn unpack_unchecked(raw: Self::Raw) -> Self;
}

macro_rules! impl_packed_field_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl PackedField for $t {
                type Raw = $t;

                fn pack(self) -> Self::Raw {
                    self
                }

                fn unpack(raw: Self::Raw) -> Option<Self> {
                    Some(raw)
                }

                unsafe fn unpack_unchecked(raw: Self::Raw) -> Self {
                    raw
                }
            }
        )*
    };
}

impl_packed_field_primitive!(u8, u16, u32, u64);

/// Error returned when a value exceeds the capacity of a packed field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldError {
    /// The name of the field that overflowed.
    pub field: &'static str,
    /// The value that was rejected.
    pub value: u64,
    /// The maximum representable value for the field.
    pub max: u64,
}

impl core::fmt::Display for FieldError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "value {} exceeds capacity {} for field `{}`",
            self.value, self.max, self.field
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FieldError {}

/// Defines a packed struct that stores multiple fields in a single integer.
///
/// See the [crate documentation](crate) for usage examples.
///
/// # Parameters
/// - `name`: The name of the generated struct
/// - `storage`: The underlying data type/size (`u8`, `u16`, `u32`, `u64`)
/// - `field`: Field name (will also be used as a getter method)
/// - `bits`: Number of bits allocated for this field
///
/// # Important notes
/// - Make sure your bit counts add up to fit in your storage type
/// - u16 can hold 16 bits total, u32 can hold 32 bits, etc.
/// - Each field gets a method with the same name to read its value, plus `set_<field>` to update it
/// - Values are stored from lowest bits to highest bits in declaration order
/// - Passing an out-of-range value to `new`/setters panics; use `set_bit`/`get_bit` for raw bit access
#[macro_export]
macro_rules! packed_bits {
    // typed fields. Each field declares its own type and bit width.
    // Fields implement the `PackedField` trait (enums via derive, primitives
    // via blanket impls). Listed before the bare-field arm so `color: Color = 2`
    // matches here instead of being parsed as a bare expression.
    (
        struct $name:ident($storage:ty) {
            $(
                $field:ident: $ty:ty = $bits:expr,
            )*
        }
    ) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub struct $name($storage);

        impl $name {
            /// Creates a new packed struct from its fields.
            ///
            /// Panics if any field value exceeds its allocated bit width.
            pub fn new($($field: $ty),*) -> Self {
                $crate::static_assertions::const_assert!(($($bits +)* 0) <= core::mem::size_of::<$storage>() * 8);

                let mut packed = 0;
                packed_bits!(@impl_typed_new $storage, [$($field: $ty: $bits),*], 0, packed);
                Self(packed)
            }

            packed_bits!(@impl_typed_getters $storage, [$($field: $ty: $bits),*]);
            packed_bits!(@impl_typed_setters $storage, [$($field: $ty: $bits),*]);
        }

        impl $name {
            /// Returns the raw underlying storage value.
            pub const fn get_raw(&self) -> $storage {
                self.0
            }

            /// Overwrites the raw underlying storage value.
            pub const fn set_raw(&mut self, value: $storage) {
                self.0 = value;
            }

            /// Constructs from a raw underlying storage value.
            pub const fn from_raw(value: $storage) -> Self {
                Self(value)
            }

            packed_bits!(@impl_bit_ops_methods $storage);
        }

        impl core::convert::TryFrom<$storage> for $name {
            type Error = $crate::FieldError;

            fn try_from(value: $storage) -> Result<Self, Self::Error> {
                packed_bits!(@impl_typed_try_from $storage, value, [$($field: $ty: $bits),*]);
                Ok(Self(value))
            }
        }

    };

    // bare fields. The field type is the storage type (`day: 5`).
    (
       struct $name:ident($storage:ty) {
            $(
                $field:ident: $bits:expr,
            )*
        }
    ) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub struct $name($storage);

        impl $name {
            /// Creates a new packed struct from its fields.
            ///
            /// Panics if any field value exceeds its allocated bit width.
            pub const fn new($($field: $storage),*) -> Self {
                $crate::static_assertions::const_assert!(($($bits +)* 0) <= core::mem::size_of::<$storage>() * 8);

                packed_bits!(@impl_new_asserts [$($field: $bits),*]);
                packed_bits!(@impl_new_pack [$($field: $bits),*], 0, 0)
            }

            packed_bits!(@impl_getters $storage, [$($field: $bits),*]);
            packed_bits!(@impl_setters $storage, [$($field: $bits),*]);
        }

        impl $name {
            /// Returns the raw underlying storage value.
            pub const fn get_raw(&self) -> $storage {
                self.0
            }

            /// Overwrites the raw underlying storage value.
            pub const fn set_raw(&mut self, value: $storage) {
                self.0 = value;
            }

            /// Constructs from a raw underlying storage value.
            pub const fn from_raw(value: $storage) -> Self {
                Self(value)
            }

            packed_bits!(@impl_bit_ops_methods $storage);
        }

        impl core::convert::From<$storage> for $name {
            fn from(raw: $storage) -> Self {
                Self(raw)
            }
        }

    };

    (@impl_new_asserts [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*]) => {
        assert!($first <= ((1 << $first_bits) - 1), concat!("value for field `", stringify!($first), "` exceeds its capacity"));
        packed_bits!(@impl_new_asserts [$($field: $bits),*]);
    };
    (@impl_new_asserts []) => {};

    (@impl_new_pack [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*], $offset:expr, $acc:expr) => {
        packed_bits!(@impl_new_pack [$($field: $bits),*], $offset + $first_bits, $acc | (($first & ((1 << $first_bits) - 1)) << $offset))
    };
    (@impl_new_pack [], $offset:expr, $acc:expr) => {
        Self($acc)
    };

     (@impl_getters $storage:ty, [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*]) => {
        #[doc = concat!("Returns the `", stringify!($first), "` field.")]
        pub fn $first(&self) -> $storage {
            self.0 & ((1 << $first_bits) - 1)
        }

        packed_bits!(@impl_getters $storage, [$($field: $bits),*], $first_bits);
    };

    (@impl_getters $storage:ty, [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*], $offset:expr) => {
        #[doc = concat!("Returns the `", stringify!($first), "` field.")]
        pub fn $first(&self) -> $storage {
            (self.0 >> $offset) & ((1 << $first_bits) - 1)
        }

        packed_bits!(@impl_getters $storage, [$($field: $bits),*], $offset + $first_bits);
    };

    (@impl_getters $storage:ty, [], $offset:expr) => {};
    (@impl_getters $storage:ty, []) => {};
    // setters internal macro rules
    (@impl_setters $storage:ty, [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*]) => {
        $crate::paste! {
            #[doc = concat!("Sets the `", stringify!($first), "` field, returning `self` for chaining.")]
            ///
            /// Panics if `value` exceeds the field's bit width.
            pub fn [<set_ $first>](&mut self, value: $storage) -> &mut Self {
                assert!(value <= ((1 << $first_bits) - 1), concat!("value for field `", stringify!($first), "` exceeds its capacity"));
                let mask = ((1 << $first_bits) - 1);
                self.0 = (self.0 & !mask) | (value & mask);
                self
            }

        }

        packed_bits!(@impl_setters $storage, [$($field: $bits),*], $first_bits);
    };

    (@impl_setters $storage:ty, [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*], $offset:expr) => {
        $crate::paste! {
            #[doc = concat!("Sets the `", stringify!($first), "` field, returning `self` for chaining.")]
            ///
            /// Panics if `value` exceeds the field's bit width.
            pub fn [<set_ $first>](&mut self, value: $storage) -> &mut Self {
                assert!(value <= ((1 << $first_bits) - 1), concat!("value for field `", stringify!($first), "` exceeds its capacity"));
                let mask = ((1 << $first_bits) - 1) << $offset;
                self.0 = (self.0 & !mask) | ((value & ((1 << $first_bits) - 1)) << $offset);
                self
            }

        }

        packed_bits!(@impl_setters $storage, [$($field: $bits),*], $offset + $first_bits);
    };

    (@impl_setters $storage:ty, [], $offset:expr) => {};
    (@impl_setters $storage:ty, []) => {};

    // Typed-field internal rules
    (@impl_typed_new $storage:ty, [$first:ident: $first_ty:ty: $first_bits:expr $(, $field:ident: $ty:ty: $bits:expr)*], $offset:expr, $packed:ident) => {
        let value = <$first_ty as $crate::PackedField>::pack($first) as $storage;
        assert!(value <= ((1 << $first_bits) - 1), concat!("value for field `", stringify!($first), "` exceeds its capacity"));
        $packed |= (value & ((1 << $first_bits) - 1)) << $offset;
        packed_bits!(@impl_typed_new $storage, [$($field: $ty: $bits),*], $offset + $first_bits, $packed);
    };
    (@impl_typed_new $storage:ty, [], $offset:expr, $packed:ident) => {};

    (@impl_typed_getters $storage:ty, [$first:ident: $first_ty:ty: $first_bits:expr $(, $field:ident: $ty:ty: $bits:expr)*]) => {
        #[doc = concat!("Returns the `", stringify!($first), "` field, or `None` if its raw bits have no valid value.")]
        pub fn $first(&self) -> Option<$first_ty> {
            <$first_ty as $crate::PackedField>::unpack((self.0 & ((1 << $first_bits) - 1)) as <$first_ty as $crate::PackedField>::Raw)
        }

        $crate::paste! {
            #[doc = concat!("Returns the raw `", stringify!($first), "` field bits.")]
            pub fn [<$first _raw>](&self) -> <$first_ty as $crate::PackedField>::Raw {
                (self.0 & ((1 << $first_bits) - 1)) as <$first_ty as $crate::PackedField>::Raw
            }

            #[doc = concat!("Returns the `", stringify!($first), "` field without validating its raw bits.")]
            ///
            /// # Safety
            ///
            /// The raw bits must have a valid value for `$first_ty`; otherwise
            /// the behavior is undefined.
            pub unsafe fn [<$first _unchecked>](&self) -> $first_ty {
                // SAFETY: the caller guarantees the raw value is a valid representation.
                unsafe {
                    <$first_ty as $crate::PackedField>::unpack_unchecked((self.0 & ((1 << $first_bits) - 1)) as <$first_ty as $crate::PackedField>::Raw)
                }
            }
        }

        packed_bits!(@impl_typed_getters $storage, [$($field: $ty: $bits),*], $first_bits);
    };

    (@impl_typed_getters $storage:ty, [$first:ident: $first_ty:ty: $first_bits:expr $(, $field:ident: $ty:ty: $bits:expr)*], $offset:expr) => {
        #[doc = concat!("Returns the `", stringify!($first), "` field, or `None` if its raw bits have no valid value.")]
        pub fn $first(&self) -> Option<$first_ty> {
            <$first_ty as $crate::PackedField>::unpack(((self.0 >> $offset) & ((1 << $first_bits) - 1)) as <$first_ty as $crate::PackedField>::Raw)
        }

        $crate::paste! {
            #[doc = concat!("Returns the raw `", stringify!($first), "` field bits.")]
            pub fn [<$first _raw>](&self) -> <$first_ty as $crate::PackedField>::Raw {
                ((self.0 >> $offset) & ((1 << $first_bits) - 1)) as <$first_ty as $crate::PackedField>::Raw
            }

            #[doc = concat!("Returns the `", stringify!($first), "` field without validating its raw bits.")]
            ///
            /// # Safety
            ///
            /// The raw bits must have a valid value for `$first_ty`; otherwise
            /// the behavior is undefined.
            pub unsafe fn [<$first _unchecked>](&self) -> $first_ty {
                // SAFETY: the caller guarantees the raw value is a valid representation.
                unsafe {
                    <$first_ty as $crate::PackedField>::unpack_unchecked(((self.0 >> $offset) & ((1 << $first_bits) - 1)) as <$first_ty as $crate::PackedField>::Raw)
                }
            }
        }

        packed_bits!(@impl_typed_getters $storage, [$($field: $ty: $bits),*], $offset + $first_bits);
    };

    (@impl_typed_getters $storage:ty, [], $offset:expr) => {};
    (@impl_typed_getters $storage:ty, []) => {};

    (@impl_typed_setters $storage:ty, [$first:ident: $first_ty:ty: $first_bits:expr $(, $field:ident: $ty:ty: $bits:expr)*]) => {
        $crate::paste! {
            #[doc = concat!("Sets the `", stringify!($first), "` field, returning `self` for chaining.")]
            ///
            /// Panics if the packed value exceeds the field's bit width.
            pub fn [<set_ $first>](&mut self, value: $first_ty) -> &mut Self {
                let packed = <$first_ty as $crate::PackedField>::pack(value) as $storage;
                assert!(packed <= ((1 << $first_bits) - 1), concat!("value for field `", stringify!($first), "` exceeds its capacity"));
                let mask = ((1 << $first_bits) - 1);
                self.0 = (self.0 & !mask) | (packed & mask);
                self
            }

            #[doc = concat!("Sets the raw `", stringify!($first), "` field bits, returning an error if the value exceeds the field's bit width.")]
            pub fn [<set_ $first _raw>](&mut self, value: <$first_ty as $crate::PackedField>::Raw) -> Result<&mut Self, $crate::FieldError> {
                let value = value as $storage;
                let mask = ((1 << $first_bits) - 1);
                if value > mask {
                    return Err($crate::FieldError { field: stringify!($first), value: value as u64, max: mask as u64 });
                }
                self.0 = (self.0 & !mask) | (value & mask);
                Ok(self)
            }

            #[doc = concat!("Sets the `", stringify!($first), "` field, returning an error if the packed value exceeds the field's bit width.")]
            pub fn [<try_set_ $first>](&mut self, value: $first_ty) -> Result<&mut Self, $crate::FieldError> {
                let packed = <$first_ty as $crate::PackedField>::pack(value) as $storage;
                let mask = ((1 << $first_bits) - 1);
                if packed > mask {
                    return Err($crate::FieldError { field: stringify!($first), value: packed as u64, max: mask as u64 });
                }
                self.0 = (self.0 & !mask) | (packed & mask);
                Ok(self)
            }
        }

        packed_bits!(@impl_typed_setters $storage, [$($field: $ty: $bits),*], $first_bits);
    };

    (@impl_typed_setters $storage:ty, [$first:ident: $first_ty:ty: $first_bits:expr $(, $field:ident: $ty:ty: $bits:expr)*], $offset:expr) => {
        $crate::paste! {
            #[doc = concat!("Sets the `", stringify!($first), "` field, returning `self` for chaining.")]
            ///
            /// Panics if the packed value exceeds the field's bit width.
            pub fn [<set_ $first>](&mut self, value: $first_ty) -> &mut Self {
                let packed = <$first_ty as $crate::PackedField>::pack(value) as $storage;
                assert!(packed <= ((1 << $first_bits) - 1), concat!("value for field `", stringify!($first), "` exceeds its capacity"));
                let mask = ((1 << $first_bits) - 1) << $offset;
                self.0 = (self.0 & !mask) | ((packed & ((1 << $first_bits) - 1)) << $offset);
                self
            }

            #[doc = concat!("Sets the raw `", stringify!($first), "` field bits, returning an error if the value exceeds the field's bit width.")]
            pub fn [<set_ $first _raw>](&mut self, value: <$first_ty as $crate::PackedField>::Raw) -> Result<&mut Self, $crate::FieldError> {
                let value = value as $storage;
                let mask = ((1 << $first_bits) - 1) << $offset;
                if value > ((1 << $first_bits) - 1) {
                    return Err($crate::FieldError { field: stringify!($first), value: value as u64, max: ((1 << $first_bits) - 1) as u64 });
                }
                self.0 = (self.0 & !mask) | ((value & ((1 << $first_bits) - 1)) << $offset);
                Ok(self)
            }

            #[doc = concat!("Sets the `", stringify!($first), "` field, returning an error if the packed value exceeds the field's bit width.")]
            pub fn [<try_set_ $first>](&mut self, value: $first_ty) -> Result<&mut Self, $crate::FieldError> {
                let packed = <$first_ty as $crate::PackedField>::pack(value) as $storage;
                let mask = ((1 << $first_bits) - 1) << $offset;
                if packed > ((1 << $first_bits) - 1) {
                    return Err($crate::FieldError { field: stringify!($first), value: packed as u64, max: ((1 << $first_bits) - 1) as u64 });
                }
                self.0 = (self.0 & !mask) | ((packed & ((1 << $first_bits) - 1)) << $offset);
                Ok(self)
            }
        }

        packed_bits!(@impl_typed_setters $storage, [$($field: $ty: $bits),*], $offset + $first_bits);
    };

    (@impl_typed_setters $storage:ty, [], $offset:expr) => {};
    (@impl_typed_setters $storage:ty, []) => {};

    (@impl_typed_try_from $storage:ty, $value:ident, [$first:ident: $first_ty:ty: $first_bits:expr $(, $field:ident: $ty:ty: $bits:expr)*]) => {
        {
            let mask = ((1 << $first_bits) - 1);
            let raw = ($value & mask) as <$first_ty as $crate::PackedField>::Raw;
            if <$first_ty as $crate::PackedField>::unpack(raw).is_none() {
                return Err($crate::FieldError {
                    field: stringify!($first),
                    value: raw as u64,
                    max: ((1 << $first_bits) - 1) as u64,
                });
            }
        }
        packed_bits!(@impl_typed_try_from $storage, $value, [$($field: $ty: $bits),*], $first_bits);
    };

    (@impl_typed_try_from $storage:ty, $value:ident, [$first:ident: $first_ty:ty: $first_bits:expr $(, $field:ident: $ty:ty: $bits:expr)*], $offset:expr) => {
        {
            let mask = ((1 << $first_bits) - 1) << $offset;
            let raw = (($value & mask) >> $offset) as <$first_ty as $crate::PackedField>::Raw;
            if <$first_ty as $crate::PackedField>::unpack(raw).is_none() {
                return Err($crate::FieldError {
                    field: stringify!($first),
                    value: raw as u64,
                    max: ((1 << $first_bits) - 1) as u64,
                });
            }
        }
        packed_bits!(@impl_typed_try_from $storage, $value, [$($field: $ty: $bits),*], $offset + $first_bits);
    };

    (@impl_typed_try_from $storage:ty, $value:ident, [], $offset:expr) => {};
    (@impl_typed_try_from $storage:ty, $value:ident, []) => {};

    (@impl_bit_ops_methods $storage:ty) => {
        /// Returns the storage width in bits.
        pub fn bit_width(&self) -> usize {
            core::mem::size_of::<$storage>() * 8
        }

        /// Returns whether the bit at `index` is set.
        ///
        /// Panics if `index` is out of range.
        pub fn get_bit(&self, index: usize) -> bool {
            assert!(index < self.bit_width(), "bit index {} out of range", index);
            (self.0 >> index) & 1 == 1
        }

        /// Sets or clears the bit at `index`, returning `self` for chaining.
        ///
        /// Panics if `index` is out of range.
        pub fn set_bit(&mut self, index: usize, value: bool) -> &mut Self {
            assert!(index < self.bit_width(), "bit index {} out of range", index);
            let mask = 1 << index;
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
            self
        }

        /// Clears the bit at `index`, returning `self` for chaining.
        ///
        /// Panics if `index` is out of range.
        pub fn clear_bit(&mut self, index: usize) -> &mut Self {
            self.set_bit(index, false)
        }

        /// Toggles the bit at `index`, returning `self` for chaining.
        ///
        /// Panics if `index` is out of range.
        pub fn toggle_bit(&mut self, index: usize) -> &mut Self {
            assert!(index < self.bit_width(), "bit index {} out of range", index);
            self.0 ^= 1 << index;
            self
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    // Define packed structs
    packed_bits!(
        struct Date(u16) {
            day: 5,
            month: 4,
            year: 7,
        }
    );

    packed_bits! {
        struct Rgb565(u16) {
            blue: 5,
            green: 6,
            red: 5,
        }
    }

    packed_bits! {
        struct Time(u32) {
            second: 6,
            minute: 6,
            hour: 5,
        }
    }

    packed_bits! {
        struct Lc3Add(u16) {
            value: 5,   // SR2 (register mode) or imm5 (immediate mode)
            imm: 1,     // 0 = register mode, 1 = immediate mode
            sr1: 3,
            dr: 3,
            opcode: 4,  // 0b0001 for ADD
        }
    }

    packed_bits! {
        struct TcpFlags(u8) {
            fin: 1,
            syn: 1,
            _rst: 1,
            _psh: 1,
            ack: 1,
            _urg: 1,
            _ece: 1,
            _cwr: 1,
        }
    }

    #[test]
    fn basic_functionality() {
        let color = Rgb565::new(31, 63, 31);
        // ADD R2, R1, R3 (register mode) -> 0x144C
        let add_reg = Lc3Add::from(0x144C);
        // ADD R0, R1, #5 (immediate mode) -> 0x1065
        let add_imm = Lc3Add::new(0b00101, 1, 1, 0, 0b0001);
        let flags = TcpFlags::new(0, 1, 0, 0, 1, 0, 0, 0);

        assert_eq!((31, 63, 31), (color.blue(), color.green(), color.red()));
        assert_eq!(0x144C, add_reg.get_raw());
        assert_eq!(
            (0b01100, 0, 1, 2, 0b0001),
            (
                add_reg.value(),
                add_reg.imm(),
                add_reg.sr1(),
                add_reg.dr(),
                add_reg.opcode()
            )
        );
        assert_eq!(0x1065, add_imm.get_raw());
        assert_eq!((0, 1, 1), (flags.fin(), flags.syn(), flags.ack()));

        assert_eq!(2, size_of::<Rgb565>());
        assert_eq!(2, size_of::<Lc3Add>());
        assert_eq!(1, size_of::<TcpFlags>());
    }

    #[test]
    fn stress_test_comprehensive() {
        // Test many combinations to ensure no bit interference
        for day in [1, 15, 31] {
            for month in [1, 6, 12] {
                for year in [0, 50, 99] {
                    let date = Date::new(day, month, year);
                    assert_eq!((day, month, year), (date.day(), date.month(), date.year()));
                }
            }
        }

        // Test TCP flags exhaustively
        for i in 0..=255u8 {
            let flags = TcpFlags::new(
                i & 1,
                (i >> 1) & 1,
                (i >> 2) & 1,
                (i >> 3) & 1,
                (i >> 4) & 1,
                (i >> 5) & 1,
                (i >> 6) & 1,
                (i >> 7) & 1,
            );
            assert_eq!(i & 1, flags.fin());
            assert_eq!((i >> 1) & 1, flags.syn());
            assert_eq!((i >> 4) & 1, flags.ack());
        }
    }

    #[test]
    fn memory_efficiency() {
        // Verify packed types are smaller than unpacked equivalents
        struct UnpackedDate {
            _day: u16,
            _month: u16,
            _year: u16,
        }
        struct UnpackedRgb {
            _r: u16,
            _g: u16,
            _b: u16,
        }

        assert!(size_of::<Date>() < size_of::<UnpackedDate>());
        assert!(size_of::<Rgb565>() < size_of::<UnpackedRgb>());

        // 100 * 2 bytes
        assert_eq!(200, size_of::<[Date; 100]>());
        // 100 * 2 bytes
        assert_eq!(200, size_of::<[Rgb565; 100]>());
        // 100 * 1 byte
        assert_eq!(100, size_of::<[TcpFlags; 100]>());
    }

    #[test]
    fn set_functionality() {
        let mut date = Date::new(1, 1, 0);
        date.set_day(31).set_month(12).set_year(99);
        assert_eq!((31, 12, 99), (date.day(), date.month(), date.year()));

        let mut color = Rgb565::new(0, 0, 0);
        color.set_red(31).set_green(63).set_blue(31);
        assert_eq!((31, 63, 31), (color.blue(), color.green(), color.red()));

        let mut time = Time::new(0, 0, 0);
        time.set_hour(23).set_minute(59).set_second(59);
        assert_eq!((59, 59, 23), (time.second(), time.minute(), time.hour()));
    }

    #[test]
    fn raw_access() {
        let date = Date::new(25, 12, 99);
        let raw = date.get_raw();
        assert_eq!(raw, (25 | 12 << 5 | 99 << 9));
        assert_eq!(date, Date::from_raw(raw));

        let mut color = Rgb565::new(31, 63, 31);
        color.set_raw(0);
        assert_eq!((0, 0, 0), (color.blue(), color.green(), color.red()));
        assert_eq!(0, color.get_raw());
    }

    #[test]
    fn bit_manipulation() {
        let mut flags = TcpFlags::new(0, 0, 0, 0, 0, 0, 0, 0);

        // setting individual bits
        flags.set_bit(0, true).set_bit(4, true);
        assert_eq!(1, flags.fin());
        assert_eq!(1, flags.ack());
        assert_eq!(0b0001_0001, flags.get_raw());

        // get_bit reads back
        assert!(flags.get_bit(0));
        assert!(flags.get_bit(4));
        assert!(!flags.get_bit(1));

        // clear_bit
        flags.clear_bit(4);
        assert_eq!(0, flags.ack());
        assert_eq!(1, flags.get_raw());

        // toggle_bit
        flags.toggle_bit(4).toggle_bit(0);
        assert_eq!(0, flags.fin());
        assert_eq!(1, flags.ack());
        assert_eq!(0b0001_0000, flags.get_raw());

        // bit_width
        assert_eq!(8, flags.bit_width());
        assert_eq!(16, Date::new(1, 1, 1).bit_width());
    }

    #[test]
    #[should_panic(expected = "bit index 8 out of range")]
    fn bit_manipulation_out_of_range() {
        let mut flags = TcpFlags::new(0, 0, 0, 0, 0, 0, 0, 0);
        flags.set_bit(8, true);
    }

    #[test]
    #[should_panic(expected = "exceeds its capacity")]
    fn new_overflow_panics() {
        let _ = Date::new(32, 1, 1);
    }

    #[test]
    #[should_panic(expected = "exceeds its capacity")]
    fn setter_overflow_panics() {
        let mut date = Date::new(1, 1, 1);
        date.set_month(16);
    }

    #[test]
    fn const_fn_creation() {
        const BIRTHDAY: Date = Date::new(25, 12, 99);
        const WHITE: Rgb565 = Rgb565::new(31, 63, 31);
        const SYN_ACK: TcpFlags = TcpFlags::new(0, 1, 0, 0, 1, 0, 0, 0);

        assert_eq!(
            (25, 12, 99),
            (BIRTHDAY.day(), BIRTHDAY.month(), BIRTHDAY.year())
        );
        assert_eq!((31, 63, 31), (WHITE.blue(), WHITE.green(), WHITE.red()));
        assert_eq!((0, 1, 1), (SYN_ACK.fin(), SYN_ACK.syn(), SYN_ACK.ack()));
    }

    #[cfg(feature = "derive")]
    mod typed {
        use super::*;
        use packed_bits_derive::PackedField;

        #[derive(PackedField, Debug, Clone, Copy, PartialEq, Eq)]
        enum Color {
            Red = 0,
            Green = 2,
            Blue = 3,
            Yellow = 4, // exceeds the 2-bit field (max 3)
        }

        packed_bits! {
            struct Pixel(u16) {
                color: Color = 2,
                alpha: u8 = 8,
            }
        }

        #[test]
        fn typed_new_get() {
            let pixel = Pixel::new(Color::Blue, 200);
            assert_eq!(Some(Color::Blue), pixel.color());
            assert_eq!(3, pixel.color_raw());
            assert_eq!(Some(200), pixel.alpha());
            assert_eq!(200, pixel.alpha_raw());
            assert_eq!(2, size_of::<Pixel>());
        }

        #[test]
        fn typed_checked_getter() {
            let mut pixel = Pixel::new(Color::Red, 1);
            // Raw value 1 is a hole (no variant has discriminant 1).
            pixel.set_raw(1 | (1 << 2));
            assert_eq!(None, pixel.color());
            assert_eq!(1, pixel.color_raw());
            assert_eq!(Some(1), pixel.alpha());
        }

        #[test]
        fn typed_unchecked_getter() {
            let pixel = Pixel::new(Color::Green, 1);
            assert_eq!(Color::Green, unsafe { pixel.color_unchecked() });
        }

        #[test]
        fn typed_setters() {
            let mut pixel = Pixel::new(Color::Red, 0);
            pixel.set_color(Color::Blue);
            assert_eq!(Some(Color::Blue), pixel.color());

            pixel.set_color_raw(2).unwrap();
            assert_eq!(Some(Color::Green), pixel.color());

            pixel.try_set_color(Color::Blue).unwrap();
            assert_eq!(Some(Color::Blue), pixel.color());
        }

        #[test]
        #[should_panic(expected = "exceeds its capacity")]
        fn typed_set_overflow_panics() {
            // Yellow = 4 cannot fit in the 2-bit field (max 3).
            let mut pixel = Pixel::new(Color::Red, 0);
            pixel.set_color(Color::Yellow);
        }

        #[test]
        fn typed_set_raw_overflow_errors() {
            let mut pixel = Pixel::new(Color::Red, 0);
            let err = pixel.set_color_raw(4).unwrap_err();
            assert_eq!("color", err.field);
            assert_eq!(4, err.value);
            assert_eq!(3, err.max);

            let err = pixel.try_set_color(Color::Yellow).unwrap_err();
            assert_eq!(4, err.value);
        }

        #[test]
        fn typed_try_from() {
            // color = 2 (Green) at bits 0-1, alpha = 1 at bits 2-9.
            let ok = Pixel::try_from(2 | (1 << 2)).unwrap();
            assert_eq!(Some(Color::Green), ok.color());
            assert_eq!(Some(1), ok.alpha());

            // Raw value 1 puts color = 1, a hole with no valid variant.
            let err = Pixel::try_from(1).unwrap_err();
            assert_eq!("color", err.field);
        }
    }
}
