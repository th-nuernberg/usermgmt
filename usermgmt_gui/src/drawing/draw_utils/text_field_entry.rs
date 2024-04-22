#[derive(Debug)]
pub struct TextFieldEntry<'a, 'b> {
    label: &'a str,
    content: ContentField<'a>,
    tool_tip: Option<&'b str>,
    as_password: bool,
}

#[derive(Debug)]
enum ContentField<'a> {
    Required(&'a mut String),
    Optional(&'a mut Option<String>),
}

impl<'a, 'b> TextFieldEntry<'a, 'b> {
    pub fn content(&self) -> &str {
        match &self.content {
            ContentField::Required(something) => something.as_str(),
            ContentField::Optional(Some(something)) => something.as_str(),
            ContentField::Optional(None) => "",
        }
    }

    pub fn set_content(&mut self, content: String) {
        match &mut self.content {
            ContentField::Required(required) => **required = content,
            ContentField::Optional(optional) => **optional = Some(content),
        }
    }

    pub fn label(&self) -> &str {
        self.label
    }

    pub fn tool_tip(&self) -> Option<&str> {
        self.tool_tip
    }

    pub fn as_password(&self) -> bool {
        self.as_password
    }

    pub fn new(label: &'a str, content: &'a mut String) -> Self {
        Self {
            label,
            content: ContentField::Required(content),
            as_password: false,
            tool_tip: None,
        }
    }

    pub fn new_opt(label: &'a str, content: &'a mut Option<String>) -> Self {
        Self {
            label,
            content: ContentField::Optional(content),
            as_password: false,
            tool_tip: None,
        }
    }

    pub fn with_as_password(mut self) -> Self {
        self.as_password = true;
        self
    }

    pub fn with_tool_tip(self, too_tip: Option<&'b str>) -> Self {
        Self {
            label: self.label,
            content: self.content,
            as_password: self.as_password,
            tool_tip: too_tip,
        }
    }

    pub fn with_tooltip(self, tooltip: &'b str) -> Self {
        Self {
            label: self.label,
            content: self.content,
            as_password: self.as_password,
            tool_tip: Some(tooltip),
        }
    }
}
