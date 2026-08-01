use proc_macro::TokenStream;

#[proc_macro_derive(PackedField, attributes(packed_bits))]
pub fn derive_packed_field(input: TokenStream) -> TokenStream {
    input
}
