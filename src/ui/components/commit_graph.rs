use crate::models::Commit;
use crate::state::AppState;
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke, Vec2};
use std::collections::{HashMap, HashSet};

const ROW_HEIGHT: f32 = 28.0;
const LANE_WIDTH: f32 = 14.0;
const DOT_RADIUS: f32 = 4.0;
const GRAPH_COLORS: [Color32; 7] = [
    Color32::from_rgb(55, 138, 221),
    Color32::from_rgb(214, 119, 70),
    Color32::from_rgb(90, 185, 118),
    Color32::from_rgb(173, 112, 212),
    Color32::from_rgb(218, 183, 64),
    Color32::from_rgb(74, 193, 196),
    Color32::from_rgb(222, 104, 152),
];

fn lane_color(lane: usize) -> Color32 {
    GRAPH_COLORS[lane % GRAPH_COLORS.len()]
}

struct GraphRow {
    commit_idx: usize,
    /// Which lane the dot sits on
    lane: usize,
    /// Vertical connectors to draw below this row: (lane, color)
    continuing: Vec<(usize, usize)>,
    /// Fork lines from this row's lane out to child lanes: (to_lane, color_lane)
    forking: Vec<(usize, usize)>,
    width: usize,
}

/// Build lane-assignment rows from a topologically sorted commit list.
fn build_graph(commits: &[Commit]) -> Vec<GraphRow> {
    // Map commit id → index in the list
    let id_to_idx: HashMap<&str, usize> = commits
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.as_str(), i))
        .collect();

    // lanes[i] = commit id currently "occupying" lane i (None = free)
    let mut lanes: Vec<Option<usize>> = Vec::new();

    // For each commit: which lane index it was assigned
    let mut commit_lane: Vec<usize> = vec![0; commits.len()];

    // For each commit: which lane indices its parents should occupy
    // (used to build continuing/merging/forking)
    let mut rows: Vec<GraphRow> = Vec::with_capacity(commits.len());

    for (idx, commit) in commits.iter().enumerate() {
        // Find or create the lane for this commit
        let my_lane = lanes
            .iter()
            .position(|slot| *slot == Some(idx))
            .unwrap_or_else(|| {
                // No lane reserved yet — find a free slot or append
                let free = lanes.iter().position(|s| s.is_none());
                let pos = free.unwrap_or(lanes.len());
                if pos == lanes.len() {
                    lanes.push(None);
                }
                pos
            });

        commit_lane[idx] = my_lane;

        // Collect parent indices that are known
        let parent_idxs: Vec<usize> = commit
            .parents
            .iter()
            .filter_map(|pid| id_to_idx.get(pid.as_str()).copied())
            .collect();

        // --- Assign lanes to parents ---
        // First parent inherits our lane; additional parents get new lanes.
        let mut forking = Vec::new();
        for (pi, &pidx) in parent_idxs.iter().enumerate() {
            // Check if parent already has a lane reserved
            let already = lanes.iter().position(|s| *s == Some(pidx));
            if already.is_none() {
                let target_lane = if pi == 0 {
                    my_lane
                } else {
                    // Append a new lane
                    let pos = lanes.iter().position(|s| s.is_none()).unwrap_or(lanes.len());
                    if pos == lanes.len() {
                        lanes.push(None);
                    }
                    pos
                };
                lanes[target_lane] = Some(pidx);
                if target_lane != my_lane {
                    forking.push((target_lane, target_lane));
                }
            } else if let Some(al) = already {
                if al != my_lane {
                    forking.push((al, al));
                }
            }
        }

        // Release our lane if it won't be used by anyone further down
        // (i.e., no parent claims this lane via first-parent inheritance)
        let first_parent_idx = parent_idxs.first().copied();
        let lane_claimed = first_parent_idx.map_or(false, |pidx| {
            lanes.get(my_lane) == Some(&Some(pidx))
        });
        if !lane_claimed {
            lanes[my_lane] = None;
        }

        // Continuing lanes: all occupied slots that are not my_lane
        let continuing: Vec<(usize, usize)> = lanes
            .iter()
            .enumerate()
            .filter_map(|(li, slot)| {
                if slot.is_some() && li != my_lane {
                    Some((li, li))
                } else {
                    None
                }
            })
            .collect();

        let width = lanes.iter().rposition(|s| s.is_some()).map_or(1, |p| p + 2);

        rows.push(GraphRow {
            commit_idx: idx,
            lane: my_lane,
            continuing,
            forking,
            width,
        });
    }

    rows
}

