#[macro_use]
extern crate yaserde;
#[macro_use]
extern crate yaserde_derive;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn root_flatten_struct() {
    init();

    #[derive(YaDeserialize, YaSerialize, PartialEq, Debug)]
    #[yaserde(flatten)]
    pub struct Content {
        binary_data: String,
        string_data: String,
    }

    let model = Content {
        binary_data: "binary".to_string(),
        string_data: "string".to_string(),
    };

    let content = "<binary_data>binary</binary_data><string_data>string</string_data>";

    serialize_and_validate!(model, content);
    deserialize_and_validate!(content, model, Content);
}

#[test]
fn flatten_attribute() {
    init();

    #[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
    struct HtmlText {
        #[yaserde(flatten)]
        text_attributes: TextAttributes,
        #[yaserde(attribute)]
        display: String,
    }

    #[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
    struct TextAttributes {
        #[yaserde(attribute)]
        bold: bool,
        #[yaserde(flatten)]
        font: FontAttributes,
    }

    #[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
    #[yaserde(namespace = "ns: http://www.sample.com/ns/domain")]
    pub struct FontAttributes {
        #[yaserde(attribute, prefix = "ns")]
        size: u32,
    }

    let model = HtmlText {
        text_attributes: TextAttributes {
            bold: true,
            font: FontAttributes { size: 24 },
        },
        display: "block".to_string(),
    };

    let content = r#"
    <HtmlText xmlns:ns="http://www.sample.com/ns/domain" display="block" bold="true" ns:size="24" />"#;

    serialize_and_validate!(model, content);
    deserialize_and_validate!(content, model, HtmlText);
}
