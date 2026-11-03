//! Navigation tree state and rendering.
//!
//! The tree has top-level rows (dashboard and categories) and, under a
//! category, either screens or sections. A section is an optional third
//! level that groups related screens (IP › Firewall › NAT). Hiding always
//! records screen and category ids, never section ids, so hidden menus stay
//! the same whether sections are shown or not.

use std::collections::{HashMap, HashSet};

use mtui_core::NavItem;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::layout::fit_line;
use crate::styles::Styles;

/// Result of toggling a sidebar row's hidden flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleHidden {
    Hidden,
    Restored,
    LastVisible,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct FlatNavEntry {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub is_group: bool,
    pub expanded: bool,
    pub hidden: bool,
    pub unavailable: bool,
    pub badge: Option<String>,
}

#[must_use]
pub fn flatten_nav(items: &[NavItem], expanded: Option<&str>) -> Vec<FlatNavEntry> {
    flatten_nav_filtered(items, expanded, &HashSet::new(), &HashMap::new(), false)
}

#[must_use]
pub fn flatten_nav_filtered(
    items: &[NavItem],
    expanded: Option<&str>,
    hidden: &HashSet<String>,
    unavailable: &HashMap<String, String>,
    show_hidden: bool,
) -> Vec<FlatNavEntry> {
    let view = NavView {
        expanded,
        expanded_section: None,
        hidden,
        unavailable,
        show_hidden,
    };
    view.flatten(items)
}

/// Inputs shared by every flatten step.
struct NavView<'a> {
    expanded: Option<&'a str>,
    expanded_section: Option<&'a str>,
    hidden: &'a HashSet<String>,
    unavailable: &'a HashMap<String, String>,
    show_hidden: bool,
}

impl NavView<'_> {
    fn flatten(&self, items: &[NavItem]) -> Vec<FlatNavEntry> {
        let mut out = Vec::new();
        for item in items {
            let group_hidden = self.hidden.contains(&item.id);
            if self.unavailable.contains_key(&item.id) {
                continue;
            }
            if user_concealed(group_hidden, self.show_hidden) {
                continue;
            }
            let is_group = !item.children.is_empty();
            if is_group && !group_hidden && self.visible_leaves(item) == 0 {
                continue;
            }
            let is_expanded = is_group && self.expanded == Some(item.id.as_str());
            out.push(FlatNavEntry {
                id: item.id.clone(),
                label: item.label.clone(),
                depth: 0,
                is_group,
                expanded: is_expanded,
                hidden: group_hidden,
                unavailable: false,
                badge: None,
            });
            if is_expanded {
                self.push_children(&mut out, item, group_hidden);
            }
        }
        out
    }

    fn push_children(&self, out: &mut Vec<FlatNavEntry>, group: &NavItem, group_hidden: bool) {
        for child in &group.children {
            if child.children.is_empty() {
                self.push_leaf(out, child, 1, group_hidden, &group.id);
                continue;
            }
            // Section: hidden when every screen in it is hidden.
            let available = leaves(child)
                .into_iter()
                .filter(|leaf| !self.unavailable.contains_key(&leaf.id))
                .collect::<Vec<_>>();
            let section_hidden =
                group_hidden || available.iter().all(|leaf| self.hidden.contains(&leaf.id));
            if available.is_empty() || user_concealed(section_hidden, self.show_hidden) {
                continue;
            }
            let is_expanded = self.expanded_section == Some(child.id.as_str());
            out.push(FlatNavEntry {
                id: child.id.clone(),
                label: child.label.clone(),
                depth: 1,
                is_group: true,
                expanded: is_expanded,
                hidden: section_hidden,
                unavailable: false,
                badge: None,
            });
            if is_expanded {
                for leaf in &child.children {
                    self.push_leaf(out, leaf, 2, group_hidden, &group.id);
                }
            }
        }
    }

    fn push_leaf(
        &self,
        out: &mut Vec<FlatNavEntry>,
        leaf: &NavItem,
        depth: usize,
        group_hidden: bool,
        group_id: &str,
    ) {
        let leaf_hidden = self.hidden.contains(&leaf.id) || group_hidden;
        if self.unavailable.contains_key(&leaf.id) {
            return;
        }
        if user_concealed(leaf_hidden, self.show_hidden) {
            return;
        }
        out.push(FlatNavEntry {
            id: leaf.id.clone(),
            label: leaf.label.clone(),
            depth,
            is_group: false,
            expanded: false,
            hidden: leaf_hidden,
            unavailable: false,
            badge: self
                .unavailable
                .get(&leaf.id)
                .cloned()
                .or_else(|| self.unavailable.get(group_id).cloned()),
        });
    }

    /// Screens under `item` that are neither unavailable nor (unless showing
    /// hidden rows) hidden.
    fn visible_leaves(&self, item: &NavItem) -> usize {
        leaves(item)
            .into_iter()
            .filter(|leaf| {
                !self.unavailable.contains_key(&leaf.id)
                    && !user_concealed(self.hidden.contains(&leaf.id), self.show_hidden)
            })
            .count()
    }
}

fn user_concealed(tucked: bool, show_hidden: bool) -> bool {
    tucked && !show_hidden
}

/// Every screen (leaf) under `item`, in tree order. A leaf returns itself.
fn leaves(item: &NavItem) -> Vec<&NavItem> {
    if item.children.is_empty() {
        return vec![item];
    }
    item.children.iter().flat_map(leaves).collect()
}

/// Path of ids from a top-level row down to `id`, inclusive.
fn path_to<'a>(items: &'a [NavItem], id: &str) -> Option<Vec<&'a NavItem>> {
    for item in items {
        if item.id == id {
            return Some(vec![item]);
        }
        if let Some(mut rest) = path_to(&item.children, id) {
            rest.insert(0, item);
            return Some(rest);
        }
    }
    None
}

fn find<'a>(items: &'a [NavItem], id: &str) -> Option<&'a NavItem> {
    path_to(items, id).and_then(|path| path.last().copied())
}

