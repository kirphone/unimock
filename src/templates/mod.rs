use std::collections::HashMap;

pub struct Template{
    text: String,
    id: u32,
    vars: HashMap<String, Variables>,
    body: TemplateBody
}

enum TemplateBody{
    XMLTemplateBody,
    JsonTemplateBody,
    RegexTemplateBody
}

enum Variables{
    ResponseHeader {key: String, value: String},
    FromHeader {key: String, value: String}
}

pub struct TemplateService{
    templates: HashMap<u32, Template>
}