use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Padding},
};

#[derive(Clone)]
pub struct Border<'a> {
    block: Block<'a>,
}

impl<'a> Border<'a> {
    pub fn new() -> Self {
        let block = Block::bordered().border_type(BorderType::Rounded);

        Self { block }
    }

    pub fn border_type<T>(self, border_type: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        let border_type: Line<'_> = border_type.into();

        let block = self.block.title(border_type.left_aligned());

        Self { block }
    }

    pub fn horizontal(self, padding: u16) -> Self {
        self.padding(Padding::horizontal(padding))
    }

    pub fn padding(self, padding: Padding) -> Self {
        Self {
            block: self.block.padding(padding),
        }
    }

    pub fn status<T>(self, status: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        let status: Line<'_> = status.into();

        let block = self.block.title(status.right_aligned());

        Self { block }
    }

    pub fn title<T>(self, title: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        let title: Line<'_> = title.into();

        let block = self.block.title(title.bold().centered());

        Self { block }
    }

    pub fn uniform(self, padding: u16) -> Self {
        self.padding(Padding::uniform(padding))
    }

    pub fn vertical(self, padding: u16) -> Self {
        self.padding(Padding::vertical(padding))
    }
}

impl<'a> Default for Border<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a Border<'a>> for Block<'a> {
    fn from(border: &'a Border) -> Self {
        border.block.clone()
    }
}