/// Sections are rows with children below the top level. They are not stored
/// in the hidden set; hiding one hides its screens.
fn is_section(items: &[NavItem], id: &str) -> bool {
    path_to(items, id)
        .is_some_and(|path| path.len() > 1 && !path[path.len() - 1].children.is_empty())
}

fn collapse_empty_groups(items: &[NavItem], hidden: &mut HashSet<String>) {
    for item in items {
        if item.children.is_empty() || hidden.contains(&item.id) {
            continue;
        }
        if leaves(item).iter().all(|leaf| hidden.contains(&leaf.id)) {
            hidden.insert(item.id.clone());
            clear_hidden_children(item, hidden);
        }
    }
}

/// Drop per-child hides once the category itself is hidden, so restoring the
/// parent brings every screen back in one step.
fn clear_hidden_children(item: &NavItem, hidden: &mut HashSet<String>) {
    for child in &item.children {
        hidden.remove(&child.id);
        clear_hidden_children(child, hidden);
    }
}

fn subsume_hidden_children(items: &[NavItem], hidden: &mut HashSet<String>, id: &str) {
    if let Some(item) = items.iter().find(|item| item.id == id) {
        clear_hidden_children(item, hidden);
    }
}

fn visible_leaf_count(
    items: &[NavItem],
    hidden: &HashSet<String>,
    unavailable: &HashMap<String, String>,
) -> usize {
    items
        .iter()
        .map(|item| {
            if hidden.contains(&item.id) || unavailable.contains_key(&item.id) {
                0
            } else {
                leaves(item)
                    .into_iter()
                    .filter(|leaf| {
                        !hidden.contains(&leaf.id) && !unavailable.contains_key(&leaf.id)
                    })
                    .count()
            }
        })
        .sum()
}

#[derive(Debug, Clone)]
pub struct NavState {
    pub tree: Vec<NavItem>,
    pub entries: Vec<FlatNavEntry>,
    pub selected: usize,
    pub expanded: Option<String>,
    /// Open section inside `expanded`, when the tree has sections.
    pub expanded_section: Option<String>,
    pub hidden: HashSet<String>,
    pub unavailable: HashMap<String, String>,
    pub show_hidden: bool,
    pub row_offset: usize,
    viewport_height: usize,
}

impl NavState {
    #[must_use]
    pub fn new(items: &[NavItem]) -> Self {
        let mut state = Self {
            tree: items.to_vec(),
            entries: Vec::new(),
            selected: 0,
            expanded: None,
            expanded_section: None,
            hidden: HashSet::new(),
            unavailable: HashMap::new(),
            show_hidden: false,
            row_offset: 0,
            viewport_height: 0,
        };
        state.rebuild();
        state
    }

    /// Swap the tree (for example between flat and sectioned layouts) and
    /// keep the cursor on the same screen when it still exists.
    pub fn set_tree(&mut self, items: &[NavItem]) {
        let selected = self.selected_id().map(str::to_owned);
        let cursor_on_section = selected
            .as_deref()
            .is_some_and(|id| is_section(&self.tree, id));
        self.tree = items.to_vec();
        self.expanded_section = None;
        if self
            .expanded
            .as_deref()
            .is_some_and(|id| find(&self.tree, id).is_none())
        {
            self.expanded = None;
        }
        self.rebuild();
        let Some(id) = selected else {
            return;
        };
        if cursor_on_section || find(&self.tree, &id).is_none() {
            if let Some(group) = self.expanded.clone() {
                let _ = self.focus_row(&group);
            }
            return;
        }
        let _ = self.select_id(&id);
    }

    fn focus_row(&mut self, id: &str) -> bool {
        match self.entries.iter().position(|entry| entry.id == id) {
            Some(idx) => {
                self.selected = idx;
                self.ensure_selection_visible();
                true
            }
            None => false,
        }
    }

