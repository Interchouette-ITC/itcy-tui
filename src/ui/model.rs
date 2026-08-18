// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! TUI application state (no HTTP).

use super::hits::HitMap;
use super::ViewMode;
use crate::artefact::Artefact;
use crate::commands::SLASH_COMMANDS;
use crate::github::{PubsBranch, PubsRemote, TreeFetch};
use crate::health::{replace_health_path, HealthStatus, DEFAULT_INGRESS_HEALTH_URL};
use crate::line_edit::LineEditor;
use crate::palette::CommandHistory;
use crate::saved_parse::SavedRow;
use crate::status::RuntimeStatus;
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::time::Instant;

/// Keyboard overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Keys are view/global.
    #[default]
    Normal,
    /// `/` filter textarea.
    Filter,
    /// `:` command textarea.
    Command,
}

/// Which pubs pane receives `j`/`k` and wheel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PubsFocus {
    /// Artefact table.
    #[default]
    List,
    /// `body.md` preview.
    Preview,
}

/// I/O the event loop should spawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoNeed {
    /// Nothing.
    None,
    /// GitHub tree for current remote+branch.
    Tree,
    /// Body+meta for the selected artefact.
    Preview,
    /// `POST /entrypoint/slash` `/list`.
    SavedList,
}

/// Pane model.
pub struct StatusModel {
    /// Product `/health` URL.
    pub health_url: String,
    /// Product `/status` URL.
    pub status_url: String,
    /// Product webhook wake URL.
    pub webhook_url: String,
    /// Ingress health URL.
    pub ingress_url: String,
    /// Last product health.
    pub health: HealthStatus,
    /// Last ingress health.
    pub ingress_health: HealthStatus,
    /// Last `/status` JSON.
    pub runtime: Option<RuntimeStatus>,
    /// Tick counter (spinner + polls).
    pub ticks: u64,
    /// Active view.
    pub view: ViewMode,
    /// View under Help.
    pub help_from: ViewMode,
    /// Overlay mode.
    pub input: InputMode,
    /// Filter / command editor.
    pub textarea: LineEditor,
    /// Colon history.
    pub history: CommandHistory,
    /// Tab-complete list.
    pub completions: Vec<String>,
    /// Tab-complete index.
    pub completion_idx: Option<usize>,
    /// Publications.
    pub pubs: PubsPane,
    /// Saved `/list` rows.
    pub saved_rows: Vec<SavedRow>,
    /// Saved-list error.
    pub saved_error: String,
    /// Commands table selection.
    pub cmd_state: TableState,
    /// Commands filter.
    pub cmd_filter: String,
    /// Saved table selection.
    pub saved_state: TableState,
    /// Saved filter.
    pub saved_filter: String,
    /// Live body scroll.
    pub live_scroll: u16,
    /// Help scroll.
    pub help_scroll: u16,
    /// Tree/preview/list in flight.
    pub pending: bool,
    /// Footer `copied` flash.
    pub copied: bool,
    /// Last mouse hit map.
    pub hits: HitMap,
    /// Status/error line (palette unknown, etc.).
    pub notice: String,
}

/// Publications browser.
pub struct PubsPane {
    /// Org or fork.
    pub remote: PubsRemote,
    /// Branch.
    pub branch: PubsBranch,
    /// Current tree.
    pub artefacts: Vec<Artefact>,
    /// Table state (selected + offset).
    pub table: TableState,
    /// Filter string (mirrors textarea when Filter).
    pub filter: String,
    /// Loaded body.
    pub body: String,
    /// Preview scroll.
    pub body_scroll: u16,
    /// Last error.
    pub error: String,
    /// Rate remaining.
    pub rate_remaining: Option<u32>,
    /// List vs preview.
    pub focus: PubsFocus,
    /// Tree cache.
    pub cache: HashMap<(PubsRemote, PubsBranch), TreeFetch>,
    /// Body cache.
    pub bodies: HashMap<(PubsRemote, PubsBranch, String), String>,
    /// Selection generation (stale preview).
    pub select_gen: u64,
    /// When selection last changed.
    pub last_select: Option<Instant>,
    /// Preview load pending debounce.
    pub preview_dirty: bool,
}

