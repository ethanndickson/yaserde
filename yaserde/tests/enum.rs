#[macro_use]
extern crate yaserde;
#[macro_use]
extern crate yaserde_derive;

#[test]
fn basic_enum() {
    #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "base")]
    pub struct XmlStruct {
        color: Color,
    }

    #[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "color")]
    pub enum Color {
        #[default]
        White,
        Black,
    }

    assert_eq!(Color::default(), Color::White);

    #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
    pub struct RGBColor {
        red: String,
        green: String,
        blue: String,
    }

    #[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
    pub enum Alpha {
        #[default]
        Transparent,
        Opaque,
    }

    let model = XmlStruct {
        color: Color::Black,
    };

    let content = "<base><color>1</color></base>";
    serialize_and_validate!(model, content);
    deserialize_and_validate!(content, model, XmlStruct);
}

#[test]
fn attribute_enum() {
    #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "base")]
    pub struct XmlStruct {
        #[yaserde(attribute)]
        color: Color,
    }

    #[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "color")]
    pub enum Color {
        #[default]
        Pink,
    }

    let model = XmlStruct { color: Color::Pink };

    let content = r#"<base color="0" />"#;
    serialize_and_validate!(model, content);
    deserialize_and_validate!(content, model, XmlStruct);
}
