///! The only reason for this procedural macro is to
///! implement `From<u8>` for `WifiCommand` which has
///! 53 variants.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(FromByte)]
pub fn from_discriminant(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_from_u8(&ast)
}

fn impl_from_u8(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let idents: Vec<Ident>;
    if let syn::Data::Enum(e) = &ast.data {
        idents = e.variants.clone().into_iter().map(|v| v.ident).collect();
    } else {
        panic!("FromByte is only meant to be derived for #[repr(u8)] enums");
    }
    let tokens = quote! {
        impl From<u8> for #name {
            fn from(value: u8) -> Self {
                match value {
                    #(x if x == #name::#idents as u8 => #name::#idents,)*
                    _ => { #name::Invalid }
                }
            }
        }
    };
    TokenStream::from(tokens)
}