/// BFS from `tip_id` through `commits` parent links; returns all reachable IDs.
fn reachable_from(tip_id: &str, commits: &[Commit]) -> HashSet<String> {
    let parent_map: HashMap<&str, &[String]> =
        commits.iter().map(|c| (c.id.as_str(), c.parents.as_slice())).collect();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue = vec![tip_id.to_owned()];
    while let Some(id) = queue.pop() {
        if visited.insert(id.clone()) {
            if let Some(parents) = parent_map.get(id.as_str()) {
                queue.extend(parents.iter().cloned());
            }
        }
    }
    visited
}

pub struct CommitGraph {
    rows: Vec<GraphRow>,
    selected: Option<usize>,
    /// Commit list snapshot used to build `rows` (detect staleness by length)
    built_for_len: usize,
}

impl CommitGraph {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            selected: None,
            built_for_len: usize::MAX,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) -> Option<String> {
        let commits: Vec<Commit> = {
            match state.repository_state.as_ref() {
                Some(rs) => rs.commits.clone(),
                None => {
                    let p = crate::ui::colors::get(state.dark_mode);
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("No repository loaded")
                                .color(p.text_tertiary)
                                .size(13.0),
                        );
                    });
                    return None;
                }
            }
        };

        if commits.is_empty() {
            let p = crate::ui::colors::get(state.dark_mode);
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No commits yet")
                        .color(p.text_tertiary)
                        .size(13.0),
                );
            });
            return None;
        }

        // Rebuild graph layout when commit list changes
        if self.built_for_len != commits.len() {
            self.rows = build_graph(&commits);
            self.built_for_len = commits.len();
        }

        let p = crate::ui::colors::get(state.dark_mode);
        let row_count = self.rows.len();
        let max_lanes = self.rows.iter().map(|r| r.width).max().unwrap_or(1);
        let graph_col_width = max_lanes as f32 * LANE_WIDTH + 8.0;

        let tag_map: HashMap<String, Vec<String>> = {
            let mut m: HashMap<String, Vec<String>> = HashMap::new();
            if let Some(rs) = state.repository_state.as_ref() {
                for tag in &rs.tags {
                    m.entry(tag.commit_id.clone()).or_default().push(tag.name.clone());
                }
            }
            m
        };

        // Commits reachable from the selected (or current) branch — used to dim others.
        let on_branch: HashSet<String> = {
            let tip = state.repository_state.as_ref().and_then(|rs| {
                let name = state.ui_state.selected_branch_name()
                    .or_else(|| rs.branches.iter().find(|b| b.is_current).map(|b| b.name.as_str()));
                name.and_then(|n| rs.branches.iter().find(|b| b.name == n).map(|b| b.commit_id.clone()))
            });
            tip.map(|id| reachable_from(&id, &commits)).unwrap_or_default()
        };

        ui.spacing_mut().item_spacing.y = 0.0;

        let mut cherry_pick_id: Option<String> = None;
        let mut cherry_pick_no_commit_id: Option<String> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, ROW_HEIGHT, row_count, |ui, visible| {
                for row_idx in visible {
                    let row = &self.rows[row_idx];
                    let commit = &commits[row.commit_idx];
                    let selected = self.selected == Some(row_idx);

                    let (response, painter) = ui.allocate_painter(
                        Vec2::new(ui.available_width(), ROW_HEIGHT),
                        egui::Sense::click(),
                    );
                    let rect = response.rect;

                    // Row background
                    if selected {
                        painter.rect_filled(rect, 0.0, p.accent_sel_bg);
                    } else if response.hovered() {
                        painter.rect_filled(rect, 0.0, p.bg_secondary);
                    }

                    if response.clicked() {
                        self.selected = Some(row_idx);
                    }

                    // Context menu
                    let commit_id = commit.id.clone();
                    response.context_menu(|ui| {
                        if ui.button("Cherry-pick").clicked() {
                            cherry_pick_id = Some(commit_id.clone());
                            ui.close_menu();
                        }
                        if ui.button("Cherry-pick (edit message)").clicked() {
                            cherry_pick_no_commit_id = Some(commit_id.clone());
                            ui.close_menu();
                        }
                        if ui.button("Copy hash").clicked() {
                            ui.output_mut(|o| o.copied_text = commit_id.clone());
                            ui.close_menu();
                        }
                    });

                    // Tooltip: full commit message on hover
                    response.on_hover_text(commit.message.trim());

                    // --- Graph drawing (clipped to its column so lines never bleed into text) ---
                    let graph_rect = Rect::from_min_size(
                        rect.min,
                        Vec2::new(graph_col_width, ROW_HEIGHT),
                    );
                    let graph_painter = painter.with_clip_rect(graph_rect);
                    draw_graph_cell(&graph_painter, graph_rect, row, row_idx, row_count);

                    // --- Text columns ---
                    let text_x = rect.min.x + graph_col_width + 4.0;
                    let text_y = rect.center().y;

                    // Short hash
                    let hash = &commit.id[..7.min(commit.id.len())];
                    painter.text(
                        Pos2::new(text_x, text_y),
                        egui::Align2::LEFT_CENTER,
                        hash,
                        egui::FontId::monospace(11.0),
                        p.text_tertiary,
                    );

                    // Branch labels: show any branch names pointing to this commit
                    let label_x = text_x + 52.0;
                    let mut label_end_x = label_x;

                    if let Some(rs) = state.repository_state.as_ref() {
                        for branch in rs.branches.iter().filter(|b| b.commit_id == commit.id) {
                            let label_text = branch.name.as_str();
                            let label_color = if branch.is_current {
                                p.accent_text
                            } else {
                                p.text_secondary
                            };
                            let bg = if branch.is_current {
                                p.accent_sel_bg
                            } else {
                                p.bg_tertiary
                            };

                            let galley = ui.fonts(|f| {
                                f.layout_no_wrap(
                                    label_text.to_string(),
                                    egui::FontId::proportional(10.0),
                                    label_color,
                                )
                            });
                            let lw = galley.size().x + 6.0;
                            let lh = galley.size().y + 2.0;
                            let label_rect = Rect::from_min_size(
                                Pos2::new(label_end_x, text_y - lh / 2.0),
                                Vec2::new(lw, lh),
                            );
                            painter.rect_filled(label_rect, 3.0, bg);
                            painter.rect_stroke(
                                label_rect,
                                3.0,
                                Stroke::new(1.0, p.border),
                            );
                            painter.galley(
                                Pos2::new(label_end_x + 3.0, text_y - galley.size().y / 2.0),
                                galley,
                                label_color,
                            );
                            label_end_x += lw + 4.0;
                        }

                        // Tag chips after branch chips
                        if let Some(tag_names) = tag_map.get(&commit.id) {
                            let tag_color = p.status_modified;
                            for tag_name in tag_names {
                                let galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        tag_name.to_string(),
                                        egui::FontId::proportional(10.0),
                                        tag_color,
                                    )
                                });
                                let lw = galley.size().x + 6.0;
                                let lh = galley.size().y + 2.0;
                                let tag_rect = Rect::from_min_size(
                                    Pos2::new(label_end_x, text_y - lh / 2.0),
                                    Vec2::new(lw, lh),
                                );
                                painter.rect_filled(tag_rect, 3.0, p.bg_tertiary);
                                painter.rect_stroke(tag_rect, 3.0, Stroke::new(1.0, p.border));
                                painter.galley(
                                    Pos2::new(label_end_x + 3.0, text_y - galley.size().y / 2.0),
                                    galley,
                                    tag_color,
                                );
                                label_end_x += lw + 4.0;
                            }
                        }
                    }

                    // Dim commits not reachable from the selected/current branch
                    let is_on_branch = on_branch.is_empty() || on_branch.contains(&commit.id);
                    let msg_color = if selected {
                        p.accent_text
                    } else if is_on_branch {
                        p.text_primary
                    } else {
                        p.text_tertiary
                    };

                    // Commit message — measure author+date first so we know how much is left
                    let msg_x = label_end_x + 4.0;
                    let summary = commit.message.lines().next().unwrap_or("").trim();
                    let msg_font = egui::FontId::proportional(12.0);
                    let meta_font = egui::FontId::proportional(10.5);
                    let ts = format_timestamp(commit.timestamp);
                    let meta = format!("{} · {}", commit.author, ts);
                    let meta_w = ui.fonts(|f| {
                        f.layout_no_wrap(meta.clone(), meta_font.clone(), p.text_tertiary).size().x
                    });
                    let avail_msg = (rect.max.x - msg_x - meta_w - 16.0).max(0.0);
                    let display_msg = ellipsize(summary, avail_msg, ui, &msg_font);
                    painter.text(
                        Pos2::new(msg_x, text_y),
                        egui::Align2::LEFT_CENTER,
                        &display_msg,
                        msg_font,
                        msg_color,
                    );

                    // Author + date (right side)
                    painter.text(
                        Pos2::new(rect.max.x - 8.0, text_y),
                        egui::Align2::RIGHT_CENTER,
                        &meta,
                        meta_font,
                        p.text_tertiary,
                    );
                }
            });

        if let Some(id) = cherry_pick_no_commit_id {
            if let Some(commit) = commits.iter().find(|c| c.id == id) {
                let msg = commit.message.trim().to_owned();
                let short = &id[..7.min(id.len())];
                match state.repository_state_mut().cherry_pick_no_commit(&id) {
                    Ok(()) => {
                        state.ui_state.set_commit_message(&msg);
                        state.set_info(format!("Staged {} — edit message and commit", short));
                    }
                    Err(e) => {
                        let em = e.to_string();
                        if em.contains("allow-empty") || em.contains("is now empty") {
                            state.set_info(format!("Skipped {}: changes already present", short));
                        } else {
                            state.set_error(format!("Cherry-pick failed: {}", e));
                        }
                    }
                }
            }
        }

        cherry_pick_id
    }

    pub fn selected_commit_id<'a>(&self, commits: &'a [crate::models::Commit]) -> Option<&'a str> {
        self.selected.and_then(|idx| commits.get(idx)).map(|c| c.id.as_str())
    }
}

