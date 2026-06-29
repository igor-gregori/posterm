#[derive(Clone, Copy, PartialEq)]
pub enum Panel {
    Sidebar,
    Request,
    Response,
}

pub struct App {
    pub running: bool,
    pub active_panel: Panel,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            active_panel: Panel::Sidebar,
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
