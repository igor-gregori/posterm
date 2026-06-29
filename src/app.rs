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

impl RequestTab {
    pub fn next(&self) -> Self {
        match self {
            RequestTab::Headers => RequestTab::Body,
            RequestTab::Body => RequestTab::Params,
            RequestTab::Params => RequestTab::Headers,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RequestFocus {
    Method,
    Url,
    Tab,
}

pub struct App {
    pub running: bool,
    pub active_panel: Panel,
    pub request: RequestModel,
    pub request_tab: RequestTab,
    pub request_focus: RequestFocus,
    pub editing: bool,
    pub kv_row: usize,
    pub kv_on_key: bool,
    pub response: Option<Result<http::Response, String>>,
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            active_panel: Panel::Sidebar,
            request: RequestModel::new(),
            request_tab: RequestTab::Headers,
            request_focus: RequestFocus::Url,
            editing: false,
            kv_row: 0,
            kv_on_key: true,
            response: None,
            loading: false,
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
