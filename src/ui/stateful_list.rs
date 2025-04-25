use ratatui::{
    style::{Color, Style},
    text::Span,
    widgets::{List, ListItem, ListState},
};

use crate::api::data::{Channel, Guild, Msg};

#[derive(Debug, Clone)]
pub struct StatefulList<Element> {
    pub state: ListState,
    pub items: Vec<Element>,
}

impl<Element> From<StatefulList<Element>> for List<'_>
where
    for<'a> Element: Into<ListItem<'a>>,
{
    fn from(value: StatefulList<Element>) -> Self {
        List::new(value.items)
    }
}

impl<Element> StatefulList<Element> {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

impl<Element> From<Vec<Element>> for StatefulList<Element> {
    fn from(value: Vec<Element>) -> Self {
        StatefulList {
            state: ListState::default(),
            items: value,
        }
    }
}

impl From<Msg> for ListItem<'_> {
    fn from(value: Msg) -> Self {
        let name = value.user.name;
        let content = value.content;
        let combo = format!("{}: {}", name, content);
        ListItem::new(combo)
    }
}

impl From<Guild> for ListItem<'_> {
    fn from(value: Guild) -> Self {
        let text = value.name.clone();
        ListItem::new(text).style(Style::default().fg(Color::Black).bg(Color::White))
    }
}

impl From<Channel> for ListItem<'_> {
    fn from(value: Channel) -> Self {
        let text = value.name.clone();
        ListItem::new(text).style(Style::default().fg(Color::Black).bg(Color::White))
    }
}
