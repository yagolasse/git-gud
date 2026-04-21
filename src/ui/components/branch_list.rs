//! Branch list component for Git Gud
//!
//! This component displays a list of branches in the repository
//! and allows the user to select and checkout branches.

use crate::state::AppState;
use eframe::egui;

/// Branch list UI component
pub struct BranchList {
    /// Filter text for branches
    filter: String,
}

impl BranchList {
    /// Create a new branch list component
    pub fn new() -> Self {
        Self {
            filter: String::new(),
        }
    }
    
    /// Show the branch list component
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Branches");
        
        // Filter input
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter);
            if ui.button("✕").clicked() {
                self.filter.clear();
            }
        });
        
        ui.separator();
        
        // Check if repository is loaded
        if !state.has_repository() {
            ui.label("No repository loaded");
            return;
        }
        
        let branches = &state.repository_state().branches;
        
        if branches.is_empty() {
            ui.label("No branches found");
            return;
        }
        
        // Apply filter and collect branch names
        let filtered_branches: Vec<&crate::models::Branch> = if self.filter.is_empty() {
            branches.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            branches.iter()
                .filter(|branch| branch.name.to_lowercase().contains(&filter_lower))
                .collect()
        };
        
        // Track interactions
        let mut branch_to_select = None;
        let mut branch_to_checkout = None;
        
        // Display branches with fixed scrollbar positioning
        let scroll_area = egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 100.0)
            .auto_shrink([false, false]); // Don't shrink, always show scroll area
        
        scroll_area.show(ui, |ui| {
            // Use a container to ensure content fills width
            ui.vertical(|ui| {
                for branch in filtered_branches {
                    ui.horizontal(|ui| {
                        ui.set_min_width(ui.available_width());
                        let (selected, checkout) = self.show_branch_ui(ui, state, branch);
                        if selected {
                            branch_to_select = Some(branch.name.clone());
                        }
                        if checkout {
                            branch_to_checkout = Some(branch.name.clone());
                        }
                    });
                }
            });
        });
        
        // Handle interactions outside the closure
        if let Some(branch_name) = branch_to_select {
            state.ui_state.select_branch(branch_name);
        }
        
        if let Some(branch_name) = branch_to_checkout {
            if let Err(e) = state.repository_state_mut().checkout_branch(&branch_name) {
                state.set_error(format!("Failed to checkout branch {}: {}", branch_name, e));
            } else {
                state.set_info(format!("Checked out branch: {}", branch_name));
            }
        }
        
        ui.separator();
        
        // Branch actions
        ui.horizontal(|ui| {
            if ui.button("New Branch").clicked() {
                // TODO: Implement create branch dialog
                state.set_info("Create branch feature not yet implemented".to_string());
            }
        });
    }
    
    /// Show a single branch item UI and return interaction flags
    fn show_branch_ui(&mut self, ui: &mut egui::Ui, state: &AppState, branch: &crate::models::Branch) -> (bool, bool) {
        let mut selected = false;
        let mut checkout = false;
        
        ui.horizontal(|ui| {
            // Branch icon and name
            let branch_text = if branch.is_current {
                format!("🌿 {}", branch.name)
            } else if branch.is_remote {
                format!("🌐 {}", branch.name)
            } else {
                format!("🌱 {}", branch.name)
            };
            
            // Selectable label
            let response = ui.selectable_label(
                state.ui_state.selected_branch.as_ref() == Some(&branch.name),
                branch_text,
            );
            
            // Handle selection
            if response.clicked() {
                selected = true;
            }
            
            // Double-click to checkout
            if response.double_clicked() && !branch.is_current {
                checkout = true;
            }
            
            // Context menu
            response.context_menu(|ui| {
                if ui.button("Checkout").clicked() && !branch.is_current {
                    checkout = true;
                    ui.close_menu();
                }
                
                if ui.button("Delete").clicked() && !branch.is_current {
                    // Note: We can't modify state here, so we'll need a different approach
                    // For now, just close the menu
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Copy name").clicked() {
                    ui.output_mut(|o| o.copied_text = branch.name.clone());
                    ui.close_menu();
                }
            });
        });
        
        (selected, checkout)
    }
}

impl Default for BranchList {
    fn default() -> Self {
        Self::new()
    }
}