impl Default for PubsPane {
    fn default() -> Self {
        let mut table = TableState::default();
        table.select(Some(0));
        Self {
            remote: PubsRemote::Org,
            branch: PubsBranch::Drafts,
            artefacts: Vec::new(),
            table,
            filter: String::new(),
            body: String::new(),
            body_scroll: 0,
            error: String::new(),
            rate_remaining: None,
            focus: PubsFocus::List,
            cache: HashMap::new(),
            bodies: HashMap::new(),
            select_gen: 0,
            last_select: None,
            preview_dirty: false,
        }
    }
}

impl PubsPane {
    /// Apply a tree fetch into the pane and cache.
    pub fn apply_tree(&mut self, fetch: TreeFetch) {
        self.rate_remaining = fetch.rate_remaining;
        self.cache.insert((self.remote, self.branch), fetch.clone());
        self.artefacts = fetch.artefacts;
        self.table.select(Some(0));
        self.body.clear();
        self.body_scroll = 0;
        self.error = fetch.error.unwrap_or_default();
        self.bump_select();
    }

    /// Use cache or ask the loop to fetch.
    pub fn set_remote(&mut self, remote: PubsRemote) -> IoNeed {
        if self.remote == remote {
            return IoNeed::None;
        }
        self.remote = remote;
        self.load_cached_or_need()
    }

    /// Jump branch.
    pub fn set_branch(&mut self, branch: PubsBranch) -> IoNeed {
        if self.branch == branch {
            return IoNeed::None;
        }
        self.branch = branch;
        self.load_cached_or_need()
    }

    fn load_cached_or_need(&mut self) -> IoNeed {
        let key = (self.remote, self.branch);
        self.cache.get(&key).cloned().map_or(IoNeed::Tree, |fetch| {
            self.apply_tree(fetch);
            IoNeed::None
        })
    }

    /// Filtered artefact indexes (id + subject).
    #[must_use]
    pub fn filtered_indexes(&self) -> Vec<usize> {
        let q = self.filter.to_ascii_lowercase();
        self.artefacts
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                q.is_empty()
                    || a.id.to_ascii_lowercase().contains(&q)
                    || a.subject.to_ascii_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Selected artefact.
    #[must_use]
    pub fn selected_artefact(&self) -> Option<&Artefact> {
        let idx = *self.filtered_indexes().get(self.selected())?;
        self.artefacts.get(idx)
    }

    /// Selected row in the filtered list.
    #[must_use]
    pub fn selected(&self) -> usize {
        self.table.selected().unwrap_or(0)
    }

    fn clamp_selected(&mut self) {
        let n = self.filtered_indexes().len();
        if n == 0 {
            self.table.select(Some(0));
        } else if self.selected() >= n {
            self.table.select(Some(n.saturating_sub(1)));
        }
    }

    /// Select a visible filtered row (mouse click).
    pub fn select_visible(&mut self, idx: usize) {
        let n = self.filtered_indexes().len();
        if idx >= n {
            return;
        }
        self.table.select(Some(idx));
        self.bump_select();
    }

    /// Move selection; marks preview dirty.
    pub fn move_selection(&mut self, delta: i32) {
        let n = i32::try_from(self.filtered_indexes().len()).unwrap_or(0);
        if n == 0 {
            self.table.select(Some(0));
            return;
        }
        let cur = i32::try_from(self.selected()).unwrap_or(0);
        let next = (cur + delta).rem_euclid(n);
        self.table.select(Some(usize::try_from(next).unwrap_or(0)));
        self.bump_select();
    }

    /// First / last row.
    pub fn select_edge(&mut self, last: bool) {
        let n = self.filtered_indexes().len();
        if n == 0 {
            return;
        }
        self.table.select(Some(if last { n - 1 } else { 0 }));
        self.bump_select();
    }

    /// Half-page.
    pub fn page(&mut self, down: bool) {
        let step = 8_i32;
        self.move_selection(if down { step } else { -step });
    }

    fn bump_select(&mut self) {
        self.select_gen = self.select_gen.saturating_add(1);
        self.last_select = Some(Instant::now());
        self.preview_dirty = true;
        self.body_scroll = 0;
        if let Some(art) = self.selected_artefact() {
            let key = (self.remote, self.branch, art.body_path.clone());
            if let Some(body) = self.bodies.get(&key) {
                self.body.clone_from(body);
                self.preview_dirty = false;
            } else {
                self.body.clear();
            }
        }
    }

    /// Select first filtered id containing `needle` (case-insensitive).
    pub fn open_prefix(&mut self, needle: &str) -> bool {
        let q = needle.to_ascii_lowercase();
        let indexes = self.filtered_indexes();
        for (vis, &idx) in indexes.iter().enumerate() {
            if self.artefacts[idx].id.to_ascii_lowercase().contains(&q) {
                self.table.select(Some(vis));
                self.bump_select();
                return true;
            }
        }
        false
    }

    /// Store preview result if `gen` is still current.
    pub fn apply_preview(
        &mut self,
        gen: u64,
        id: &str,
        body: Result<String, String>,
        subject: String,
    ) {
        if gen != self.select_gen {
            return;
        }
        match body {
            Ok(text) => {
                if let Some(art) = self.selected_artefact() {
                    let key = (self.remote, self.branch, art.body_path.clone());
                    self.bodies.insert(key, text.clone());
                }
                self.body = text;
                self.error.clear();
            }
            Err(e) => {
                self.body.clear();
                self.error = e;
            }
        }
        if let Some(a) = self.artefacts.iter_mut().find(|a| a.id == id) {
            a.subject.clone_from(&subject);
        }
        if let Some(fetch) = self.cache.get_mut(&(self.remote, self.branch)) {
            if let Some(a) = fetch.artefacts.iter_mut().find(|a| a.id == id) {
                a.subject = subject;
            }
        }
        self.preview_dirty = false;
    }

    /// Scroll preview.
    pub fn scroll_body(&mut self, delta: i16) {
        if delta < 0 {
            self.body_scroll = self
                .body_scroll
                .saturating_sub(u16::try_from(-delta).unwrap_or(0));
        } else {
            self.body_scroll = self
                .body_scroll
                .saturating_add(u16::try_from(delta).unwrap_or(0));
        }
    }

    /// Sync filter from overlay and clamp.
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.clamp_selected();
    }
}

