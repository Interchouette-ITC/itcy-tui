// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! One-line overlay editor for `/` filter and `:` commands.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Single-line buffer with a character cursor.
#[derive(Debug, Clone, Default)]
pub struct LineEditor {
    buf: String,
    /// Character index (not byte).
    cursor: usize,
}

impl LineEditor {
    /// First (only) line, matching a one-line textarea API.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        vec![self.buf.clone()]
    }

    /// Cursor column for `Frame::set_cursor_position` (chars, ASCII commands).
    #[must_use]
    pub fn cursor_col(&self) -> u16 {
        u16::try_from(self.cursor).unwrap_or(u16::MAX)
    }

    /// Insert at the cursor.
    pub fn insert_str(&mut self, s: &str) {
        let mut chars: Vec<char> = self.buf.chars().collect();
        let at = self.cursor.min(chars.len());
        chars.splice(at..at, s.chars());
        self.cursor = at.saturating_add(s.chars().count());
        self.buf = chars.into_iter().collect();
    }

    /// Replace the whole line and put the cursor at the end.
    pub fn set_line(&mut self, line: &str) {
        self.buf = line.to_string();
        self.cursor = self.buf.chars().count();
    }

    /// Apply a key (Esc / Enter / Tab are handled by the loop).
    pub fn input(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char(c) if !ctrl => self.insert_str(&c.to_string()),
            KeyCode::Char('u') if ctrl => {
                self.buf.clear();
                self.cursor = 0;
            }
            KeyCode::Char('w') if ctrl => self.delete_word(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete_forward(),
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => {
                let n = self.buf.chars().count();
                if self.cursor < n {
                    self.cursor = self.cursor.saturating_add(1);
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.buf.chars().count(),
            _ => {}
        }
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut chars: Vec<char> = self.buf.chars().collect();
        let at = self.cursor.saturating_sub(1);
        chars.remove(at);
        self.cursor = at;
        self.buf = chars.into_iter().collect();
    }

    fn delete_forward(&mut self) {
        let mut chars: Vec<char> = self.buf.chars().collect();
        if self.cursor >= chars.len() {
            return;
        }
        chars.remove(self.cursor);
        self.buf = chars.into_iter().collect();
    }

    fn delete_word(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let chars: Vec<char> = self.buf.chars().collect();
        let mut i = self.cursor;
        while i > 0 && chars[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        let mut kept = chars;
        kept.drain(i..self.cursor);
        self.cursor = i;
        self.buf = kept.into_iter().collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    #[test]
    fn insert_and_backspace() {
        let mut ed = LineEditor::default();
        ed.insert_str("DRAFT");
        ed.input(key(KeyCode::Backspace));
        assert_eq!(ed.lines()[0], "DRAF");
        assert_eq!(ed.cursor_col(), 4);
    }

    #[test]
    fn set_line_puts_cursor_at_end() {
        let mut ed = LineEditor::default();
        ed.set_line("pubs");
        assert_eq!(ed.lines()[0], "pubs");
        assert_eq!(ed.cursor_col(), 4);
    }
}
