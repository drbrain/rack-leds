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

    pub fn build(self) -> Block<'a> {
        self.block
    }

    pub fn detail<T>(self, detail: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        let detail: Line<'_> = detail.into();

        let block = self.block.title(detail.right_aligned());

        Self { block }
    }

    pub fn name<T>(self, name: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        let name: Line<'_> = name.into();

        let block = self.block.title(name.left_aligned());

        Self { block }
    }

    pub fn help<T>(self, help: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        let help: Line<'_> = help.into();

        let block = self.block.title_bottom(help.right_aligned().italic());

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

    pub fn status<T>(self, title: T) -> Self
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
}

impl<'a> Default for Border<'a> {
    fn default() -> Self {
        Self::new()
    }
}
