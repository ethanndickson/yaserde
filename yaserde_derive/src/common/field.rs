use crate::common::attribute::YaSerdeAttribute;
use heck::ToUpperCamelCase;
use proc_macro2::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::fmt;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::Type::Path;

#[derive(Debug)]
pub struct YaSerdeField {
    syn_field: syn::Field,
    attributes: YaSerdeAttribute,
}

impl YaSerdeField {
    pub fn new(syn_field: syn::Field) -> Self {
        let attributes = YaSerdeAttribute::parse(&syn_field.attrs);

        YaSerdeField {
            syn_field,
            attributes,
        }
    }

    pub fn is_attribute(&self) -> bool {
        self.attributes.attribute
    }

    pub fn is_text_content(&self) -> bool {
        self.attributes.text
    }

    pub fn is_flatten(&self) -> bool {
        self.attributes.flatten
    }

    pub fn is_generic(&self) -> bool {
        self.attributes.generic
    }

    pub fn label(&self) -> Option<Ident> {
        self.syn_field.ident.clone()
    }

    pub fn is_skip_serializing(&self) -> bool {
        self.attributes.skip_serializing
    }

    pub fn get_value_label(&self) -> Option<syn::Ident> {
        self.syn_field
            .ident
            .clone()
            .map(|ident| syn::Ident::new(&format!("__{}_value", ident.unraw()), ident.span()))
    }

    pub fn renamed_label_without_namespace(&self) -> String {
        self.attributes
            .rename
            .clone()
            .unwrap_or_else(|| self.label().as_ref().unwrap().to_string())
    }

    pub fn renamed_label(&self, root_attributes: &YaSerdeAttribute) -> String {
        let prefix = if root_attributes.default_namespace == self.attributes.prefix {
            "".to_string()
        } else {
            self.attributes
                .prefix
                .clone()
                .map_or("".to_string(), |prefix| prefix + ":")
        };

        let label = self.renamed_label_without_namespace();

        format!("{}{}", prefix, label)
    }

    pub fn get_visitor_ident(&self, struct_name: Option<&syn::Path>) -> Ident {
        let label = self.renamed_label_without_namespace();

        let struct_id = struct_name.map_or_else(
            || "".to_string(),
            |struct_name| {
                struct_name
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect()
            },
        );

        Ident::new(
            &format!(
                "__Visitor_{}_{}",
                label.replace('.', "_").to_upper_camel_case(),
                struct_id
            ),
            self.get_span(),
        )
    }

    pub fn get_type(&self) -> Field {
        Field::from(&self.syn_field)
    }

    pub fn get_span(&self) -> Span {
        self.syn_field.span()
    }

    pub fn get_default_function(&self) -> Option<Ident> {
        self.attributes
            .default
            .as_ref()
            .map(|default| Ident::new(default, self.get_span()))
    }

    pub fn get_skip_serializing_if_function(&self) -> Option<Ident> {
        self.attributes
            .skip_serializing_if
            .as_ref()
            .map(|skip_serializing_if| Ident::new(skip_serializing_if, self.get_span()))
    }

    pub fn get_namespace_matching(
        &self,
        root_attributes: &YaSerdeAttribute,
        element_namespace: TokenStream,
        element_name: TokenStream,
    ) -> TokenStream {
        root_attributes.get_namespace_matching(
            &self.attributes.prefix,
            element_namespace,
            element_name,
            false,
        )
    }

