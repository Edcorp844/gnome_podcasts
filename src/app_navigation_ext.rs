use relm4::{ComponentController, Controller};

use crate::pages::{home::HomePage, new::NewPage, search::SearchPage, shows::ShowsPage};

#[derive(Debug)]
pub enum NavigationPage {
    Search,
    Home,
    New,
    Shows,
    Library(String),
}

impl NavigationPage {
    pub fn from_name(name: &str) -> Self {
        match name {
            "Search" => Self::Search,
            "Home" => Self::Home,
            "New" => Self::New,
            "Shows" => Self::Shows,
            other => Self::Library(other.to_string()),
        }
    }

    pub fn to_key(&self) -> String {
        match self {
            Self::Search => "Search".to_string(),
            Self::Home => "Home".to_string(),
            Self::New => "New".to_string(),
            Self::Shows => "Shows".to_string(),
            Self::Library(sub) => format!("Library_{}", sub),
        }
    }
}

pub enum PageController {
    Search(Controller<SearchPage>),
    Home(Controller<HomePage>),
    New(Controller<NewPage>),
    Shows(Controller<ShowsPage>),
    //Library(Controller<LibraryPage>),
}

impl PageController {
    pub(crate) fn widget(&self) -> &adw::NavigationPage {
        match self {
            Self::Search(c) => c.widget(),
            Self::Home(c) => c.widget(),
            Self::New(c) => c.widget(),
            Self::Shows(c) => c.widget(),
        }
    }
}