    fn rebuild(&mut self) {
        let view = NavView {
            expanded: self.expanded.as_deref(),
            expanded_section: self.expanded_section.as_deref(),
            hidden: &self.hidden,
            unavailable: &self.unavailable,
            show_hidden: self.show_hidden,
        };
        self.entries = view.flatten(&self.tree);
        if self.entries.is_empty() {
            self.selected = 0;
            self.row_offset = 0;
            return;
        }
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len() - 1;
        }
        self.ensure_selection_visible();
    }

    /// Keep the focused row inside the inner pane height, reserving a hint
    /// row when the tree is taller than the pane.
    pub fn sync_viewport(&mut self, pane_height: usize) {
        self.viewport_height = pane_height;
        self.ensure_selection_visible();
    }

    fn list_height(&self) -> usize {
        nav_list_height(self.viewport_height, self.entries.len())
    }

    fn ensure_selection_visible(&mut self) {
        let visible = self.list_height();
        let total = self.entries.len();
        if visible == 0 || total == 0 {
            self.row_offset = 0;
            return;
        }
        let visible = visible.min(total);
        let max_off = total.saturating_sub(visible);
        if self.selected < self.row_offset {
            self.row_offset = self.selected;
        } else if self.selected >= self.row_offset.saturating_add(visible) {
            self.row_offset = self.selected.saturating_add(1).saturating_sub(visible);
        }
        self.row_offset = self.row_offset.min(max_off);
    }

    pub fn move_by(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        if delta == 0 {
            return;
        }
        let Some(len) = isize::try_from(self.entries.len()).ok() else {
            return;
        };
        let Some(selected) = isize::try_from(self.selected).ok() else {
            return;
        };
        let idx = selected
            .saturating_add(delta)
            .clamp(0, len.saturating_sub(1));
        if let Ok(index) = usize::try_from(idx) {
            self.selected = index;
        }
        self.ensure_selection_visible();
    }

    pub fn page_by(&mut self, direction: isize) {
        let page = isize::try_from(self.list_height().max(1)).unwrap_or(1);
        self.move_by(direction.saturating_mul(page));
    }

    pub fn select_first(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = 0;
        self.ensure_selection_visible();
    }

    pub fn select_last(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = self.entries.len() - 1;
        self.ensure_selection_visible();
    }

    #[must_use]
    pub fn selected_id(&self) -> Option<&str> {
        self.entries.get(self.selected).map(|e| e.id.as_str())
    }

    /// Replace the hidden-id set and rebuild visible rows.
    pub fn set_hidden_ids(&mut self, ids: impl IntoIterator<Item = String>) {
        self.hidden = ids.into_iter().filter(|id| !id.is_empty()).collect();
        self.rebuild();
    }

    /// Mark menus this device cannot offer (missing package or architecture).
    /// They stay out of the tree even when showing user-hidden rows.
    pub fn set_unavailable(&mut self, ids: HashMap<String, String>) {
        self.unavailable = ids.into_iter().filter(|(id, _)| !id.is_empty()).collect();
        self.rebuild();
    }

    pub fn set_show_hidden(&mut self, show: bool) {
        self.show_hidden = show;
        self.rebuild();
    }

    /// Reveal or tuck hidden rows. Returns the new `show_hidden` flag.
    pub fn toggle_show_hidden(&mut self) -> bool {
        self.show_hidden = !self.show_hidden;
        self.rebuild();
        self.show_hidden
    }

    /// Ids the hidden set records for `id`: the id itself, or every screen
    /// of a section (sections are layout only and never stored).
    fn hide_targets(&self, id: &str) -> Vec<String> {
        if is_section(&self.tree, id) {
            find(&self.tree, id).map_or_else(Vec::new, |section| {
                leaves(section)
                    .into_iter()
                    .map(|leaf| leaf.id.clone())
                    .collect()
            })
        } else {
            vec![id.to_string()]
        }
    }

    /// Whether `id` (screen, section or category) is currently hidden.
    fn is_hidden(&self, id: &str) -> bool {
        let targets = self.hide_targets(id);
        !targets.is_empty() && targets.iter().all(|target| self.hidden.contains(target))
    }

    /// Hidden set after hiding `id`, with the category folding rules applied.
    fn hidden_after_hiding(&self, id: &str) -> HashSet<String> {
        let mut next = self.hidden.clone();
        for target in self.hide_targets(id) {
            next.insert(target);
        }
        subsume_hidden_children(&self.tree, &mut next, id);
        collapse_empty_groups(&self.tree, &mut next);
        next
    }

    /// Hide or restore `id`. Restoring a child whose parent group is hidden
    /// restores the group instead. Hiding a category clears any previously
    /// hidden children so one restore brings the whole menu back. Hiding the
    /// last remaining child of a category also hides that category. After a
    /// hide, the cursor moves to the next remaining row, or the previous one
    /// if this was the last. A section hides or restores all its screens.
    pub fn toggle_hidden(&mut self, id: &str) -> ToggleHidden {
        if let Some(group) = self
            .top_level_of(id)
            .filter(|group| *group != id && self.hidden.contains(*group))
        {
            let group = group.to_string();
            self.hidden.remove(&group);
            self.rebuild();
            return ToggleHidden::Restored;
        }
        if self.is_hidden(id) {
            for target in self.hide_targets(id) {
                self.hidden.remove(&target);
            }
            self.rebuild();
            return ToggleHidden::Restored;
        }
        let hide_idx = self.entries.iter().position(|entry| entry.id == id);
        let following = hide_idx.map_or_else(Vec::new, |idx| {
            self.entries[idx.saturating_add(1)..]
                .iter()
                .map(|entry| entry.id.clone())
                .collect()
        });
        let preceding = hide_idx.map_or_else(Vec::new, |idx| {
            self.entries[..idx]
                .iter()
                .rev()
                .map(|entry| entry.id.clone())
                .collect()
        });
        let next = self.hidden_after_hiding(id);
        if visible_leaf_count(&self.tree, &next, &self.unavailable) == 0 {
            return ToggleHidden::LastVisible;
        }
        if self
            .expanded
            .as_deref()
            .is_some_and(|expanded| next.contains(expanded))
        {
            self.expanded = None;
            self.expanded_section = None;
        }
        self.hidden = next;
        if self
            .expanded_section
            .clone()
            .is_some_and(|section| self.is_hidden(&section))
        {
            self.expanded_section = None;
        }
        self.rebuild();
        self.select_remaining(&following, &preceding);
        ToggleHidden::Hidden
    }

    fn select_remaining(&mut self, following: &[String], preceding: &[String]) {
        for id in following.iter().chain(preceding) {
            if let Some(idx) = self.entries.iter().position(|entry| entry.id == *id) {
                self.selected = idx;
                return;
            }
        }
    }

    #[must_use]
    pub fn would_hide_last_leaf(&self, id: &str) -> bool {
        if self.is_hidden(id) {
            return false;
        }
        let next = self.hidden_after_hiding(id);
        visible_leaf_count(&self.tree, &next, &self.unavailable) == 0
    }

    /// Category that would also hide if `id` holds its last visible screens.
    #[must_use]
    pub fn hide_collapses_parent(&self, id: &str) -> Option<&str> {
        let group_id = self.top_level_of(id).filter(|group| *group != id)?;
        if self.hidden.contains(group_id) {
            return None;
        }
        let group = find(&self.tree, group_id)?;
        let targets = self.hide_targets(id);
        let remaining = leaves(group)
            .into_iter()
            .filter(|leaf| {
                !targets.contains(&leaf.id)
                    && !self.hidden.contains(&leaf.id)
                    && !self.unavailable.contains_key(&leaf.id)
            })
            .count();
        (remaining == 0).then_some(group_id)
    }

    #[must_use]
    pub fn label_of(&self, id: &str) -> Option<&str> {
        find(&self.tree, id).map(|item| item.label.as_str())
    }

    /// First leaf that remains after hiding, used as the post-connect landing.
    #[must_use]
    pub fn first_openable_id(&self) -> Option<String> {
        for item in &self.tree {
            if self.omitted_from_nav(&item.id) {
                continue;
            }
            if let Some(leaf) = leaves(item)
                .into_iter()
                .find(|leaf| !self.omitted_from_nav(&leaf.id))
            {
                return Some(leaf.id.clone());
            }
        }
        None
    }

    fn omitted_from_nav(&self, id: &str) -> bool {
        self.unavailable.contains_key(id) || (!self.show_hidden && self.hidden.contains(id))
    }

    fn top_level_of(&self, id: &str) -> Option<&str> {
        path_to(&self.tree, id).and_then(|path| path.first().map(|item| item.id.as_str()))
    }

    /// Select `id`, expanding its category (and collapsing others) so the
    /// matching row is visible. Group and section ids select their first
    /// visible screen.
    pub fn select_id(&mut self, id: &str) -> bool {
        let Some(path) = path_to(&self.tree, id) else {
            return false;
        };
        if path
            .iter()
            .any(|item| self.unavailable.contains_key(&item.id))
        {
            return false;
        }
        let target = path[path.len() - 1];
        let leaf = if target.children.is_empty() {
            target
        } else {
            let Some(leaf) = leaves(target).into_iter().find(|leaf| {
                !self.unavailable.contains_key(&leaf.id)
                    && (self.show_hidden || !self.hidden.contains(&leaf.id))
            }) else {
                return false;
            };
            leaf
        };
        let Some(leaf_path) = path_to(&self.tree, &leaf.id) else {
            return false;
        };
        let leaf_id = leaf.id.clone();
        let expanded = (leaf_path.len() > 1).then(|| leaf_path[0].id.clone());
        let expanded_section = (leaf_path.len() > 2).then(|| leaf_path[1].id.clone());
        self.expanded = expanded;
        self.expanded_section = expanded_section;
        self.rebuild();
        self.focus_row(&leaf_id)
    }

    /// Build styled lines for every visible row.
    ///
    /// `cursor` (`selected`) is the keyboard highlight; `viewed_id` is the
    /// resource currently shown in the content pane. Those can differ while
    /// browsing the tree before Enter. The open item is bright text; the
    /// focused cursor is a bounded selection bar (not a box border).
    /// Hidden rows (only drawn in show-hidden mode) use a `×` mark and
    /// strikethrough so they stay recoverable without relying on color.
    #[must_use]
    pub fn render_lines(
        &self,
        focused: bool,
        viewed_id: Option<&str>,
        styles: &Styles,
        width: usize,
    ) -> Vec<Line<'static>> {
        self.entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                nav_row_line(
                    entry,
                    idx == self.selected,
                    viewed_id == Some(entry.id.as_str()),
                    focused,
                    width,
                    styles,
                )
            })
            .collect()
    }

    /// Windowed sidebar for a pane of `height` rows, with bottom arrows when
    /// the tree does not fit. Call [`Self::sync_viewport`] with the same
    /// height so the stored offset stays aligned with this window.
    #[must_use]
    pub fn render_pane(
        &self,
        focused: bool,
        viewed_id: Option<&str>,
        styles: &Styles,
        width: usize,
        height: usize,
    ) -> Vec<Line<'static>> {
        if height == 0 {
            return Vec::new();
        }
        let total = self.entries.len();
        let list_h = nav_list_height(height, total);
        let start = nav_window_start(self.row_offset, self.selected, list_h, total);
        let end = start.saturating_add(list_h).min(total);
        let mut lines: Vec<Line<'static>> = self.entries[start..end]
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let abs = start.saturating_add(idx);
                nav_row_line(
                    entry,
                    abs == self.selected,
                    viewed_id == Some(entry.id.as_str()),
                    focused,
                    width,
                    styles,
                )
            })
            .collect();
        if nav_shows_scroll_hint(height, total) {
            lines.push(nav_scroll_hint(width, start > 0, end < total, styles));
        }
        lines
    }
}