fn hooks_url_from_health(health_url: &str) -> String {
    replace_health_path(health_url, "/hooks/github")
        .unwrap_or_else(|| format!("{}/hooks/github", health_url.trim_end_matches('/')))
}

impl StatusModel {
    /// Build from probe snapshots.
    pub fn new(
        health_url: impl Into<String>,
        status_url: impl Into<String>,
        health: HealthStatus,
        runtime: Option<RuntimeStatus>,
    ) -> Self {
        let health_url = health_url.into();
        let webhook_url = hooks_url_from_health(&health_url);
        let mut cmd_state = TableState::default();
        cmd_state.select(Some(0));
        let mut saved_state = TableState::default();
        saved_state.select(Some(0));
        Self {
            health_url,
            status_url: status_url.into(),
            webhook_url,
            ingress_url: DEFAULT_INGRESS_HEALTH_URL.to_string(),
            health,
            ingress_health: HealthStatus::Down {
                reason: "not probed".into(),
            },
            runtime,
            ticks: 0,
            view: ViewMode::Live,
            help_from: ViewMode::Live,
            input: InputMode::Normal,
            textarea: LineEditor::default(),
            history: CommandHistory::default(),
            completions: Vec::new(),
            completion_idx: None,
            pubs: PubsPane::default(),
            saved_rows: Vec::new(),
            saved_error: String::new(),
            cmd_state,
            cmd_filter: String::new(),
            saved_state,
            saved_filter: String::new(),
            live_scroll: 0,
            help_scroll: 0,
            pending: false,
            copied: false,
            hits: HitMap::default(),
            notice: String::new(),
        }
    }

    /// Overlay is open.
    #[must_use]
    pub const fn overlay_open(&self) -> bool {
        matches!(self.input, InputMode::Filter | InputMode::Command)
    }

    /// Land on pubs when product is down.
    pub fn apply_boot_view(&mut self) -> IoNeed {
        if matches!(self.health, HealthStatus::Ok) {
            return IoNeed::None;
        }
        self.show_publications()
    }

