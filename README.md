

# packed_bits

Memory-efficient bit packing library

## Installation

Update your Cargo.toml:

```toml
# std
[dependencies]
packed_bits = { version = "0.1.0", features = ["std"] }

# no_std 
[dependencies]
packed_bits = { version = "0.1.0", default-features = false }
```

## Memory Savings

Without packing: day(4 bytes) + month(4 bytes) + year(4 bytes) = 12 bytes total
With packing: everything fits in just 2 bytes!

## Usage

```rust
use packed_bits::packed_bits;

packed_bits! {
    struct Date(u16) {
        day: 5,    // Can store 1-31 (needs 5 bits since 2^5 = 32)
        month: 4,  // Can store 1-12 (needs 4 bits since 2^4 = 16)
        year: 7,   // Can store 0-99 (needs 7 bits since 2^7 = 128)
    }
}

let birthday = Date::new(25, 12, 99);
// read values
println!("Day: {}", birthday.day());     // 25
println!("Month: {}", birthday.month()); // 12
println!("Year: {}", birthday.year());   // 99

// Memory usage
use core::mem::size_of;
assert_eq!(2, size_of::<Date>()); // Only 2 bytes!

### RGB Color (16-bit)
packed_bits! {
    struct Rgb565(u16) {
        blue: 5,   // 0-31
        green: 6,  // 0-63
        red: 5,    // 0-31
    }
}
let white = Rgb565::new(31, 63, 31);

### TCP Flags(8-bit)
packed_bits! {
    struct TcpFlags(u8) {
        fin: 1,
        syn: 1,
        rst: 1,
        psh: 1,
        ack: 1,
        urg: 1,
        ece: 1,
        cwr: 1,
    }
}

let syn_ack = TcpFlags::new(0, 1, 0, 0, 1, 0, 0, 0);
```

## Features

- No dependencies - Pure Rust implementation
- no_std compatible - Works in embedded environments
- Zero-cost abstractions - Compiles to raw bit operations
- Type safe - Each field gets its own accessor method
- Memory efficient - Pack multiple values into single integers

## Todo

[ ] impl setter methods to update individual fields
[ ] impl compile-time validation for bit overflow 
[ ] impl const fn support for compile-time creation
[ ] impl bit manipulation methods (set_bit, clear_bit, etc.)

## Important Notes

- Make sure your bit counts add up to fit in your storage type
- u16 can hold 16 bits total, u32 can hold 32 bits, etc.
- Each field gets a method with the same name to read its value
- Values are stored from lowest bits to highest bits in declaration order
- Maximum value for each field is (2^bits) - 1