/// Rows available for labels. When the tree is taller than the pane and at
/// least two rows exist, the last row is reserved for up/down arrows.
fn nav_list_height(pane_height: usize, total: usize) -> usize {
    if pane_height == 0 || total == 0 {
        return 0;
    }
    if nav_shows_scroll_hint(pane_height, total) {
        pane_height.saturating_sub(1)
    } else {
        total.min(pane_height)
    }
}

fn nav_shows_scroll_hint(pane_height: usize, total: usize) -> bool {
    total > pane_height && pane_height >= 2
}

fn nav_window_start(offset: usize, selected: usize, list_h: usize, total: usize) -> usize {
    if list_h == 0 || total == 0 {
        return 0;
    }
    let visible = list_h.min(total);
    let max_off = total.saturating_sub(visible);
    let mut off = offset.min(max_off);
    if selected < off {
        off = selected;
    } else if selected >= off.saturating_add(visible) {
        off = selected.saturating_add(1).saturating_sub(visible);
    }
    off.min(max_off)
}

fn nav_scroll_hint(width: usize, can_up: bool, can_down: bool, styles: &Styles) -> Line<'static> {
    let arrow = |available: bool| {
        if available {
            styles.text
        } else {
            styles.muted.add_modifier(Modifier::DIM)
        }
    };
    let content_w = 4;
    let pad = width.saturating_sub(content_w) / 2;
    fit_line(
        Line::from(vec![
            Span::raw(" ".repeat(pad)),
            Span::styled("▲", arrow(can_up)),
            Span::raw("  "),
            Span::styled("▼", arrow(can_down)),
        ]),
        width.max(1),
    )
}

