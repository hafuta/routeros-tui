//! Tab strip above a screen that belongs to a sidebar section (tab layout).

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::layout::fit_line;
use crate::styles::Styles;

const GAP: &str = "   ";
const MORE_LEFT: &str = "‹ ";
const MORE_RIGHT: &str = " ›";

/// One line of tabs. The active tab is bracketed and bold (not color-only);
/// when the strip is wider than `width`, it scrolls so the active tab stays
/// visible and `‹` / `›` mark the clipped side.
#[must_use]
pub fn content_tab_line(
    tabs: &[(String, String)],
    active: &str,
    width: usize,
    styles: &Styles,
) -> Line<'static> {
    if tabs.is_empty() || width == 0 {
        return Line::default();
    }
    let cells: Vec<(String, bool)> = tabs
        .iter()
        .map(|(id, label)| {
            let on = id == active;
            let text = if on {
                format!("[{label}]")
            } else {
                format!(" {label} ")
            };
            (text, on)
        })
        .collect();
    let active_idx = cells.iter().position(|(_, on)| *on).unwrap_or(0);
    let span_width = |range: &[(String, bool)]| -> usize {
        let cells: usize = range.iter().map(|(text, _)| text.width()).sum();
        cells + GAP.width() * range.len().saturating_sub(1)
    };

    // Widest window [start, end) that contains the active tab.
    let mut start = 0;
    let mut end = cells.len();
    let budget = |start: usize, end: usize| {
        let mut w = span_width(&cells[start..end]);
        if start > 0 {
            w += MORE_LEFT.width();
        }
        if end < cells.len() {
            w += MORE_RIGHT.width();
        }
        w
    };
    while budget(start, end) > width && end - start > 1 {
        if end - 1 > active_idx {
            end -= 1;
        } else {
            start += 1;
        }
    }

    let mut spans = Vec::new();
    if start > 0 {
        spans.push(Span::styled(MORE_LEFT, styles.muted));
    }
    for (i, (text, on)) in cells[start..end].iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(GAP, styles.muted));
        }
        let style = if *on {
            styles.text.add_modifier(Modifier::BOLD)
        } else {
            styles.muted
        };
        spans.push(Span::styled(text.clone(), style));
    }
    if end < cells.len() {
        spans.push(Span::styled(MORE_RIGHT, styles.muted));
    }
    fit_line(Line::from(spans), width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::line_plain;
    use mtui_core::{DefaultTheme, Theme};

    fn styles() -> Styles {
        Styles::from_palette(DefaultTheme::new().palette())
    }

    fn tabs() -> Vec<(String, String)> {
        ["Filter Rules", "NAT", "Mangle", "Raw", "Connections"]
            .iter()
            .enumerate()
            .map(|(i, label)| (format!("t{i}"), (*label).to_string()))
            .collect()
    }

    #[test]
    fn active_tab_is_bracketed() {
        let line = line_plain(&content_tab_line(&tabs(), "t1", 80, &styles()));
        assert!(line.contains("[NAT]"), "{line}");
        assert!(line.contains(" Filter Rules "), "{line}");
        assert!(!line.contains('‹') && !line.contains('›'), "{line}");
    }

    #[test]
    fn narrow_strip_keeps_the_active_tab_visible() {
        let line = line_plain(&content_tab_line(&tabs(), "t4", 24, &styles()));
        assert!(line.contains("[Connections]"), "{line}");
        assert!(line.starts_with('‹'), "{line}");
        assert!(line.width() <= 24, "{line}");

        let line = line_plain(&content_tab_line(&tabs(), "t0", 24, &styles()));
        assert!(line.contains("[Filter Rules]"), "{line}");
        assert!(line.trim_end().ends_with('›'), "{line}");
    }

    #[test]
    fn empty_tabs_render_nothing() {
        assert!(content_tab_line(&[], "x", 40, &styles()).spans.is_empty());
    }
}
