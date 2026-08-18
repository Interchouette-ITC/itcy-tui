// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Mouse hit rectangles from the last draw.

use ratatui::layout::Rect;

/// Click targets recorded while drawing.
#[derive(Debug, Clone, Default)]
pub struct HitMap {
    /// `d:live` tab.
    pub live_tab: Rect,
    /// `c:commands` tab.
    pub commands_tab: Rect,
    /// `p:pubs` tab.
    pub pubs_tab: Rect,
    /// `s:list` tab.
    pub list_tab: Rect,
    /// Org chip.
    pub org_chip: Rect,
    /// Fork chip.
    pub fork_chip: Rect,
    /// Branch chips in `PubsBranch::ALL` order.
    pub branch_chips: [Rect; 4],
    /// Publications list table.
    pub pubs_list: Rect,
    /// Publications preview.
    pub pubs_preview: Rect,
    /// Commands table.
    pub commands_table: Rect,
    /// Saved-list table.
    pub saved_table: Rect,
    /// Live dashboard body.
    pub live_body: Rect,
    /// Help body.
    pub help_body: Rect,
}

/// Whether `col,row` is inside `r` (including empty-rect false).
#[must_use]
pub const fn contains(r: Rect, col: u16, row: u16) -> bool {
    r.width > 0
        && r.height > 0
        && col >= r.x
        && col < r.x.saturating_add(r.width)
        && row >= r.y
        && row < r.y.saturating_add(r.height)
}