fn nav_row_line(
    entry: &FlatNavEntry,
    is_cursor: bool,
    is_viewed: bool,
    pane_focused: bool,
    width: usize,
    styles: &Styles,
) -> Line<'static> {
    let chevron = if entry.is_group {
        if entry.expanded { "▾ " } else { "▸ " }
    } else {
        ""
    };
    let indent = "  ".repeat(entry.depth);
    let mark = if entry.hidden { "× " } else { "" };
    let badge = entry
        .badge
        .as_deref()
        .map(|package| format!(" !{package}"))
        .unwrap_or_default();
    let body = format!("{indent}{chevron}{mark}{}{badge}", entry.label);
    let body_style = if entry.hidden {
        let base = if is_viewed {
            styles.text
        } else {
            styles.hidden
        };
        base.add_modifier(Modifier::CROSSED_OUT)
    } else if entry.unavailable {
        styles.hidden
    } else if is_viewed {
        styles.text
    } else if entry.depth > 0 {
        styles.quiet
    } else {
        styles.muted
    };
    let line = Line::from(vec![Span::styled(body, body_style)]);
    if is_cursor && pane_focused {
        crate::layout::fit_line(
            crate::paint::line_on_bg(line, styles.selection),
            width.max(1),
        )
    } else {
        line
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    fn tree() -> Vec<NavItem> {
        vec![
            NavItem {
                id: "dashboard".into(),
                label: "Dashboard".into(),
                children: vec![],
            },
            NavItem {
                id: "bridge-group".into(),
                label: "Bridge".into(),
                children: vec![
                    NavItem {
                        id: "bridges".into(),
                        label: "Bridge".into(),
                        children: vec![],
                    },
                    NavItem {
                        id: "bridge-vlans".into(),
                        label: "VLANs".into(),
                        children: vec![],
                    },
                ],
            },
            NavItem {
                id: "ip-group".into(),
                label: "IP".into(),
                children: vec![NavItem {
                    id: "arp".into(),
                    label: "ARP".into(),
                    children: vec![],
                }],
            },
        ]
    }

    fn visible_ids(state: &NavState) -> Vec<&str> {
        state
            .entries
            .iter()
            .map(|entry| entry.id.as_str())
            .collect()
    }

    fn styles() -> Styles {
        use mtui_core::{DefaultTheme, Theme};
        Styles::from_palette(DefaultTheme::new().palette())
    }

    fn line_text(line: &Line<'_>) -> String {
        crate::layout::line_plain(line)
    }

    #[test]
    fn starts_collapsed_with_groups_selectable() {
        let state = NavState::new(&tree());
        assert_eq!(
            visible_ids(&state),
            ["dashboard", "bridge-group", "ip-group"]
        );
        assert_eq!(state.selected_id(), Some("dashboard"));
        assert!(state.expanded.is_none());
    }

    #[test]
    fn move_down_selects_group_headers() {
        let mut state = NavState::new(&tree());
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("bridge-group"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("ip-group"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("ip-group"));
    }

    #[test]
    fn entering_a_group_expands_it_and_selects_the_first_child() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-group"));
        assert_eq!(state.expanded.as_deref(), Some("bridge-group"));
        assert_eq!(state.selected_id(), Some("bridges"));
        assert_eq!(
            visible_ids(&state),
            [
                "dashboard",
                "bridge-group",
                "bridges",
                "bridge-vlans",
                "ip-group"
            ]
        );
    }

    #[test]
    fn expanding_another_group_collapses_the_previous() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-group"));
        assert!(state.select_id("ip-group"));
        assert_eq!(state.expanded.as_deref(), Some("ip-group"));
        assert_eq!(state.selected_id(), Some("arp"));
        assert_eq!(
            visible_ids(&state),
            ["dashboard", "bridge-group", "ip-group", "arp"]
        );
        assert!(!visible_ids(&state).contains(&"bridges"));
    }

    #[test]
    fn move_between_expanded_children_and_next_group() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridges"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("bridge-vlans"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("ip-group"));
        state.move_by(-1);
        assert_eq!(state.selected_id(), Some("bridge-vlans"));
    }

    #[test]
    fn move_up_from_first_item_stays_put() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("dashboard"));
        state.move_by(-1);
        assert_eq!(state.selected_id(), Some("dashboard"));
    }

    #[test]
    fn move_down_from_last_item_stays_put() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("arp"));
    }

    #[test]
    fn cursor_on_unopened_row_uses_focus_marker() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        state.move_by(1);
        let lines = state.render_lines(true, Some("dashboard"), &styles, 24);
        let selected = state.selected;
        assert_eq!(state.selected_id(), Some("bridge-group"));
        assert!(
            lines[selected]
                .spans
                .iter()
                .all(|span| span.style.bg == Some(styles.selection)),
            "cursor row must be a bounded fill: {:?}",
            lines[selected]
        );
        assert!(line_text(&lines[selected]).contains("▸ "));
        assert!(!line_text(&lines[selected]).contains('›'));
        assert!(line_text(&lines[0]).contains("Dashboard"));
        assert!(
            lines[0].spans.iter().all(|span| span.style.bg.is_none()),
            "unfocused nav rows stay without fill"
        );
        assert_eq!(
            lines[0].spans.last().map(|span| span.style.fg),
            Some(styles.text.fg)
        );
    }

    #[test]
    fn viewed_and_focused_row_uses_cursor_and_text() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        let lines = state.render_lines(true, Some("arp"), &styles, 24);
        let selected = state.selected;
        assert!(
            lines[selected]
                .spans
                .iter()
                .all(|span| span.style.bg == Some(styles.selection))
        );
        assert_eq!(
            lines[selected]
                .spans
                .iter()
                .find(|span| !span.content.chars().all(|ch| ch == ' '))
                .and_then(|span| span.style.fg),
            styles.text.fg
        );
        assert!(!line_text(&lines[selected]).contains('›'));
    }

    #[test]
    fn viewed_row_drops_cursor_when_nav_unfocused() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        let lines = state.render_lines(false, Some("arp"), &styles, 24);
        let selected = state.selected;
        assert!(
            lines[selected]
                .spans
                .iter()
                .all(|span| span.style.bg.is_none())
        );
        assert!(!line_text(&lines[selected]).contains('›'));
        assert_eq!(
            lines[selected].spans.last().map(|span| span.style.fg),
            Some(styles.text.fg)
        );
    }

    #[test]
    fn group_rows_use_muted_style_when_not_selected() {
        let styles = styles();
        let state = NavState::new(&tree());
        let lines = state.render_lines(true, Some("dashboard"), &styles, 24);
        assert_eq!(
            lines[1].spans.last().map(|span| span.style),
            Some(styles.muted)
        );
        assert!(line_text(&lines[1]).contains("▸ "));
        assert!(!line_text(&lines[1]).contains("▾ "));
    }

    #[test]
    fn expanded_group_uses_open_chevron() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-group"));
        let lines = state.render_lines(false, Some("bridges"), &styles, 24);
        assert!(line_text(&lines[1]).contains("▾ "));
    }

    #[test]
    fn nested_rows_are_quieter_than_top_level() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridges"));
        let lines = state.render_lines(false, Some("bridges"), &styles, 24);
        assert_eq!(
            lines[1].spans.last().map(|span| span.style.fg),
            Some(styles.muted.fg),
            "top-level group should stay muted"
        );
        assert_eq!(
            lines[3].spans.last().map(|span| span.style.fg),
            Some(styles.quiet.fg),
            "unfocused child should recede: {}",
            line_text(&lines[3])
        );
        assert_eq!(
            lines[2].spans.last().map(|span| span.style.fg),
            Some(styles.text.fg)
        );
    }

    #[test]
    fn select_id_reveals_nested_resource() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        assert_eq!(state.selected_id(), Some("arp"));
        assert_eq!(state.expanded.as_deref(), Some("ip-group"));
        assert!(!state.select_id("missing"));
        assert_eq!(state.selected_id(), Some("arp"));
    }

    #[test]
    fn selecting_dashboard_collapses_groups() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-vlans"));
        assert!(state.select_id("dashboard"));
        assert!(state.expanded.is_none());
        assert_eq!(
            visible_ids(&state),
            ["dashboard", "bridge-group", "ip-group"]
        );
    }

    #[test]
    fn hiding_a_group_removes_it_and_its_children() {
        let mut state = NavState::new(&tree());
        assert_eq!(state.toggle_hidden("bridge-group"), ToggleHidden::Hidden);
        assert_eq!(visible_ids(&state), ["dashboard", "ip-group"]);
        assert!(!state.select_id("bridges"));
    }

    #[test]
    fn hiding_a_group_clears_previously_hidden_children() {
        let mut state = NavState::new(&tree());
        assert_eq!(state.toggle_hidden("bridges"), ToggleHidden::Hidden);
        assert!(state.hidden.contains("bridges"));
        assert_eq!(state.toggle_hidden("bridge-group"), ToggleHidden::Hidden);
        assert!(state.hidden.contains("bridge-group"));
        assert!(
            !state.hidden.contains("bridges"),
            "parent hide should drop child hides: {:?}",
            state.hidden
        );
        assert_eq!(state.toggle_hidden("bridge-group"), ToggleHidden::Restored);
        assert!(state.hidden.is_empty());
        assert!(state.select_id("bridges"));
        assert_eq!(state.selected_id(), Some("bridges"));
    }

    #[test]
    fn hiding_a_child_omits_it_and_skips_it_when_opening_the_group() {
        let mut state = NavState::new(&tree());
        assert_eq!(state.toggle_hidden("bridges"), ToggleHidden::Hidden);
        assert!(state.select_id("bridge-group"));
        assert_eq!(state.selected_id(), Some("bridge-vlans"));
        assert_eq!(
            visible_ids(&state),
            ["dashboard", "bridge-group", "bridge-vlans", "ip-group"]
        );
    }

    #[test]
    fn hiding_every_child_tucks_the_empty_group() {
        let mut state = NavState::new(&tree());
        assert_eq!(state.toggle_hidden("arp"), ToggleHidden::Hidden);
        assert!(state.hidden.contains("ip-group"));
        assert!(!state.hidden.contains("arp"));
        assert_eq!(visible_ids(&state), ["dashboard", "bridge-group"]);
        assert_eq!(state.first_openable_id().as_deref(), Some("dashboard"));
    }

    #[test]
    fn hiding_a_row_selects_the_next_remaining_row() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridges"));
        assert_eq!(state.toggle_hidden("bridges"), ToggleHidden::Hidden);
        assert_eq!(state.selected_id(), Some("bridge-vlans"));
        assert_eq!(state.toggle_hidden("bridge-vlans"), ToggleHidden::Hidden);
        assert!(state.hidden.contains("bridge-group"));
        assert!(!state.hidden.contains("bridge-vlans"));
        assert_eq!(state.selected_id(), Some("ip-group"));
    }

    #[test]
    fn hiding_the_last_row_selects_the_previous_row() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        assert_eq!(state.toggle_hidden("arp"), ToggleHidden::Hidden);
        assert_eq!(state.selected_id(), Some("bridge-group"));
    }

    #[test]
    fn show_hidden_keeps_tucked_rows_marked() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert_eq!(state.toggle_hidden("ip-group"), ToggleHidden::Hidden);
        assert!(state.toggle_show_hidden());
        assert!(visible_ids(&state).contains(&"ip-group"));
        let lines = state.render_lines(false, Some("dashboard"), &styles, 24);
        let ip = state
            .entries
            .iter()
            .position(|entry| entry.id == "ip-group")
            .expect("ip group visible in show-hidden");
        let text = line_text(&lines[ip]);
        assert!(text.contains('×'), "hidden mark: {text}");
        assert!(text.contains("IP"));
        assert!(
            lines[ip]
                .spans
                .iter()
                .any(|span| span.style.add_modifier.contains(Modifier::CROSSED_OUT)),
            "strikethrough must mark hidden rows: {:?}",
            lines[ip]
        );
        assert_eq!(state.toggle_hidden("ip-group"), ToggleHidden::Restored);
        state.set_show_hidden(false);
        assert!(visible_ids(&state).contains(&"ip-group"));
        let restored = state.render_lines(false, Some("dashboard"), &styles, 24);
        assert!(!line_text(&restored[ip]).contains('×'));
    }

    #[test]
    fn restoring_a_child_of_a_hidden_group_restores_the_group() {
        let mut state = NavState::new(&tree());
        assert_eq!(state.toggle_hidden("bridge-group"), ToggleHidden::Hidden);
        state.set_show_hidden(true);
        assert!(state.select_id("bridges"));
        assert_eq!(state.toggle_hidden("bridges"), ToggleHidden::Restored);
        state.set_show_hidden(false);
        assert!(visible_ids(&state).contains(&"bridge-group"));
        assert!(!state.hidden.contains("bridge-group"));
    }

    #[test]
    fn refuses_to_hide_the_last_visible_leaf() {
        let mut state = NavState::new(&tree());
        assert_eq!(state.toggle_hidden("bridge-group"), ToggleHidden::Hidden);
        assert_eq!(state.toggle_hidden("ip-group"), ToggleHidden::Hidden);
        assert_eq!(state.toggle_hidden("dashboard"), ToggleHidden::LastVisible);
        assert_eq!(visible_ids(&state), ["dashboard"]);
        assert!(!state.hidden.contains("dashboard"));
    }

    #[test]
    fn unavailable_menus_stay_out_when_showing_hidden() {
        let mut state = NavState::new(&tree());
        let mut missing = HashMap::new();
        missing.insert("arp".into(), "hotspot".into());
        state.set_unavailable(missing);
        assert!(!visible_ids(&state).contains(&"arp"));
        assert!(!visible_ids(&state).contains(&"ip-group"));
        state.set_show_hidden(true);
        assert!(!visible_ids(&state).contains(&"arp"));
        assert!(!state.select_id("arp"));
        let styles = styles();
        let lines = state.render_lines(false, None, &styles, 32);
        assert!(
            lines
                .iter()
                .map(line_text)
                .all(|line| !line.contains("ARP")),
        );
    }

    #[test]
    fn last_visible_ignores_unavailable_leaves() {
        let mut state = NavState::new(&tree());
        let mut missing = HashMap::new();
        missing.insert("dashboard".into(), "package".into());
        missing.insert("bridges".into(), "package".into());
        missing.insert("bridge-vlans".into(), "package".into());
        state.set_unavailable(missing);
        assert_eq!(state.toggle_hidden("ip-group"), ToggleHidden::LastVisible);
        assert!(!state.hidden.contains("ip-group"));
    }

    fn tall_tree() -> Vec<NavItem> {
        (0..12)
            .map(|i| NavItem {
                id: format!("item-{i}"),
                label: format!("Item {i}"),
                children: vec![],
            })
            .collect()
    }

    #[test]
    fn pane_scroll_keeps_focus_in_window() {
        let mut state = NavState::new(&tall_tree());
        state.sync_viewport(5);
        state.select_last();
        let styles = styles();
        let lines = state.render_pane(true, None, &styles, 24, 5);
        assert_eq!(lines.len(), 5);
        assert!(line_text(&lines[3]).contains("Item 11"));
        assert_eq!(state.selected, 11);
        assert_eq!(state.row_offset, 8);
    }

    #[test]
    fn pane_scroll_offset_is_sticky_inside_the_window() {
        let mut state = NavState::new(&tall_tree());
        state.sync_viewport(5);
        state.selected = 6;
        state.ensure_selection_visible();
        assert_eq!(state.row_offset, 3);
        state.move_by(-1);
        assert_eq!(state.selected, 5);
        assert_eq!(state.row_offset, 3);
    }

    #[test]
    fn short_menu_has_no_scroll_hint() {
        let mut state = NavState::new(&tree());
        state.sync_viewport(12);
        let styles = styles();
        let lines = state.render_pane(true, None, &styles, 24, 12);
        assert!(lines.iter().all(|line| !line_text(line).contains('▲')));
        assert!(lines.iter().all(|line| !line_text(line).contains('▼')));
    }

    #[test]
    fn overflow_hint_uses_text_when_that_direction_can_scroll() {
        let mut state = NavState::new(&tall_tree());
        state.sync_viewport(5);
        let styles = styles();
        let top = state.render_pane(true, None, &styles, 24, 5);
        let hint = top.last().expect("hint row");
        assert!(line_text(hint).contains('▲'));
        assert!(line_text(hint).contains('▼'));
        let up = hint
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "▲")
            .expect("up arrow");
        let down = hint
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "▼")
            .expect("down arrow");
        assert_eq!(up.style.fg, styles.muted.fg);
        assert!(up.style.add_modifier.contains(Modifier::DIM));
        assert_eq!(down.style.fg, styles.text.fg);
        assert!(!down.style.add_modifier.contains(Modifier::DIM));

        state.select_last();
        let bottom = state.render_pane(true, None, &styles, 24, 5);
        let hint = bottom.last().expect("hint row");
        let up = hint
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "▲")
            .expect("up arrow");
        let down = hint
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "▼")
            .expect("down arrow");
        assert_eq!(up.style.fg, styles.text.fg);
        assert_eq!(down.style.fg, styles.muted.fg);
        assert!(down.style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn one_row_pane_shows_the_focused_item() {
        let mut state = NavState::new(&tall_tree());
        state.sync_viewport(1);
        state.select_last();
        let styles = styles();
        let lines = state.render_pane(true, None, &styles, 24, 1);
        assert_eq!(lines.len(), 1);
        assert!(line_text(&lines[0]).contains("Item 11"));
        assert!(!line_text(&lines[0]).contains('▲'));
    }
}

