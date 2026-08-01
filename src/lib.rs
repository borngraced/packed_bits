#![cfg_attr(not(feature = "std"), no_std)]

pub use paste::paste;
pub use static_assertions;

/// Memory-efficient bit packing library.
/// Define a packed_bits struct that stores multiple fields in a single integer.
///
/// # Parameters
/// - `name`: The name of the generated struct
/// - `storage`: The underlying data type/size (`u8`, `u16`, `u32`, `u64`)
/// - `field`: Field name (will also be used as a getter method)
/// - `bits`: Number of bits allocated for this field
///
/// # Memory savings example
/// Without packing: day(4 bytes) + month(4 bytes) + year(4 bytes) = 12 bytes total
/// With packing: everything fits in just 2 bytes!
///
/// # Example
/// ```rust
/// use packed_bits::packed_bits;
/// packed_bits! {
///     struct Date(u16) {
///         day: 5,    // Can store 1-31 (needs 5 bits since 2^5 = 32)
///         month: 4,  // Can store 1-12 (needs 4 bits since 2^4 = 16)
///         year: 7,   // Can store 0-99 (needs 7 bits since 2^7 = 128)
///     }
/// }
///
/// // Create a new date
/// let mut birthday = Date::new(25, 12, 99);
///
/// // Getting the values back out
/// println!("Day: {}", birthday.day());     // Day: 25
/// println!("Month: {}", birthday.month()); // Month: 12
/// println!("Year: {}", birthday.year());   // Year: 99
///
/// // Update a single field (chainable)
/// birthday.set_day(1).set_month(1);
/// assert_eq!((1, 1, 99), (birthday.day(), birthday.month(), birthday.year()));
///
/// // const-compatible creation
/// const EPOCH: Date = Date::new(1, 1, 0);
///
/// // Memory Usage
/// assert_eq!(2, core::mem::size_of::<Date>()); // Only 2 bytes!
/// ```
///
/// # Important notes
/// - Make sure your bit counts add up to fit in your storage type
/// - u16 can hold 16 bits total, u32 can hold 32 bits, etc.
/// - Each field gets a method with the same name to read its value, plus `set_<field>` to update it
/// - Values are stored from lowest bits to highest bits in declaration order
/// - Passing an out-of-range value to `new`/setters panics; use `set_bit`/`get_bit` for raw bit access
#[macro_export]
macro_rules! packed_bits {
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
        pub fn $first(&self) -> $storage {
            self.0 & ((1 << $first_bits) - 1)
        }

        packed_bits!(@impl_getters $storage, [$($field: $bits),*], $first_bits);
    };

    (@impl_getters $storage:ty, [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*], $offset:expr) => {
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

    (@impl_bit_ops_methods $storage:ty) => {
        pub fn bit_width(&self) -> usize {
            core::mem::size_of::<$storage>() * 8
        }

        pub fn get_bit(&self, index: usize) -> bool {
            assert!(index < self.bit_width(), "bit index {} out of range", index);
            (self.0 >> index) & 1 == 1
        }

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

        pub fn clear_bit(&mut self, index: usize) -> &mut Self {
            self.set_bit(index, false)
        }

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
        let date = Date::new(25, 12, 99);
        let color = Rgb565::new(31, 63, 31);
        let time = Time::new(59, 59, 23);
        let flags = TcpFlags::new(0, 1, 0, 0, 1, 0, 0, 0);

        assert_eq!((25, 12, 99), (date.day(), date.month(), date.year()));
        assert_eq!((31, 63, 31), (color.blue(), color.green(), color.red()));
        assert_eq!((59, 59, 23), (time.second(), time.minute(), time.hour()));
        assert_eq!((0, 1, 1), (flags.fin(), flags.syn(), flags.ack()));

        assert_eq!(2, size_of::<Date>());
        assert_eq!(2, size_of::<Rgb565>());
        assert_eq!(4, size_of::<Time>());
        assert_eq!(1, size_of::<TcpFlags>());
    }

    #[test]
    fn boundary_values() {
        let min_date = Date::new(0, 0, 0);
        let max_date = Date::new(31, 15, 127);

        let black = Rgb565::new(0, 0, 0);
        let white = Rgb565::new(31, 63, 31);

        let midnight = Time::new(0, 0, 0);
        let max_time = Time::new(59, 59, 23);

        assert_eq!(
            (0, 0, 0),
            (min_date.day(), min_date.month(), min_date.year())
        );
        assert_eq!(
            (31, 15, 127),
            (max_date.day(), max_date.month(), max_date.year())
        );
        assert_eq!((0, 0, 0), (black.blue(), black.green(), black.red()));
        assert_eq!((31, 63, 31), (white.blue(), white.green(), white.red()));
        assert_eq!(
            (0, 0, 0),
            (midnight.second(), midnight.minute(), midnight.hour())
        );
        assert_eq!(
            (59, 59, 23),
            (max_time.second(), max_time.minute(), max_time.hour())
        );
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

        assert_eq!((25, 12, 99), (BIRTHDAY.day(), BIRTHDAY.month(), BIRTHDAY.year()));
        assert_eq!((31, 63, 31), (WHITE.blue(), WHITE.green(), WHITE.red()));
        assert_eq!((0, 1, 1), (SYN_ACK.fin(), SYN_ACK.syn(), SYN_ACK.ack()));
    }
}
