//! Tab bar UI component.
//!
//! Renders the tab strip and emits TabSelect/TabClose events.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::tabs::{ActiveTab, HomepageTab, TabClose, TabMarker, TabOrder, TabSelect};
use super::NewTabDialog;

pub fn tab_bar_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut dialog: ResMut<NewTabDialog>,
    tab_order: Res<TabOrder>,
    tabs: Query<(&Name, Has<HomepageTab>, Has<ActiveTab>), With<TabMarker>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    
    egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for &entity in tab_order.entities().iter() {
                let Ok((name, is_homepage, is_active)) = tabs.get(entity) else { continue };
                
                let label = if is_active {
                    egui::RichText::new(name.as_str()).strong()
                } else {
                    egui::RichText::new(name.as_str())
                };
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_active, label).clicked() {
                        commands.trigger(TabSelect::from(entity));
                    }
                    if !is_homepage {
                        if ui.small_button("×").clicked() {
                            commands.trigger(TabClose::from(entity));
                        }
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
