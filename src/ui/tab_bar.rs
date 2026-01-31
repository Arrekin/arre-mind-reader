//! Tab bar UI component.
//!
//! Renders the tab strip and emits TabSelect/TabClose events.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::tabs::{ActiveTab, TabClose, TabMarker, TabSelect};
use super::NewTabDialog;

pub fn tab_bar_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut dialog: ResMut<NewTabDialog>,
    tabs: Query<(Entity, &Name, Has<ActiveTab>), With<TabMarker>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    let tab_info: Vec<(Entity, Name, bool)> = tabs.iter()
        .map(|(e, name, is_active)| (e, name.clone(), is_active))
        .collect();
    
    egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (entity, name, is_active) in &tab_info {
                let label = if *is_active {
                    egui::RichText::new(name.as_str()).strong()
                } else {
                    egui::RichText::new(name.as_str())
                };
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(*is_active, label).clicked() {
                        commands.trigger(TabSelect::from(*entity));
                    }
                    if ui.small_button("×").clicked() {
                        commands.trigger(TabClose::from(*entity));
                    }
                });
                ui.separator();
            }
            
            if ui.button("+ New").clicked() {
                dialog.open = true;
                dialog.text_input.clear();
            }
        });
    });
}
