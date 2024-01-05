use crate::common::{Field, YaSerdeAttribute, YaSerdeField};
use crate::ser::{implement_serializer::implement_serializer, label::build_label_name};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Fields;
use syn::Ident;
use syn::{DataEnum, Generics};

pub fn serialize(
    data_enum: &DataEnum,
    name: &Ident,
    root: &str,
    root_attributes: &YaSerdeAttribute,
    generics: &Generics,
) -> TokenStream {
    let inner_enum_inspector = inner_enum_inspector(data_enum, name, root_attributes);

    implement_serializer(
        name,
        root,
        root_attributes,
        quote!(),
        quote!(match self {
          #inner_enum_inspector
        }),
        generics,
    )
}

fn inner_enum_inspector(
    data_enum: &DataEnum,
    name: &Ident,
    root_attributes: &YaSerdeAttribute,
) -> TokenStream {
    data_enum
    .variants
    .iter()
    .map(|variant| {
      let variant_attrs = YaSerdeAttribute::parse(&variant.attrs);

      let label = &variant.ident;
      let label_name = build_label_name(label, &variant_attrs, &root_attributes.default_namespace);

      match variant.fields {
        Fields::Unit => quote! {
          &#name::#label => {
            let internal = format!("{}",#name::#label as u32);
            let data_event = ::yaserde::xml::writer::XmlEvent::characters(&internal);
            writer.write(data_event).map_err(|e| e.to_string())?;
          }
        },
        Fields::Named(ref fields) => {
          let enum_fields: TokenStream = fields
            .named
            .iter()
            .map(|field| YaSerdeField::new(field.clone()))
            .filter(|field| !field.is_attribute())
            .filter_map(|field| {
              let field_label = field.label();

              if field.is_text_content() {
                return Some(quote!(
                  let data_event = ::yaserde::xml::writer::XmlEvent::characters(&self.#field_label);
                  writer.write(data_event).map_err(|e| e.to_string())?;
                ));
              }

              let field_label_name = field.renamed_label(root_attributes);

              match field.get_type() {
                Field::String
                | Field::Bool
                | Field::U8
                | Field::I8
                | Field::U16
                | Field::I16
                | Field::U32
                | Field::I32
                | Field::F32
                | Field::U64
                | Field::I64
                | Field::F64 => {
                  Some(quote! {
                    match self {
                      &#name::#label { ref #field_label, .. } => {
                        let struct_start_event =
                          ::yaserde::xml::writer::XmlEvent::start_element(#field_label_name);
                        writer.write(struct_start_event).map_err(|e| e.to_string())?;

                        let string_value = #field_label.to_string();
                        let data_event = ::yaserde::xml::writer::XmlEvent::characters(&string_value);
                        writer.write(data_event).map_err(|e| e.to_string())?;

                        let struct_end_event = ::yaserde::xml::writer::XmlEvent::end_element();
                        writer.write(struct_end_event).map_err(|e| e.to_string())?;
                      },
                      _ => {},
                    }
                  })
                },
                Field::Struct { .. } => Some(quote! {
                  match self {
                    &#name::#label{ref #field_label, ..} => {
                      writer.set_start_event_name(
                        ::std::option::Option::Some(#field_label_name.to_string()),
                      );
                      writer.set_skip_start_end(false);
                      ::yaserde::YaSerialize::serialize(#field_label, writer)?;
                    },
                    _ => {}
                  }
                }),
                Field::Vec { .. } => Some(quote! {
                  match self {
                    &#name::#label { ref #field_label, .. } => {
                      for item in #field_label {
                        writer.set_start_event_name(
                          ::std::option::Option::Some(#field_label_name.to_string()),
                        );
                        writer.set_skip_start_end(false);
                        ::yaserde::YaSerialize::serialize(item, writer)?;
                      }
                    },
                    _ => {}
                  }
                }),
                Field::Option { .. } => None,
              }
            })
            .collect();

          quote! {
            &#name::#label{..} => {
              #enum_fields
            }
          }
        }
        Fields::Unnamed(ref fields) => {
          let enum_fields: TokenStream = fields
            .unnamed
            .iter()
            .map(|field| YaSerdeField::new(field.clone()))
            .filter(|field| !field.is_attribute())
            .map(|field| {
              let write_element = |action: &TokenStream| {
                quote! {
                  let struct_start_event = ::yaserde::xml::writer::XmlEvent::start_element(#label_name);
                  writer.write(struct_start_event).map_err(|e| e.to_string())?;

                  #action

                  let struct_end_event = ::yaserde::xml::writer::XmlEvent::end_element();
                  writer.write(struct_end_event).map_err(|e| e.to_string())?;
                }
              };

              let write_string_chars = quote! {
                let data_event = ::yaserde::xml::writer::XmlEvent::characters(item);
                writer.write(data_event).map_err(|e| e.to_string())?;
              };

              let write_simple_type = write_element(&quote! {
                let s = item.to_string();
                let data_event = ::yaserde::xml::writer::XmlEvent::characters(&s);
                writer.write(data_event).map_err(|e| e.to_string())?;
              });

              let serialize = quote! {
                writer.set_start_event_name(::std::option::Option::None);
                writer.set_skip_start_end(true);
                ::yaserde::YaSerialize::serialize(item, writer)?;
              };

              let write_sub_type = |data_type| {
                write_element(match data_type {
                  Field::String => &write_string_chars,
                  _ => &serialize,
                })
              };

              let match_field = |write: &TokenStream| {
                quote! {
                  match self {
                    &#name::#label(ref item) => {
                      #write
                    },
                    _ => {},
                  }
                }
              };

              match field.get_type() {
                Field::Option { data_type } => {
                  let write = write_sub_type(*data_type);

                  match_field(&quote! {
                    if let ::std::option::Option::Some(item) = item {
                      #write
                    }
                  })
                }
                Field::Vec { data_type } => {
                  let write = write_sub_type(*data_type);

                  match_field(&quote! {
                    for item in item {
                      #write
                    }
                  })
                }
                Field::Struct { .. } => write_element(&match_field(&serialize)),
                Field::String => match_field(&write_element(&write_string_chars)),
                _simple_type => match_field(&write_simple_type),
              }
            })
            .collect();

          quote! {
            &#name::#label{..} => {
              #enum_fields
            }
          }
        }
      }
    })
    .collect()
}
