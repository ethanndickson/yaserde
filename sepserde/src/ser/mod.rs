//! Generic data structure serialization framework.
//!

use crate::YaSerialize;

use alloc::string::{String, ToString};
pub use xml_no_std as xml;
use xml_no_std::writer::XmlEvent;
use xml_no_std::{EmitterConfig, EventWriter};

/// Serialize XML into a plain String with IEEE 2030.5 Formatting
pub fn to_string<T: YaSerialize>(model: &T) -> Result<String, String> {
    let data = serialize_with_writer(model, &Config::default())?;
    Ok(data)
}

/// Serialize XML into a plain String with control on formatting (via EmitterConfig parameters)
pub fn to_string_with_config<T: YaSerialize>(model: &T, config: &Config) -> Result<String, String> {
    let data = serialize_with_writer(model, config)?;
    Ok(data)
}

pub fn serialize_with_writer<T: YaSerialize>(model: &T, config: &Config) -> Result<String, String> {
    let mut serializer = Serializer::new_from_writer(config);
    match YaSerialize::serialize(model, &mut serializer) {
        Ok(()) => Ok(serializer.into_inner()),
        Err(msg) => Err(msg),
    }
}

pub fn to_string_content<T: YaSerialize>(model: &T) -> Result<String, String> {
    let data = serialize_with_writer_content(model)?;
    Ok(data)
}

pub fn serialize_with_writer_content<T: YaSerialize>(model: &T) -> Result<String, String> {
    let mut serializer = Serializer::new_for_inner();
    serializer.set_skip_start_end(true);
    match YaSerialize::serialize(model, &mut serializer) {
        Ok(()) => Ok(serializer.into_inner()),
        Err(msg) => Err(msg),
    }
}

pub struct Serializer {
    writer: EventWriter,
    skip_start_end: bool,
    generic: bool,
    start_event_name: Option<String>,
}

impl Serializer {
    pub fn new(writer: EventWriter) -> Self {
        Serializer {
            writer,
            generic: false,
            skip_start_end: false,
            start_event_name: None,
        }
    }

    pub fn new_from_writer(config: &Config) -> Self {
        let mut emitter_config = EmitterConfig::new()
            .cdata_to_characters(true)
            .perform_indent(config.perform_indent)
            .write_document_declaration(config.write_document_declaration);

        if let Some(indent_string_value) = &config.indent_string {
            emitter_config = emitter_config.indent_string(indent_string_value.clone());
        }

        Self::new(EventWriter::new_with_config(emitter_config))
    }

    pub fn new_for_inner() -> Self {
        let config = EmitterConfig::new().write_document_declaration(false);

        Self::new(EventWriter::new_with_config(config))
    }

    pub fn into_inner(self) -> String {
        self.writer.into_inner()
    }

    pub fn skip_start_end(&self) -> bool {
        self.skip_start_end
    }

    pub fn generic(&self) -> bool {
        self.generic
    }

    pub fn set_skip_start_end(&mut self, state: bool) {
        self.skip_start_end = state;
    }

    pub fn get_start_event_name(&self) -> Option<String> {
        self.start_event_name.clone()
    }

    pub fn set_start_event_name(&mut self, name: Option<String>) {
        self.start_event_name = name;
    }

    pub fn set_generic(&mut self, state: bool) {
        self.generic = state;
    }

    pub fn write<'a, E>(&mut self, event: E) -> xml::writer::Result<()>
    where
        E: Into<XmlEvent<'a>>,
    {
        self.writer.write(event)
    }
}

pub struct Config {
    pub perform_indent: bool,
    pub write_document_declaration: bool,
    pub indent_string: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            perform_indent: true,
            write_document_declaration: false,
            indent_string: None,
        }
    }
}
