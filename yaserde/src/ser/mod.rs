//! Generic data structure serialization framework.
//!

use crate::{YaSerialize, YaserdeWrite};
use std::io::{Cursor, Write};
use std::str;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

/// Serialize XML into a plain String with no formatting (EmitterConfig).
pub fn to_string(model: Box<dyn YaSerialize>) -> Result<String, String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer(model, Box::new(buf), &Config::default())?;
  let data = str::from_utf8(cursor.to_bytes()).expect("Found invalid UTF-8");
  Ok(data.into())
}
/// Serialize XML into a plain String with control on formatting (via EmitterConfig parameters)
pub fn to_string_with_config(
  model: Box<dyn YaSerialize>,
  config: &Config,
) -> Result<String, String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer(model, Box::new(buf), config)?;
  let data = str::from_utf8(cursor.to_bytes()).expect("Found invalid UTF-8");
  Ok(data.into())
}

pub fn serialize_with_writer(
  model: Box<dyn YaSerialize>,
  writer: Box<dyn YaserdeWrite>,
  config: &Config,
) -> Result<Box<dyn YaserdeWrite>, String> {
  let mut serializer = Serializer::new_from_writer(writer, config);
  match model.serialize(&mut serializer) {
    Ok(()) => Ok(serializer.into_inner()),
    Err(msg) => Err(msg),
  }
  // match YaSerialize::serialize(model, &mut serializer) {
  //   Ok(()) => Ok(serializer.into_inner()),
  //   Err(msg) => Err(msg),
  // }
}

pub fn to_string_content<T: YaSerialize>(model: &T) -> Result<String, String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer_content(model, Box::new(buf))?;
  let data = str::from_utf8(cursor.to_bytes()).expect("Found invalid UTF-8");
  Ok(data.into())
}

pub fn serialize_with_writer_content<T: YaSerialize>(
  model: &T,
  writer: Box<dyn YaserdeWrite>,
) -> Result<Box<dyn YaserdeWrite>, String> {
  let mut serializer = Serializer::new_for_inner(writer);
  serializer.set_skip_start_end(true);
  match YaSerialize::serialize(model, &mut serializer) {
    Ok(()) => Ok(serializer.into_inner()),
    Err(msg) => Err(msg),
  }
}

pub struct Serializer<W: Write> {
  writer: EventWriter<W>,
  skip_start_end: bool,
  generic: bool,
  start_event_name: Option<String>,
}

impl<'de, W: Write> Serializer<W> {
  pub fn new(writer: EventWriter<W>) -> Self {
    Serializer {
      writer,
      generic: false,
      skip_start_end: false,
      start_event_name: None,
    }
  }

  pub fn new_from_writer(writer: W, config: &Config) -> Self {
    let mut emitter_config = EmitterConfig::new()
      .cdata_to_characters(true)
      .perform_indent(config.perform_indent)
      .write_document_declaration(config.write_document_declaration);

    if let Some(indent_string_value) = &config.indent_string {
      emitter_config = emitter_config.indent_string(indent_string_value.clone());
    }

    Self::new(EventWriter::new_with_config(writer, emitter_config))
  }

  pub fn new_for_inner(writer: W) -> Self {
    let config = EmitterConfig::new().write_document_declaration(false);

    Self::new(EventWriter::new_with_config(writer, config))
  }

  pub fn into_inner(self) -> W {
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
      perform_indent: false,
      write_document_declaration: true,
      indent_string: None,
    }
  }
}