#[cfg(test)]
mod section_tests {
    use super::*;

    fn leaf(id: &str, label: &str) -> NavItem {
        NavItem {
            id: id.into(),
            label: label.into(),
            children: vec![],
        }
    }

    fn sectioned() -> Vec<NavItem> {
        vec![
            leaf("dashboard", "Dashboard"),
            NavItem {
                id: "ip-group".into(),
                label: "IP".into(),
                children: vec![
                    leaf("addresses", "Addresses"),
                    NavItem {
                        id: "ip-group/firewall".into(),
                        label: "Firewall".into(),
                        children: vec![
                            leaf("firewall-filter", "Filter Rules"),
                            leaf("firewall-nat", "NAT"),
                        ],
                    },
                    leaf("routes", "Routes"),
                ],
            },
        ]
    }

    fn flat() -> Vec<NavItem> {
        vec![
            leaf("dashboard", "Dashboard"),
            NavItem {
                id: "ip-group".into(),
                label: "IP".into(),
                children: vec![
                    leaf("addresses", "Addresses"),
                    leaf("firewall-filter", "Firewall › Filter Rules"),
                    leaf("firewall-nat", "Firewall › NAT"),
                    leaf("routes", "Routes"),
                ],
            },
        ]
    }

    fn ids(state: &NavState) -> Vec<&str> {
        state.entries.iter().map(|e| e.id.as_str()).collect()
    }

