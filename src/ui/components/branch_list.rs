use crate::state::AppState;
use crate::ui::colors::Palette;
use eframe::egui;
use std::cell::Cell;
use std::collections::HashMap;

pub struct BranchList {
    filter: String,
    branches_open: bool,
    remotes_open: bool,
    tags_open: bool,
    stashes_open: bool,
    worktrees_open: bool,
    submodules_open: bool,
    remote_sections: HashMap<String, bool>,
}

impl BranchList {
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            branches_open: true,
            remotes_open: true,
            tags_open: false,
            stashes_open: false,
            worktrees_open: false,
            submodules_open: false,
            remote_sections: HashMap::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        let p = crate::ui::colors::get(state.dark_mode);

        ui.add_space(6.0);
        egui::Frame::none()
            .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 0.0, bottom: 0.0 })
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .hint_text("Filter…")
                        .desired_width(f32::INFINITY),
                );
            });
        ui.add_space(4.0);

        if !state.has_repository() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No repository loaded").color(p.text_tertiary).small());
            });
            return;
        }

        let filter_lower = self.filter.to_lowercase();

        let local_branches: Vec<crate::models::Branch> = {
            let branches = &state.repository_state().branches;
            branches
                .iter()
                .filter(|b| !b.is_remote)
                .filter(|b| filter_lower.is_empty() || b.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        };

        let remote_branches: Vec<crate::models::Branch> = {
            let branches = &state.repository_state().branches;
            branches
                .iter()
                .filter(|b| b.is_remote)
                .filter(|b| filter_lower.is_empty() || b.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        };

        let stashes: Vec<crate::models::StashEntry> = state.repository_state().stashes.clone();
        let tags: Vec<crate::models::Tag> = state.repository_state().tags.clone();
        let worktrees: Vec<crate::models::WorktreeEntry> = state.repository_state().worktrees.clone();
        let (ahead, behind) = state.repository_state.as_ref()
            .map(|rs| (rs.ahead, rs.behind))
            .unwrap_or((0, 0));
        let selected_branch = state.ui_state.selected_branch.clone();
        let mut branch_to_select: Option<String> = None;
        let mut branch_to_checkout: Option<String> = None;
        let mut branch_to_delete: Option<String> = None;
        let mut branch_to_rename: Option<String> = None;
        let mut branch_to_merge: Option<String> = None;
        let mut stash_to_pop: Option<usize> = None;
        let mut stash_to_drop: Option<usize> = None;
        let mut tag_to_push: Option<String> = None;
        let mut worktree_to_remove: Option<std::path::PathBuf> = None;
        let mut show_add_worktree = false;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Self::show_section_header(ui, p, "BRANCHES", &mut self.branches_open, false);
                if self.branches_open {
                    for branch in &local_branches {
                        let ab = if branch.is_current { (ahead, behind) } else { (0, 0) };
                        let (sel, chk, del, ren, mrg) = Self::show_branch_row(ui, p, &selected_branch, branch, 18.0, &branch.name, ab);
                        if sel { branch_to_select = Some(branch.name.clone()); }
                        if chk { branch_to_checkout = Some(branch.name.clone()); }
                        if del { branch_to_delete = Some(branch.name.clone()); }
                        if ren { branch_to_rename = Some(branch.name.clone()); }
                        if mrg { branch_to_merge = Some(branch.name.clone()); }
                    }
                    if local_branches.is_empty() {
                        Self::show_empty_hint(ui, p, "No local branches", 22.0);
                    }
                }

                Self::show_section_header(ui, p, "REMOTES", &mut self.remotes_open, false);
                if self.remotes_open {
                    let mut remote_groups: std::collections::BTreeMap<&str, Vec<&crate::models::Branch>> =
                        std::collections::BTreeMap::new();
                    for branch in &remote_branches {
                        if let Some((remote, _rest)) = branch.name.split_once('/') {
                            remote_groups.entry(remote).or_default().push(branch);
                        }
                    }
                    if remote_groups.is_empty() {
                        Self::show_empty_hint(ui, p, "No remote branches", 22.0);
                    } else {
                        for (remote_name, branches) in &remote_groups {
                            let open = self
                                .remote_sections
                                .entry(remote_name.to_string())
                                .or_insert(true);
                            let indent = 28.0;
                            Self::show_remote_header(ui, p, remote_name, open, indent);
                            if *open {
                                for branch in branches {
                                    let label = branch.name.split_once('/').map(|(_, r)| r).unwrap_or(&branch.name);
                                    if label == "HEAD" { continue; }
                                    let (sel, chk, _del, _ren, _mrg) =
                                        Self::show_branch_row(ui, p, &selected_branch, branch, indent + 12.0, label, (0, 0));
                                    if sel {
                                        branch_to_select = Some(branch.name.clone());
                                    }
                                    if chk {
                                        branch_to_checkout = Some(branch.name.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                Self::show_section_header(ui, p, "TAGS", &mut self.tags_open, false);
                if self.tags_open {
                    if tags.is_empty() {
                        Self::show_empty_hint(ui, p, "No tags", 22.0);
                    } else {
                        for tag in &tags {
                            let push = Self::show_tag_row(ui, p, &tag.name, 18.0);
                            if push {
                                tag_to_push = Some(tag.name.clone());
                            }
                        }
                    }
                }

                Self::show_section_header(ui, p, "STASHES", &mut self.stashes_open, false);
                if self.stashes_open {
                    if stashes.is_empty() {
                        Self::show_empty_hint(ui, p, "No stashes", 22.0);
                    } else {
                        for stash in &stashes {
                            if let Some((pop, drop)) = Self::show_stash_row(ui, p, stash) {
                                if pop { stash_to_pop = Some(stash.index); }
                                if drop { stash_to_drop = Some(stash.index); }
                            }
                        }
                    }
                }

                let wt_add = Self::show_section_header(ui, p, "WORKTREES", &mut self.worktrees_open, true);
                if wt_add { show_add_worktree = true; }
                if self.worktrees_open {
                    if worktrees.is_empty() {
                        Self::show_empty_hint(ui, p, "No worktrees", 22.0);
                    } else {
                        for wt in &worktrees {
                            if let Some(path) = Self::show_worktree_row(ui, p, wt) {
                                worktree_to_remove = Some(path);
                            }
                        }
                    }
                }

                Self::show_section_header(ui, p, "SUBMODULES", &mut self.submodules_open, false);
            });

        if let Some(name) = tag_to_push {
            state.ui_state.pending_action = Some(crate::state::PendingAction::PushTag(name));
        }
        if let Some(name) = branch_to_select {
            state.ui_state.select_branch(name);
        }
        if let Some(name) = branch_to_checkout {
            if let Err(e) = state.repository_state_mut().checkout_branch(&name) {
                let msg = e.to_string().to_lowercase();
                let display = if msg.contains("conflict") {
                    format!("Cannot checkout '{}': resolve conflicts first (see Changes panel)", name)
                } else {
                    format!("Failed to checkout {}: {}", name, e)
                };
                state.set_error(display);
            } else {
                state.set_info(format!("Checked out: {}", name));
            }
        }
        if let Some(name) = branch_to_delete {
            match state.repository_state_mut().delete_branch(&name) {
                Ok(()) => state.set_info(format!("Branch '{}' deleted", name)),
                Err(e) => state.set_error(format!("Failed to delete '{}': {}", name, e)),
            }
        }
        if let Some(name) = branch_to_rename {
            state.ui_state.rename_branch_old = name.clone();
            state.ui_state.rename_branch_new = name;
            state.ui_state.show_rename_branch_dialog = true;
        }
        if let Some(name) = branch_to_merge {
            match state.repository_state_mut().merge_branch(&name) {
                Ok(()) => state.set_info(format!("Merged '{}' into current branch", name)),
                Err(e) => state.set_error(format!("Merge failed: {}", e)),
            }
        }
        if let Some(index) = stash_to_pop {
            match state.repository_state_mut().stash_pop(index) {
                Ok(()) => state.set_info("Stash applied and removed".to_string()),
                Err(e) => state.set_error(format!("Failed to pop stash: {}", e)),
            }
        }
        if let Some(index) = stash_to_drop {
            match state.repository_state_mut().stash_drop(index) {
                Ok(()) => state.set_info("Stash dropped".to_string()),
                Err(e) => state.set_error(format!("Failed to drop stash: {}", e)),
            }
        }
        if let Some(path) = worktree_to_remove {
            match state.repository_state_mut().remove_worktree(&path) {
                Ok(()) => state.set_info(format!("Worktree '{}' removed", path.display())),
                Err(e) => state.set_error(format!("Failed to remove worktree: {}", e)),
            }
        }
        if show_add_worktree {
            state.ui_state.show_create_worktree_dialog = true;
        }

        // Worktree create dialog
        if state.ui_state.show_create_worktree_dialog {
            let ctx = ui.ctx().clone();
            let mut do_create = false;
            let mut do_cancel = false;
            egui::Window::new("Add Worktree")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(&ctx, |ui| {
                    ui.label("Path:");
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut state.ui_state.new_worktree_path).desired_width(240.0));
                        if ui.button("Browse…").clicked()
                            && let Some(p) = crate::ui::FileDialog::open_directory() {
                                state.ui_state.new_worktree_path = p.to_string_lossy().to_string();
                            }
                    });
                    ui.label("Branch:");
                    ui.add(egui::TextEdit::singleline(&mut state.ui_state.new_worktree_branch).desired_width(240.0));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Add").clicked() { do_create = true; }
                        if ui.button("Cancel").clicked() { do_cancel = true; }
                    });
                });
            if do_create && !state.ui_state.new_worktree_path.is_empty() && !state.ui_state.new_worktree_branch.is_empty() {
                let path = std::path::PathBuf::from(state.ui_state.new_worktree_path.clone());
                let branch = state.ui_state.new_worktree_branch.clone();
                state.ui_state.show_create_worktree_dialog = false;
                state.ui_state.new_worktree_path.clear();
                state.ui_state.new_worktree_branch.clear();
                match state.repository_state_mut().add_worktree(&path, &branch) {
                    Ok(()) => state.set_info(format!("Worktree added at '{}'", path.display())),
                    Err(e) => state.set_error(format!("Failed to add worktree: {}", e)),
                }
            }
            if do_cancel {
                state.ui_state.show_create_worktree_dialog = false;
                state.ui_state.new_worktree_path.clear();
                state.ui_state.new_worktree_branch.clear();
            }
        }

        // Create branch dialog
        if state.ui_state.show_create_branch_dialog {
            let ctx = ui.ctx().clone();
            let mut do_create = false;
            let mut do_cancel = false;
            egui::Window::new("Create Branch")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(&ctx, |ui| {
                    ui.label("Branch name:");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.ui_state.new_branch_name)
                            .desired_width(240.0),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_create = true;
                    }
                    ui.checkbox(&mut state.ui_state.new_branch_checkout, "Checkout after create");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let name_ok = !state.ui_state.new_branch_name.trim().is_empty();
                        if ui.add_enabled(name_ok, egui::Button::new("Create")).clicked() {
                            do_create = true;
                        }
                        if ui.button("Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });
            if do_create && !state.ui_state.new_branch_name.trim().is_empty() {
                let name = state.ui_state.new_branch_name.trim().to_string();
                let checkout = state.ui_state.new_branch_checkout;
                state.ui_state.show_create_branch_dialog = false;
                state.ui_state.new_branch_name.clear();
                match state.repository_state_mut().create_branch(&name, checkout) {
                    Ok(()) => state.set_info(format!("Branch '{}' created", name)),
                    Err(e) => state.set_error(format!("Failed to create branch: {}", e)),
                }
            }
            if do_cancel {
                state.ui_state.show_create_branch_dialog = false;
                state.ui_state.new_branch_name.clear();
            }
        }

        // Rename branch dialog
        if state.ui_state.show_rename_branch_dialog {
            let ctx = ui.ctx().clone();
            let mut do_rename = false;
            let mut do_cancel = false;
            let old_name = state.ui_state.rename_branch_old.clone();
            egui::Window::new("Rename Branch")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(&ctx, |ui| {
                    ui.label(format!("Rename \"{}\" to:", old_name));
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.ui_state.rename_branch_new)
                            .desired_width(240.0),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_rename = true;
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let name_ok = !state.ui_state.rename_branch_new.trim().is_empty()
                            && state.ui_state.rename_branch_new != old_name;
                        if ui.add_enabled(name_ok, egui::Button::new("Rename")).clicked() {
                            do_rename = true;
                        }
                        if ui.button("Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });
            if do_rename && !state.ui_state.rename_branch_new.trim().is_empty() {
                let new_name = state.ui_state.rename_branch_new.trim().to_string();
                state.ui_state.show_rename_branch_dialog = false;
                match state.repository_state_mut().rename_branch(&old_name, &new_name) {
                    Ok(()) => state.set_info(format!("Branch renamed to '{}'", new_name)),
                    Err(e) => state.set_error(format!("Failed to rename branch: {}", e)),
                }
            }
            if do_cancel {
                state.ui_state.show_rename_branch_dialog = false;
            }
        }

        // Create tag dialog
        if state.ui_state.show_create_tag_dialog {
            let ctx = ui.ctx().clone();
            let mut do_create = false;
            let mut do_cancel = false;
            egui::Window::new("Create Tag")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(&ctx, |ui| {
                    ui.label("Tag name:");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.ui_state.new_tag_name)
                            .desired_width(240.0),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_create = true;
                    }
                    ui.add_space(4.0);
                    ui.label("Message (optional):");
                    ui.add(
                        egui::TextEdit::multiline(&mut state.ui_state.new_tag_message)
                            .desired_rows(3)
                            .desired_width(240.0),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let name_ok = !state.ui_state.new_tag_name.trim().is_empty();
                        if ui.add_enabled(name_ok, egui::Button::new("Create")).clicked() {
                            do_create = true;
                        }
                        if ui.button("Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });
            if do_create && !state.ui_state.new_tag_name.trim().is_empty() {
                let name = state.ui_state.new_tag_name.trim().to_string();
                let message = if state.ui_state.new_tag_message.trim().is_empty() {
                    name.clone()
                } else {
                    state.ui_state.new_tag_message.trim().to_string()
                };
                state.ui_state.show_create_tag_dialog = false;
                state.ui_state.new_tag_name.clear();
                state.ui_state.new_tag_message.clear();
                match state.repository_state_mut().create_tag(&name, &message) {
                    Ok(()) => state.set_info(format!("Tag '{}' created", name)),
                    Err(e) => state.set_error(format!("Failed to create tag: {}", e)),
                }
            }
            if do_cancel {
                state.ui_state.show_create_tag_dialog = false;
                state.ui_state.new_tag_name.clear();
                state.ui_state.new_tag_message.clear();
            }
        }
    }

    fn show_section_header(
        ui: &mut egui::Ui,
        p: &Palette,
        title: &str,
        open: &mut bool,
        show_add: bool,
    ) -> bool {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if response.hovered() { p.bg_tertiary } else { p.bg_secondary };
            ui.painter().rect_filled(rect, 0.0, bg);
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y - 0.5,
                egui::Stroke::new(0.5, p.border),
            );

            let y = rect.center().y;
            paint_chevron(ui.painter(), egui::pos2(rect.min.x + 11.0, y), *open, p.text_tertiary);
            ui.painter().text(
                egui::pos2(rect.min.x + 18.0, y),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(10.0),
                p.text_secondary,
            );
        }

        let mut add_clicked = false;
        if show_add && response.hovered() {
            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - 22.0, rect.min.y + 3.0),
                egui::vec2(18.0, 18.0),
            );
            let btn_id = ui.id().with(title).with("add");
            let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

            if ui.is_rect_visible(btn_rect) {
                if btn.hovered() {
                    ui.painter().rect_filled(btn_rect, 3.0, p.bg_tertiary);
                }
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "+",
                    egui::FontId::proportional(13.0),
                    p.text_secondary,
                );
            }
            add_clicked = btn.clicked();
        }

        if response.clicked() && !add_clicked {
            *open = !*open;
        }

        add_clicked
    }

    fn show_remote_header(
        ui: &mut egui::Ui,
        p: &Palette,
        name: &str,
        open: &mut bool,
        indent: f32,
    ) {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 22.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if response.hovered() { p.bg_tertiary } else { egui::Color32::TRANSPARENT };
            ui.painter().rect_filled(rect, 0.0, bg);

            let y = rect.center().y;
            paint_chevron(ui.painter(), egui::pos2(rect.min.x + indent - 6.0, y), *open, p.text_tertiary);
            ui.painter().text(
                egui::pos2(rect.min.x + indent + 4.0, y),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(10.5),
                p.text_secondary,
            );
        }

        if response.clicked() {
            *open = !*open;
        }
    }

    fn show_tag_row(ui: &mut egui::Ui, p: &Palette, tag: &str, indent: f32) -> bool {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if response.hovered() { p.bg_tertiary } else { egui::Color32::TRANSPARENT };
            ui.painter().rect_filled(rect, 0.0, bg);

            ui.painter().text(
                egui::pos2(rect.min.x + indent, rect.center().y),
                egui::Align2::LEFT_CENTER,
                tag,
                egui::FontId::proportional(11.0),
                p.text_secondary,
            );
        }

        let push_clicked = Cell::new(false);
        response.context_menu(|ui| {
            if ui.button("Copy name").clicked() {
                ui.output_mut(|o| o.copied_text = tag.to_string());
                ui.close_menu();
            }
            if ui.button("Push to origin").clicked() {
                push_clicked.set(true);
                ui.close_menu();
            }
        });

        if response.clicked() {
            ui.output_mut(|o| o.copied_text = tag.to_string());
        }

        push_clicked.get()
    }

    fn show_branch_row(
        ui: &mut egui::Ui,
        p: &Palette,
        selected_branch: &Option<String>,
        branch: &crate::models::Branch,
        indent: f32,
        label: &str,
        ahead_behind: (usize, usize),
    ) -> (bool, bool, bool, bool, bool) {
        let is_selected = selected_branch.as_ref() == Some(&branch.name);
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if is_selected {
                p.accent_sel_bg
            } else if response.hovered() {
                p.bg_tertiary
            } else {
                egui::Color32::TRANSPARENT
            };
            ui.painter().rect_filled(rect, 0.0, bg);

            let font = egui::FontId::proportional(11.0);
            let y = rect.center().y;
            let mut x = rect.min.x + indent;

            if branch.is_current {
                paint_check(ui.painter(), egui::pos2(x + 5.0, y), p.accent_success);
                x += 14.0;
            }

            let text_color = if is_selected { p.accent_text } else { p.text_secondary };
            ui.painter().text(
                egui::pos2(x, y),
                egui::Align2::LEFT_CENTER,
                label,
                font,
                text_color,
            );

            let (ahead, behind) = ahead_behind;
            if branch.is_current && (ahead > 0 || behind > 0) {
                let sf = egui::FontId::proportional(10.0);
                let mut rx = rect.max.x - 6.0;

                let behind_str = behind.to_string();
                let bw = ui.fonts(|f| f.layout_no_wrap(behind_str.clone(), sf.clone(), egui::Color32::WHITE).size().x);
                let dn_color = if behind > 0 { p.accent_success } else { p.text_tertiary };
                ui.painter().text(egui::pos2(rx - bw, y), egui::Align2::LEFT_CENTER, &behind_str, sf.clone(), p.text_tertiary);
                rx -= bw + 2.0;
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![
                        egui::pos2(rx - 3.0, y - 2.0),
                        egui::pos2(rx + 3.0, y - 2.0),
                        egui::pos2(rx, y + 2.5),
                    ],
                    dn_color, egui::Stroke::NONE,
                ));
                rx -= 10.0;

                let ahead_str = ahead.to_string();
                let aw = ui.fonts(|f| f.layout_no_wrap(ahead_str.clone(), sf.clone(), egui::Color32::WHITE).size().x);
                let up_color = if ahead > 0 { p.accent_success } else { p.text_tertiary };
                ui.painter().text(egui::pos2(rx - aw, y), egui::Align2::LEFT_CENTER, &ahead_str, sf.clone(), p.text_tertiary);
                rx -= aw + 2.0;
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![
                        egui::pos2(rx - 3.0, y + 2.0),
                        egui::pos2(rx + 3.0, y + 2.0),
                        egui::pos2(rx, y - 2.5),
                    ],
                    up_color, egui::Stroke::NONE,
                ));
            }
        }

        let checkout_from_menu = Cell::new(false);
        let delete_from_menu = Cell::new(false);
        let rename_from_menu = Cell::new(false);
        let merge_from_menu = Cell::new(false);
        response.context_menu(|ui| {
            if !branch.is_current && ui.button("Checkout").clicked() {
                checkout_from_menu.set(true);
                ui.close_menu();
            }
            if ui.button("Copy name").clicked() {
                ui.output_mut(|o| o.copied_text = branch.name.clone());
                ui.close_menu();
            }
            if !branch.is_remote {
                if ui.button("Rename").clicked() {
                    rename_from_menu.set(true);
                    ui.close_menu();
                }
                if !branch.is_current
                    && ui.button("Merge into current").clicked() {
                        merge_from_menu.set(true);
                        ui.close_menu();
                    }
            }
            ui.separator();
            if branch.is_current {
                ui.add_enabled(false, egui::Button::new("Delete branch"));
            } else if ui.button("Delete branch").clicked() {
                delete_from_menu.set(true);
                ui.close_menu();
            }
        });

        let checkout = (response.double_clicked() || checkout_from_menu.get()) && !branch.is_current;
        (response.clicked(), checkout, delete_from_menu.get(), rename_from_menu.get(), merge_from_menu.get())
    }

    /// Returns `Some((pop_clicked, drop_clicked))` for the given stash row
    fn show_stash_row(
        ui: &mut egui::Ui,
        p: &Palette,
        stash: &crate::models::StashEntry,
    ) -> Option<(bool, bool)> {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        if response.hovered() {
            ui.painter().rect_filled(rect, 0.0, p.bg_tertiary);
        }

        let y = rect.center().y;
        ui.painter().text(
            egui::pos2(rect.min.x + 22.0, y),
            egui::Align2::LEFT_CENTER,
            &stash.message,
            egui::FontId::proportional(11.0),
            p.text_secondary,
        );

        let pop_clicked = Cell::new(false);
        let drop_clicked = Cell::new(false);
        response.context_menu(|ui| {
            if ui.button("Pop").clicked() {
                pop_clicked.set(true);
                ui.close_menu();
            }
            if ui.button("Drop").clicked() {
                drop_clicked.set(true);
                ui.close_menu();
            }
        });

        Some((pop_clicked.get(), drop_clicked.get()))
    }

    fn show_empty_hint(ui: &mut egui::Ui, p: &Palette, text: &str, indent: f32) {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            ui.label(egui::RichText::new(text).color(p.text_tertiary).small());
        });
    }

    /// Renders a worktree row. Returns `Some(path)` if the user chose to remove this worktree.
    fn show_worktree_row(
        ui: &mut egui::Ui,
        p: &Palette,
        wt: &crate::models::WorktreeEntry,
    ) -> Option<std::path::PathBuf> {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            if response.hovered() {
                ui.painter().rect_filled(rect, 0.0, p.bg_tertiary);
            }

            let y = rect.center().y;
            let name = wt.path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| wt.path.to_string_lossy().to_string());

            let label = if let Some(branch) = &wt.branch {
                format!("{} ({})", name, branch)
            } else {
                name
            };

            let color = if wt.is_current { p.accent_text } else { p.text_secondary };
            ui.painter().text(
                egui::pos2(rect.min.x + 22.0, y),
                egui::Align2::LEFT_CENTER,
                &label,
                egui::FontId::proportional(11.0),
                color,
            );
        }

        let remove_clicked = Cell::new(false);
        response.context_menu(|ui| {
            if ui.button("Copy path").clicked() {
                ui.output_mut(|o| o.copied_text = wt.path.to_string_lossy().to_string());
                ui.close_menu();
            }
            if !wt.is_current
                && ui.button("Remove").clicked() {
                    remove_clicked.set(true);
                    ui.close_menu();
                }
        });

        if remove_clicked.get() { Some(wt.path.clone()) } else { None }
    }
}

