

# packed_bits

Memory-efficient bit packing library

## Installation

Update your Cargo.toml:

```toml
# std
[dependencies]
packed_bits = { git = "https://github.com/borngraced/packed_bits.git", features = ["std"] }

# no_std 
[dependencies]
packed_bits = { git = "https://github.com/borngraced/packed_bits.git", default-features = false }
```

## Memory Savings

Without packing: day(4 bytes) + month(4 bytes) + year(4 bytes) = 12 bytes total
With packing: everything fits in just 2 bytes!

## Usage

```rust
use packed_bits::packed_bits;

// LC-3 ADD instruction (16-bit). Fields map directly to the ISA layout:
//   ADD DR, SR1, SR2   -> 0001 DR SR1 0 000 SR2 00
//   ADD DR, SR1, imm5  -> 0001 DR SR1 1 imm5
packed_bits! {
    struct Lc3Add(u16) {
        value: 5,   // SR2 (register mode) or imm5 (immediate mode)
        imm: 1,     // 0 = register mode, 1 = immediate mode
        sr1: 3,
        dr: 3,
        opcode: 4,  // 0b0001 for ADD
    }
}

// ADD R2, R1, R3 (register mode) -> 0x144C
let add_reg = Lc3Add::from(0x144C);
assert_eq!((0b01100, 0, 1, 2, 0b0001), (add_reg.value(), add_reg.imm(), add_reg.sr1(), add_reg.dr(), add_reg.opcode()));

// ADD R0, R1, #5 (immediate mode) -> 0x1065
let add_imm = Lc3Add::new(0b00101, 1, 1, 0, 0b0001);
assert_eq!(0x1065, add_imm.get_raw());
```

## More Examples

### Packing domain values

```rust
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
// update values (chainable, returns &mut Self)
birthday.set_day(1).set_month(1).set_year(1);
assert_eq!((1, 1, 1), (birthday.day(), birthday.month(), birthday.year()));
// const-compatible creation
const EPOCH: Date = Date::new(1, 1, 0);
// raw bit access
let mut epoch = EPOCH;
epoch.set_bit(5, true).toggle_bit(9);
assert_eq!(true, epoch.get_bit(5));
assert_eq!(true, epoch.get_bit(9));
epoch.clear_bit(5);
assert_eq!(false, epoch.get_bit(5));
// raw storage access
assert_eq!(epoch, Date::from_raw(epoch.get_raw()));
// bit width
assert_eq!(16, epoch.bit_width());
// Memory usage
assert_eq!(2, core::mem::size_of::<Date>()); // 2 bytes!

// RGB Color (16-bit)
packed_bits! {
    struct Rgb565(u16) {
        blue: 5,   // 0-31
        green: 6,  // 0-63
        red: 5,    // 0-31
    }
}
let white = Rgb565::new(31, 63, 31);

// TCP Flags(8-bit)
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

- Minimal dependencies - Pure Rust implementation
- no_std compatible - Works in embedded environments
- Zero-cost abstractions - Compiles to raw bit operations
- Type safe - Each field gets its own accessor and setter method
- Memory efficient - Pack multiple values into single integers
- Compile-time validation - Catches bit overflow at build time
- const fn support - Create packed values at compile time via `Date::new()`
- Chainable setters - Update fields fluently via `set_<field>()`
- Runtime overflow detection - Panics when a value exceeds its field capacity
- Raw bit manipulation - `get_bit`, `set_bit`, `clear_bit`, `toggle_bit`
- Raw storage access - `get_raw`, `set_raw`, `from_raw`
- Conversion support - `From<Raw>`/`TryFrom<Raw>` construction for typed and bare fields

## Important Notes

- Make sure your bit counts add up to fit in your storage type
- u16 can hold 16 bits total, u32 can hold 32 bits, etc.
- Each field gets a method with the same name to read its value, plus `set_<field>` to update it
- Values are stored from lowest bits to highest bits in declaration order
- Maximum value for each field is (2^bits) - 1
- Passing an out-of-range value to `new`/setters panics instead of silently truncating
- Bare-field structs get `From<Raw>` (infallible); typed-field structs get `TryFrom<Raw>`, which fails if any field's raw bits have no valid value (e.g. an enum discriminant hole)
- Bit manipulation methods operate on the raw underlying storage, not logical fields
- Bit indices are 0-based; out-of-range indices panic

## TODO

- [ ] Implement `Display`/`FromStr` conversions
- [ ] Implement support for `bool` fields
- [ ] Implement support for signed integer fields
