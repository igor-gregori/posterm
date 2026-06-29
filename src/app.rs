use crate::http;
use crate::http::models::RequestModel;

#[derive(Clone, Copy, PartialEq)]
pub enum Panel {
    Sidebar,
    Request,
    Response,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RequestTab {
    Headers,
    Body,
    Params,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EditingField {
    Url,
    Headers,
    Body,
    Params,
}

pub struct App {
    pub running: bool,
    pub active_panel: Panel,
    pub request: RequestModel,
    pub request_tab: RequestTab,
    pub editing: Option<EditingField>,
    pub response: Option<Result<http::Response, String>>,
    pub loading: bool,
    pub tick: usize,
    // KV editor state
    pub kv_row: usize,
    pub kv_on_key: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            active_panel: Panel::Request,
            request: RequestModel::new(),
            request_tab: RequestTab::Headers,
            editing: None,
            response: None,
            loading: false,
            tick: 0,
            kv_row: 0,
            kv_on_key: true,
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Sidebar => Panel::Request,
            Panel::Request => Panel::Response,
            Panel::Response => Panel::Sidebar,
        };
    }
}
