# SEPSerde (YaSerde for SEP)

A fork of [YaSerde](https://github.com/media-io/yaserde) for use in IEEE 2030.5 Client & Servers as part of Smart Energy Protocol 2.0 (SEP 2.0).

Allows for serializing and deserialising all IEEE 2030.5 Resources to and from their specification adhering XML representations.

This library will require a rewrite/refactor at some point in the near future, to remove unused functionality and to minimise code generation for the sake of binary size.

## Changes:

- Enums are serialized as their internal integer representations, as required by the IEEE 2030.5 specification, instead of string representations of their variant.

- Support for generic recursive types, as required by the IEEE 2030.5 Notificaton resource.
    - YaSerialize trait now implementations must provide the name of their type as a string literal for use in constructing `xsi:type` attributes.

- Allowed YaSerialize & YaDeserialize trait objects to be constructed.

- Imported utility proc macros from [xsd-parser-rs](https://github.com/lumeohq/xsd-parser-rs) to support serdeing of primitive newtypes.

- Support for serialising & deserialising `HexBinary\d+` types as per IEEE 2030.5.