impl Default for CommitGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_graph_cell(
    painter: &Painter,
    rect: Rect,
    row: &GraphRow,
    row_idx: usize,
    row_count: usize,
) {
    let cx = |lane: usize| rect.min.x + lane as f32 * LANE_WIDTH + LANE_WIDTH / 2.0 + 4.0;
    let mid_y = rect.center().y;
    let top_y = rect.min.y;
    let bot_y = rect.max.y;

    // Vertical connectors for lanes passing through (not this row's dot lane)
    for &(lane, color_idx) in &row.continuing {
        let x = cx(lane);
        painter.line_segment(
            [Pos2::new(x, top_y), Pos2::new(x, bot_y)],
            Stroke::new(1.5, lane_color(color_idx)),
        );
    }

    // Line from top to dot for this lane (if not first row)
    if row_idx > 0 {
        let x = cx(row.lane);
        painter.line_segment(
            [Pos2::new(x, top_y), Pos2::new(x, mid_y)],
            Stroke::new(1.5, lane_color(row.lane)),
        );
    }

    // Fork lines (from dot lane outward, going below)
    for &(to_lane, color_idx) in &row.forking {
        let from_x = cx(row.lane);
        let to_x = cx(to_lane);
        // Diagonal from mid→bot using a quad-like line via two segments
        painter.line_segment(
            [Pos2::new(from_x, mid_y), Pos2::new(to_x, bot_y)],
            Stroke::new(1.5, lane_color(color_idx)),
        );
    }

    // Line from dot to bottom on our lane (if not last row and not fully forked away)
    if row_idx + 1 < row_count {
        let x = cx(row.lane);
        // Only draw if our lane continues (either we have a first parent or we fork into it)
        let first_parent_continues = !row.forking.iter().any(|(fl, _)| *fl == row.lane)
            || row.continuing.iter().any(|(cl, _)| *cl == row.lane);
        if first_parent_continues {
            painter.line_segment(
                [Pos2::new(x, mid_y), Pos2::new(x, bot_y)],
                Stroke::new(1.5, lane_color(row.lane)),
            );
        }
    }

    // Draw the dot
    painter.circle_filled(
        Pos2::new(cx(row.lane), mid_y),
        DOT_RADIUS,
        lane_color(row.lane),
    );
    painter.circle_stroke(
        Pos2::new(cx(row.lane), mid_y),
        DOT_RADIUS,
        Stroke::new(1.0, Color32::from_black_alpha(60)),
    );
}

