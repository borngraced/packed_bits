use packed_bits::packed_bits;

packed_bits!(
    OversizedFields: u8 {
        _a: 5,
        _b: 4,
    }
);

packed_bits!(
    RGB: u16 {
        red: 8,
        green: 8,
        blue: 8,
    }
);

fn main() {}