    /// Open live.
    pub const fn show_live(&mut self) {
        self.view = ViewMode::Live;
    }

    /// Open commands.
    pub const fn show_commands(&mut self) {
        self.view = ViewMode::Commands;
    }

    /// Open publications (fetch if empty).
    pub fn show_publications(&mut self) -> IoNeed {
        self.view = ViewMode::Publications;
        if self.pubs.artefacts.is_empty() && self.pubs.cache.is_empty() {
            IoNeed::Tree
        } else if self.pubs.artefacts.is_empty() {
            self.pubs.load_cached_or_need()
        } else {
            IoNeed::None
        }
    }

    /// Open saved list; caller fetches when `SavedList` returned.
    pub fn show_saved_list(&mut self) -> IoNeed {
        self.view = ViewMode::SavedList;
        if matches!(self.health, HealthStatus::Ok) {
            self.saved_error.clear();
            IoNeed::SavedList
        } else {
            self.saved_rows.clear();
            self.saved_error = "ITCy down - /list needs localhost :4700".into();
            IoNeed::None
        }
    }

    /// Open help, remembering the previous view.
    pub fn show_help(&mut self) {
        if self.view != ViewMode::Help {
            self.help_from = self.view;
        }
        self.view = ViewMode::Help;
    }

    /// Leave help.
    pub fn leave_help(&mut self) {
        if self.view == ViewMode::Help {
            self.view = self.help_from;
        }
    }

    /// Begin `/` filter.
    pub fn begin_filter(&mut self) {
        self.input = InputMode::Filter;
        self.textarea = LineEditor::default();
        let current = match self.view {
            ViewMode::Publications => self.pubs.filter.clone(),
            ViewMode::Commands => self.cmd_filter.clone(),
            ViewMode::SavedList => self.saved_filter.clone(),
            _ => String::new(),
        };
        if !current.is_empty() {
            self.textarea.insert_str(&current);
        }
        self.completions.clear();
        self.completion_idx = None;
    }

    /// Begin `:` command.
    pub fn begin_command(&mut self) {
        self.input = InputMode::Command;
        self.textarea = LineEditor::default();
        self.completions.clear();
        self.completion_idx = None;
    }

    /// Overlay line (first textarea line).
    #[must_use]
    pub fn overlay_line(&self) -> String {
        self.textarea.lines().first().cloned().unwrap_or_default()
    }

    /// Apply overlay text as the live filter.
    pub fn sync_filter_from_overlay(&mut self) {
        let line = self.overlay_line();
        match self.view {
            ViewMode::Publications => self.pubs.set_filter(line),
            ViewMode::Commands => {
                self.cmd_filter = line;
                self.clamp_cmd();
            }
            ViewMode::SavedList => {
                self.saved_filter = line;
                self.clamp_saved();
            }
            _ => {}
        }
    }

    /// Close overlay; `clear` wipes the filter.
    pub fn end_overlay(&mut self, clear: bool) {
        if self.input == InputMode::Filter && clear {
            match self.view {
                ViewMode::Publications => self.pubs.set_filter(String::new()),
                ViewMode::Commands => {
                    self.cmd_filter.clear();
                    self.clamp_cmd();
                }
                ViewMode::SavedList => {
                    self.saved_filter.clear();
                    self.clamp_saved();
                }
                _ => {}
            }
        }
        self.input = InputMode::Normal;
        self.textarea = LineEditor::default();
        self.completions.clear();
        self.completion_idx = None;
    }

    /// Filtered command indexes.
    #[must_use]
    pub fn filtered_commands(&self) -> Vec<usize> {
        let q = self.cmd_filter.to_ascii_lowercase();
        SLASH_COMMANDS
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                q.is_empty()
                    || c.usage.to_ascii_lowercase().contains(&q)
                    || c.summary.to_ascii_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Selected slash command.
    #[must_use]
    pub fn selected_command(&self) -> Option<&'static crate::commands::SlashCommand> {
        let vis = self.cmd_state.selected().unwrap_or(0);
        let idx = *self.filtered_commands().get(vis)?;
        SLASH_COMMANDS.get(idx)
    }

