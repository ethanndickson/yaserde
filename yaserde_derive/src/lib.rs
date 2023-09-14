#![recursion_limit = "256"]

// Required for Rust < 1.42
extern crate proc_macro;

mod common;
mod de;
mod ser;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(YaDeserialize, attributes(yaserde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
  let ast = syn::parse(input).unwrap();
  match de::expand_derive_deserialize(&ast) {
    Ok(expanded) => expanded.into(),
    Err(msg) => panic!("{}", msg),
  }
}

#[proc_macro_derive(YaSerialize, attributes(yaserde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
  let ast = syn::parse(input).unwrap();
  match ser::expand_derive_serialize(&ast) {
    Ok(expanded) => expanded.into(),
    Err(msg) => panic!("{}", msg),
  }
}

#[proc_macro_derive(HexBinarySerde)]
pub fn derive_hexbinary(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, .. } = parse_macro_input!(input);
  quote! {
    impl std::fmt::Display for #ident {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", self.0)
      }
    }

    impl ::std::str::FromStr for #ident {
      type Err = ::std::string::String;

      fn from_str(s: &::std::primitive::str) -> ::std::result::Result<Self, Self::Err> {
        bitflags::parser::from_str(s).map_err(|e| e.to_string())
      }
    }
  }
  .into()
}