impl Default for BranchList {
    fn default() -> Self {
        Self::new()
    }
}

fn paint_chevron(painter: &egui::Painter, center: egui::Pos2, open: bool, color: egui::Color32) {
    let points = if open {
        vec![
            egui::pos2(center.x - 4.0, center.y - 2.5),
            egui::pos2(center.x + 4.0, center.y - 2.5),
            egui::pos2(center.x, center.y + 2.5),
        ]
    } else {
        vec![
            egui::pos2(center.x - 2.5, center.y - 4.0),
            egui::pos2(center.x + 2.5, center.y),
            egui::pos2(center.x - 2.5, center.y + 4.0),
        ]
    };
    painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

fn paint_check(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    painter.line_segment(
        [egui::pos2(center.x - 4.0, center.y), egui::pos2(center.x - 1.0, center.y + 3.0)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(center.x - 1.0, center.y + 3.0), egui::pos2(center.x + 4.0, center.y - 3.0)],
        stroke,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_list_new() {
        let list = BranchList::new();
        assert!(list.filter.is_empty());
        assert!(list.branches_open);
        assert!(list.remotes_open);
        assert!(!list.tags_open);
        assert!(!list.stashes_open);
        assert!(!list.submodules_open);
        assert!(list.remote_sections.is_empty());
    }
}
