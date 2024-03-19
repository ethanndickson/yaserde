use crate::common::YaSerdeAttribute;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_namespaces_definition(attributes: &YaSerdeAttribute) -> TokenStream {
    attributes
        .namespaces
        .iter()
        .map(|(prefix, namespace)| {
            if let Some(dn) = &attributes.default_namespace {
                if dn == prefix {
                    return quote!(
                      .default_ns(#namespace)
                    );
                }
            }
            quote!(
              .ns(#prefix, #namespace)
            )
        })
        .collect()
}