    fn clamp_cmd(&mut self) {
        let n = self.filtered_commands().len();
        let sel = self.cmd_state.selected().unwrap_or(0);
        if n == 0 {
            self.cmd_state.select(Some(0));
        } else if sel >= n {
            self.cmd_state.select(Some(n - 1));
        }
    }

    /// Move commands selection.
    pub fn move_cmd(&mut self, delta: i32) {
        let n = i32::try_from(self.filtered_commands().len()).unwrap_or(0);
        if n == 0 {
            return;
        }
        let cur = i32::try_from(self.cmd_state.selected().unwrap_or(0)).unwrap_or(0);
        self.cmd_state.select(Some(
            usize::try_from((cur + delta).rem_euclid(n)).unwrap_or(0),
        ));
    }

    /// Filtered saved indexes.
    #[must_use]
    pub fn filtered_saved(&self) -> Vec<usize> {
        let q = self.saved_filter.to_ascii_lowercase();
        self.saved_rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                q.is_empty()
                    || r.id.to_ascii_lowercase().contains(&q)
                    || r.subject.to_ascii_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Selected saved row.
    #[must_use]
    pub fn selected_saved(&self) -> Option<&SavedRow> {
        let vis = self.saved_state.selected().unwrap_or(0);
        let idx = *self.filtered_saved().get(vis)?;
        self.saved_rows.get(idx)
    }

    fn clamp_saved(&mut self) {
        let n = self.filtered_saved().len();
        let sel = self.saved_state.selected().unwrap_or(0);
        if n == 0 {
            self.saved_state.select(Some(0));
        } else if sel >= n {
            self.saved_state.select(Some(n - 1));
        }
    }

    /// Move saved selection.
    pub fn move_saved(&mut self, delta: i32) {
        let n = i32::try_from(self.filtered_saved().len()).unwrap_or(0);
        if n == 0 {
            return;
        }
        let cur = i32::try_from(self.saved_state.selected().unwrap_or(0)).unwrap_or(0);
        self.saved_state.select(Some(
            usize::try_from((cur + delta).rem_euclid(n)).unwrap_or(0),
        ));
    }

    /// Apply `/list` reply.
    pub fn apply_saved_reply(&mut self, reply: Result<String, String>) {
        match reply {
            Ok(text) => {
                self.saved_rows = crate::saved_parse::parse_saved_rows(&text);
                self.saved_error.clear();
                self.saved_state.select(Some(0));
            }
            Err(e) => {
                self.saved_rows.clear();
                self.saved_error = e;
            }
        }
    }

    /// Text to yank with `y`.
    #[must_use]
    pub fn yank_text(&self) -> Option<String> {
        match self.view {
            ViewMode::Publications => self.pubs.selected_artefact().map(|a| a.id.clone()),
            ViewMode::SavedList => self.selected_saved().map(|r| r.id.clone()),
            ViewMode::Commands => self.selected_command().map(|c| c.usage.to_string()),
            _ => None,
        }
    }

    /// Whether the current view is filtered (or the `/` overlay is open).
    #[must_use]
    pub fn filter_active(&self) -> bool {
        let filled = match self.view {
            ViewMode::Publications => !self.pubs.filter.is_empty(),
            ViewMode::Commands => !self.cmd_filter.is_empty(),
            ViewMode::SavedList => !self.saved_filter.is_empty(),
            _ => false,
        };
        filled || self.input == InputMode::Filter
    }

    /// Filter match count `n/N` for the current table view.
    #[must_use]
    pub fn match_count(&self) -> Option<(usize, usize)> {
        match self.view {
            ViewMode::Publications => Some((
                self.pubs.filtered_indexes().len(),
                self.pubs.artefacts.len(),
            )),
            ViewMode::Commands => Some((self.filtered_commands().len(), SLASH_COMMANDS.len())),
            ViewMode::SavedList => Some((self.filtered_saved().len(), self.saved_rows.len())),
            _ => None,
        }
    }

    /// Whether the selected command is `/list`.
    #[must_use]
    pub fn selected_is_list_cmd(&self) -> bool {
        self.selected_command().is_some_and(|c| c.usage == "/list")
    }
}