    #[test]
    fn opening_a_category_shows_sections_collapsed() {
        let mut state = NavState::new(&sectioned());
        assert!(state.select_id("ip-group"));
        assert_eq!(state.selected_id(), Some("addresses"));
        assert_eq!(
            ids(&state),
            [
                "dashboard",
                "ip-group",
                "addresses",
                "ip-group/firewall",
                "routes"
            ]
        );
        let section = &state.entries[3];
        assert!(section.is_group);
        assert_eq!(section.depth, 1);
        assert!(!section.expanded);
    }

    #[test]
    fn entering_a_section_expands_it_and_selects_its_first_screen() {
        let mut state = NavState::new(&sectioned());
        assert!(state.select_id("ip-group/firewall"));
        assert_eq!(state.selected_id(), Some("firewall-filter"));
        assert_eq!(state.expanded.as_deref(), Some("ip-group"));
        assert_eq!(state.expanded_section.as_deref(), Some("ip-group/firewall"));
        let nat = state
            .entries
            .iter()
            .find(|e| e.id == "firewall-nat")
            .expect("nat row");
        assert_eq!(nat.depth, 2);
    }

    #[test]
    fn select_id_reveals_a_screen_inside_a_section() {
        let mut state = NavState::new(&sectioned());
        assert!(state.select_id("firewall-nat"));
        assert_eq!(state.selected_id(), Some("firewall-nat"));
        assert_eq!(state.expanded_section.as_deref(), Some("ip-group/firewall"));
        assert!(state.select_id("routes"));
        assert!(state.expanded_section.is_none());
        assert!(state.entries.iter().all(|e| e.id != "firewall-nat"));
    }

