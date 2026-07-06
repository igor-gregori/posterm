use crate::http;
use crate::http::models::RequestModel;
use crate::storage::collections::{self, Collection};
use crate::storage::env::{self, Environments};
use crate::storage::history::{self, History};

const HEADER_PLACEHOLDERS: &[&str] = &[
    "Authorization: Bearer your-token-here",
    "Content-Type: application/json",
    "X-Custom-Header: something-cool",
    "Accept: application/json",
    "X-Request-Id: abc-123-xyz",
    "Cache-Control: no-cache",
    "User-Agent: posterm/0.2",
    "X-Api-Key: super-secret-key",
];

const PARAM_PLACEHOLDERS: &[&str] = &[
    "page=1",
    "limit=25",
    "search=awesome",
    "sort=created_at",
    "filter=active",
    "offset=0",
    "q=hello+world",
    "debug=true",
];

fn pick_placeholder(list: &'static [&'static str]) -> &'static str {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    list[seed % list.len()]
}

#[derive(Clone, Copy, PartialEq)]
pub enum Panel {
    Sidebar,
    Request,
    Response,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EditingField {
    Url,
    Headers,
    Body,
    Params,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dialog {
    SaveRequest,
    NewCollection,
    SelectEnv,
    NewEnv,
    EditEnvVars,
    Help,
    History,
    CurlExport,
}

pub struct App {
    pub running: bool,
    pub active_panel: Panel,
    pub request: RequestModel,
    pub editing: Option<EditingField>,
    pub response: Option<Result<http::Response, String>>,
    pub loading: bool,
    pub loading_since: std::time::Instant,
    pub tick: usize,
    pub response_scroll: usize,
    // Status message
    pub status_message: Option<String>,
    pub status_tick: usize,
    // cURL export
    pub curl_output: String,
    // KV editor state
    pub kv_row: usize,
    // Fixed placeholders (generated once per session)
    pub placeholder_header: &'static str,
    pub placeholder_param: &'static str,
    // Collections
    pub collections: Vec<Collection>,
    pub sidebar_collection: usize,
    pub sidebar_request: Option<usize>,
    pub sidebar_expanded: Option<usize>,
    // History
    pub history: History,
    pub history_selection: usize,
    // Dialog
    pub dialog: Option<Dialog>,
    pub dialog_input: String,
    pub dialog_selection: usize,
    // Text cursor
    pub cursor_pos: usize,
    // Environments
    pub environments: Environments,
    pub env_edit_row: usize,
    pub env_edit_vars: Vec<(String, String)>,
}

impl App {
    pub fn new() -> Self {
        let collections = collections::load_collections();
        let environments = env::load_environments();
        let history = history::load_history();
        Self {
            running: true,
            active_panel: Panel::Request,
            request: RequestModel::new(),
            editing: None,
            response: None,
            loading: false,
            loading_since: std::time::Instant::now(),
            tick: 0,
            response_scroll: 0,
            status_message: None,
            status_tick: 0,
            curl_output: String::new(),
            kv_row: 0,
            placeholder_header: pick_placeholder(HEADER_PLACEHOLDERS),
            placeholder_param: pick_placeholder(PARAM_PLACEHOLDERS),
            collections,
            sidebar_collection: 0,
            sidebar_request: None,
            sidebar_expanded: None,
            history,
            history_selection: 0,
            dialog: None,
            dialog_input: String::new(),
            dialog_selection: 0,
            cursor_pos: 0,
            environments,
            env_edit_row: 0,
            env_edit_vars: Vec::new(),
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Sidebar => Panel::Request,
            Panel::Request => Panel::Response,
            Panel::Response => Panel::Sidebar,
        };
    }

    pub fn active_env_name(&self) -> &str {
        self.environments
            .active
            .as_deref()
            .unwrap_or("none")
    }
}
