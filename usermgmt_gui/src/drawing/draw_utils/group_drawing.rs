#[derive(Debug)]
pub struct GroupDrawing<'a, 'b> {
    name: &'a str,
    tooltip: Option<&'b str>,
}

impl<'a, 'b> GroupDrawing<'a, 'b> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            tooltip: None,
        }
    }

    pub fn with_tooltip(self, tooltip: Option<&'b str>) -> GroupDrawing<'a, 'b> {
        Self {
            name: self.name,
            tooltip,
        }
    }

    pub fn add_tooltip(self, text: &'b str) -> GroupDrawing<'a, 'b> {
        Self {
            name: self.name,
            tooltip: Some(text),
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn tooltip(&self) -> Option<&str> {
        self.tooltip
    }
}