    #[test]
    fn hiding_a_section_stores_only_its_screens() {
        let mut state = NavState::new(&sectioned());
        state.select_id("ip-group");
        assert_eq!(
            state.toggle_hidden("ip-group/firewall"),
            ToggleHidden::Hidden
        );
        let mut hidden: Vec<&str> = state.hidden.iter().map(String::as_str).collect();
        hidden.sort_unstable();
        assert_eq!(hidden, ["firewall-filter", "firewall-nat"]);
        assert!(state.entries.iter().all(|e| e.id != "ip-group/firewall"));

        state.set_show_hidden(true);
        let row = state
            .entries
            .iter()
            .find(|e| e.id == "ip-group/firewall")
            .expect("hidden section listed");
        assert!(row.hidden);
        assert_eq!(
            state.toggle_hidden("ip-group/firewall"),
            ToggleHidden::Restored
        );
        assert!(state.hidden.is_empty());
    }

    #[test]
    fn hiding_every_screen_through_sections_tucks_the_category() {
        let mut state = NavState::new(&sectioned());
        state.select_id("ip-group");
        state.toggle_hidden("addresses");
        state.toggle_hidden("routes");
        assert_eq!(
            state.hide_collapses_parent("ip-group/firewall"),
            Some("ip-group")
        );
        // Dashboard still remains, so this is not the last screen.
        assert!(!state.would_hide_last_leaf("ip-group/firewall"));
        assert_eq!(
            state.toggle_hidden("ip-group/firewall"),
            ToggleHidden::Hidden
        );
        assert!(state.hidden.contains("ip-group"));
        assert!(state.entries.iter().all(|e| e.id != "ip-group"));
    }

    #[test]
    fn switching_layouts_keeps_the_cursor_and_hidden_screens() {
        let mut state = NavState::new(&flat());
        state.select_id("firewall-nat");
        state.toggle_hidden("routes");

        state.set_tree(&sectioned());
        assert_eq!(state.selected_id(), Some("firewall-nat"));
        assert_eq!(state.expanded_section.as_deref(), Some("ip-group/firewall"));
        assert!(state.hidden.contains("routes"));
        assert!(state.entries.iter().all(|e| e.id != "routes"));

        state.set_tree(&flat());
        assert_eq!(state.selected_id(), Some("firewall-nat"));
        assert!(state.expanded_section.is_none());
    }

    #[test]
    fn switching_from_a_section_row_lands_on_its_category() {
        let mut state = NavState::new(&sectioned());
        state.select_id("ip-group");
        let idx = state
            .entries
            .iter()
            .position(|e| e.id == "ip-group/firewall")
            .expect("section");
        state.selected = idx;
        state.set_tree(&flat());
        assert_eq!(state.selected_id(), Some("ip-group"));
    }

    #[test]
    fn section_rows_indent_their_chevron_and_screens() {
        use mtui_core::{DefaultTheme, Theme};
        let styles = Styles::from_palette(DefaultTheme::new().palette());
        let mut state = NavState::new(&sectioned());
        state.select_id("firewall-nat");
        let lines: Vec<String> = state
            .render_lines(false, None, &styles, 30)
            .iter()
            .map(crate::layout::line_plain)
            .collect();
        assert!(
            lines.iter().any(|l| l.starts_with("  ▾ Firewall")),
            "{lines:?}"
        );
        assert!(lines.iter().any(|l| l.starts_with("    NAT")), "{lines:?}");
    }
}