/// Truncate `s` to fit within `max_px` using actual font measurement, appending `…` if needed.
fn ellipsize(s: &str, max_px: f32, ui: &egui::Ui, font: &egui::FontId) -> String {
    if max_px <= 0.0 {
        return String::new();
    }
    let full_w = ui.fonts(|f| f.layout_no_wrap(s.to_string(), font.clone(), Color32::WHITE).size().x);
    if full_w <= max_px {
        return s.to_string();
    }
    let ell_w = ui.fonts(|f| {
        f.layout_no_wrap("…".to_string(), font.clone(), Color32::WHITE).size().x
    });
    let budget = (max_px - ell_w).max(0.0);
    // Binary search for the longest prefix that fits within `budget`
    let chars: Vec<char> = s.chars().collect();
    let mut lo = 0usize;
    let mut hi = chars.len();
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let slice: String = chars[..mid].iter().collect();
        let w = ui.fonts(|f| f.layout_no_wrap(slice, font.clone(), Color32::WHITE).size().x);
        if w <= budget { lo = mid; } else { hi = mid - 1; }
    }
    format!("{}…", chars[..lo].iter().collect::<String>())
}

fn format_timestamp(ts: i64) -> String {
    // Convert Unix timestamp to a simple relative or absolute label
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff = now - ts;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 86400 * 30 {
        format!("{}d ago", diff / 86400)
    } else if diff < 86400 * 365 {
        format!("{}mo ago", diff / (86400 * 30))
    } else {
        // Fallback: year
        // Simple date from epoch
        let days = ts / 86400;
        let years_since_1970 = days / 365;
        let year = 1970 + years_since_1970;
        format!("{}", year)
    }
}
