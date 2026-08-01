use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, parse_macro_input};

/// Derives the `PackedField` trait for a fieldless enum.
///
/// The smallest integer type that can hold every discriminant becomes
/// `PackedField::Raw`. `unpack` returns `None` for raw values that do not
/// correspond to a variant; `unpack_unchecked` uses `unreachable_unchecked`
/// for the fall-through arm.
///
/// # Example
///
/// ```rust
/// use packed_bits::PackedField;
///
/// #[derive(PackedField)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
/// ```
#[proc_macro_derive(PackedField)]
pub fn derive_packed_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "PackedField can only be derived for enums",
            ));
        }
    };

    let mut discriminants: Vec<u64> = Vec::new();
    let mut variants: Vec<syn::Ident> = Vec::new();

    for (index, variant) in data.variants.iter().enumerate() {
        match &variant.fields {
            Fields::Unit => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "PackedField can only be derived for fieldless enums",
                ));
            }
        }

        let discriminant = match &variant.discriminant {
            Some((_, expr)) => parse_discriminant(expr)?,
            None => index as u64,
        };

        variants.push(variant.ident.clone());
        discriminants.push(discriminant);
    }

    if discriminants.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "PackedField cannot be derived for an empty enum",
        ));
    }

    let max = *discriminants.iter().max().unwrap();
    let raw_type = integer_type_for_max(max);

    let unpack_arms: Vec<TokenStream2> = discriminants
        .iter()
        .zip(variants.iter())
        .map(|(discriminant, variant)| {
            let lit = discriminant_literal(*discriminant, &raw_type);
            quote! {
                #lit => Some(#name::#variant),
            }
        })
        .collect();

    let unchecked_arms: Vec<TokenStream2> = discriminants
        .iter()
        .zip(variants.iter())
        .map(|(discriminant, variant)| {
            let lit = discriminant_literal(*discriminant, &raw_type);
            quote! {
                #lit => #name::#variant,
            }
        })
        .collect();

    Ok(quote! {
        impl ::packed_bits::PackedField for #name {
            type Raw = #raw_type;

            fn pack(self) -> Self::Raw {
                self as #raw_type
            }

            fn unpack(raw: Self::Raw) -> Option<Self> {
                match raw {
                    #(#unpack_arms)*
                    _ => None,
                }
            }

            unsafe fn unpack_unchecked(raw: Self::Raw) -> Self {
                match raw {
                    #(#unchecked_arms)*
                    _ => ::core::hint::unreachable_unchecked(),
                }
            }
        }
    })
}

fn parse_discriminant(expr: &syn::Expr) -> syn::Result<u64> {
    match expr {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Int(int) => int
                .base10_parse::<u64>()
                .map_err(|_| syn::Error::new_spanned(int, "discriminant must fit in a u64")),
            _ => Err(syn::Error::new_spanned(
                expr,
                "discriminant must be an integer literal",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            expr,
            "discriminant must be an integer literal",
        )),
    }
}

fn integer_type_for_max(max: u64) -> TokenStream2 {
    if max <= u8::MAX as u64 {
        quote! { u8 }
    } else if max <= u16::MAX as u64 {
        quote! { u16 }
    } else if max <= u32::MAX as u64 {
        quote! { u32 }
    } else {
        quote! { u64 }
    }
}

fn discriminant_literal(value: u64, raw_type: &TokenStream2) -> TokenStream2 {
    let _ = raw_type; // type inference picks the raw type from the scrutinee
    let lit = proc_macro2::Literal::u64_unsuffixed(value);
    quote! { #lit }
}