    pub fn ser_wrap_default_attribute(
        &self,
        builder: Option<TokenStream>,
        setter: TokenStream,
    ) -> TokenStream {
        let label = self.label();

        let yaserde_inner_definition = builder
            .map(|builder| quote!(let yaserde_inner = #builder;))
            .unwrap_or_default();

        let skip_if = self
            .get_skip_serializing_if_function()
            .map(|skip_if_function| quote!(!self.#skip_if_function(&self.#label)))
            .unwrap_or(quote!(true));

        self.get_default_function()
            .map(|default_function| {
                quote! {
                  #yaserde_inner_definition
                  let struct_start_event =
                    if #skip_if && self.#label != #default_function() {
                      #setter
                    } else {
                      struct_start_event
                    };
                }
            })
            .unwrap_or(quote! {
              #yaserde_inner_definition
              let struct_start_event = if #skip_if { #setter } else { struct_start_event };
            })
    }
}

#[derive(Debug)]
pub enum Field {
    String,
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Option { data_type: Box<Field> },
    Vec { data_type: Box<Field> },
    Struct { struct_name: syn::Path },
}

impl Field {
    pub fn get_simple_type_visitor(&self) -> Ident {
        format_ident!("visit_{}", self.to_string())
    }
}

impl From<&syn::Path> for Field {
    fn from(path: &syn::Path) -> Self {
        let result = if let Some(segment) = path.segments.last() {
            match segment.ident.to_string().as_str() {
                "String" => Some(Field::String),
                "bool" => Some(Field::Bool),
                "i8" => Some(Field::I8),
                "u8" => Some(Field::U8),
                "i16" => Some(Field::I16),
                "u16" => Some(Field::U16),
                "i32" => Some(Field::I32),
                "u32" => Some(Field::U32),
                "i64" => Some(Field::I64),
                "u64" => Some(Field::U64),
                "f32" => Some(Field::F32),
                "f64" => Some(Field::F64),
                "Option" => Some(Field::Option {
                    data_type: Box::new(Field::from(segment)),
                }),
                "Vec" => Some(Field::Vec {
                    data_type: Box::new(Field::from(segment)),
                }),
                _ => None,
            }
        } else {
            None
        };

        result.unwrap_or_else(|| Field::Struct {
            struct_name: path.clone(),
        })
    }
}

impl From<&syn::Field> for Field {
    fn from(field: &syn::Field) -> Self {
        let mut ty = &field.ty;
        while let syn::Type::Group(g) = ty {
            ty = &g.elem;
        }
        match ty {
            Path(ref path) => Field::from(&path.path),
            _ => panic!("unable to match {:?}", field.ty),
        }
    }
}

impl From<&syn::PathSegment> for Field {
    fn from(path_segment: &syn::PathSegment) -> Self {
        if let syn::PathArguments::AngleBracketed(ref args) = path_segment.arguments {
            if let Some(syn::GenericArgument::Type(Path(ref path))) = args.args.first() {
                return Field::from(&path.path);
            }
        }
        unimplemented!()
    }
}

impl From<Field> for proc_macro2::TokenStream {
    fn from(field: Field) -> proc_macro2::TokenStream {
        match field {
            Field::String => quote! { ::std::string::String },
            Field::Bool => quote! { bool },
            Field::I8 => quote! { i8 },
            Field::U8 => quote! { u8 },
            Field::I16 => quote! { i16 },
            Field::U16 => quote! { u16 },
            Field::I32 => quote! { i32 },
            Field::U32 => quote! { u32 },
            Field::I64 => quote! { i64 },
            Field::U64 => quote! { u64 },
            Field::F32 => quote! { f32 },
            Field::F64 => quote! { f64 },
            _ => panic!("Not a simple type: {:?}", field),
        }
    }
}

impl From<&Field> for String {
    fn from(field: &Field) -> String {
        match field {
            Field::String => "str".to_string(),
            Field::Bool => "bool".to_string(),
            Field::I8 => "i8".to_string(),
            Field::U8 => "u8".to_string(),
            Field::I16 => "i16".to_string(),
            Field::U16 => "u16".to_string(),
            Field::I32 => "i32".to_string(),
            Field::U32 => "u32".to_string(),
            Field::I64 => "i64".to_string(),
            Field::U64 => "u64".to_string(),
            Field::F32 => "f32".to_string(),
            Field::F64 => "f64".to_string(),
            _ => panic!("Not a simple type: {:?}", field),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_representation: String = self.into();
        write!(f, "{}", string_representation)
    }
}
