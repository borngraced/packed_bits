#![cfg_attr(not(feature = "std"), no_std)]

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
/// let birthday = Date::new(25, 12, 99);
///
/// // Getting the values back out
/// println!("Day: {}", birthday.day());     // Day: 25
/// println!("Month: {}", birthday.month()); // Month: 12
/// println!("Year: {}", birthday.year());   // Year: 99
///
/// // Memory Usage
/// #[cfg(not(feature = "std"))]
/// use core::mem::size_of;
/// #[cfg(feature = "std")]
/// use std::mem::size_of;
///
/// assert_eq!(2, core::mem::size_of::<Date>()); // Only 2 bytes!
/// ```
///
/// # Important notes
/// - Make sure your bit counts add up to fit in your storage type
/// - u16 can hold 16 bits total, u32 can hold 32 bits, etc.
/// - Each field gets a method with the same name to read its value
/// - Values are stored from lowest bits to highest bits in declaration order
#[macro_export]
macro_rules! packed_bits {
    (
       struct $name:ident($storage:ty) {
            $(
                $field:ident: $bits:expr,
            )*
        }
    ) => {
        #[derive(Copy, Clone)]
        struct $name($storage);

        impl $name {
            fn new($($field: $storage),*) -> Self {
                let fields = [$($field),*];
                let bit_sizes = [$($bits),*];

                let mut packed = 0;
                let mut offset = 0;

                for i in 0..fields.len() {
                    packed |= (fields[i] & ((1 << bit_sizes[i]) - 1)) << offset;
                    offset += bit_sizes[i];
                }

                Self(packed)
            }

            packed_bits!(@impl_getters $storage, [$($field: $bits),*]);
        }

    };

     (@impl_getters $storage:ty, [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*]) => {
        fn $first(&self) -> $storage {
            self.0 & ((1 << $first_bits) - 1)
        }

        packed_bits!(@impl_getters $storage, [$($field: $bits),*], $first_bits);
    };

    (@impl_getters $storage:ty, [$first:ident: $first_bits:expr $(, $field:ident: $bits:expr)*], $offset:expr) => {
        fn $first(&self) -> $storage {
            (self.0 >> $offset) & ((1 << $first_bits) - 1)
        }

        packed_bits!(@impl_getters $storage, [$($field: $bits),*], $offset + $first_bits);
    };

    (@impl_getters $storage:ty, [], $offset:expr) => {};
    (@impl_getters $storage:ty, []) => {};
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use core::mem::size_of;
    #[cfg(feature = "std")]
    use std::mem::size_of;

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
}
