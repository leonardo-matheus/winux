// Winux Welcome - Progress Bar
// Visual progress indicator for the wizard steps

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, ProgressBar};

/// Creates a progress bar showing wizard completion
pub fn create_progress_bar(total_steps: usize) -> Box {
    let container = Box::new(Orientation::Vertical, 4);
    container.set_margin_start(48);
    container.set_margin_end(48);
    container.set_margin_top(8);
    container.set_margin_bottom(8);

    // Progress bar
    let progress = ProgressBar::new();
    progress.set_fraction(1.0 / total_steps as f64);
    progress.add_css_class("welcome-progress");
    container.append(&progress);

    // Step label
    let label = Label::new(Some(&format!("Passo 1 de {}", total_steps)));
    label.add_css_class("dim-label");
    label.add_css_class("caption");
    label.set_halign(gtk4::Align::Center);
    container.append(&label);

    container
}

/// Updates the progress bar to show current step
pub fn update_progress(container: &Box, current_step: usize, total_steps: usize) {
    // Find and update the progress bar
    if let Some(first) = container.first_child() {
        if let Some(progress) = first.downcast_ref::<ProgressBar>() {
            let fraction = current_step as f64 / total_steps as f64;
            progress.set_fraction(fraction);
        }

        // Find and update the label
        if let Some(label_widget) = first.next_sibling() {
            if let Some(label) = label_widget.downcast_ref::<Label>() {
                label.set_text(&format!("Passo {} de {}", current_step, total_steps));
            }
        }
    }
}

/// Creates step indicators (numbered circles)
pub fn create_step_indicators(steps: &[&str]) -> Box {
    let container = Box::new(Orientation::Horizontal, 0);
    container.set_halign(gtk4::Align::Center);
    container.set_margin_top(16);
    container.set_margin_bottom(16);

    for (i, step_name) in steps.iter().enumerate() {
        // Step circle
        let step_box = Box::new(Orientation::Vertical, 4);
        step_box.set_halign(gtk4::Align::Center);

        let circle = Box::new(Orientation::Vertical, 0);
        circle.add_css_class("step-circle");
        circle.set_size_request(32, 32);

        let number = Label::new(Some(&format!("{}", i + 1)));
        number.set_halign(gtk4::Align::Center);
        number.set_valign(gtk4::Align::Center);
        circle.append(&number);

        step_box.append(&circle);

        // Step name
        let name_label = Label::new(Some(*step_name));
        name_label.add_css_class("caption");
        name_label.add_css_class("dim-label");
        step_box.append(&name_label);

        container.append(&step_box);

        // Connector line (except for last step)
        if i < steps.len() - 1 {
            let line = Box::new(Orientation::Horizontal, 0);
            line.add_css_class("step-line");
            line.set_size_request(40, 2);
            line.set_valign(gtk4::Align::Center);
            line.set_margin_bottom(20);
            container.append(&line);
        }
    }

    container
}

/// Updates step indicators to show completed/current/upcoming states
pub fn update_step_indicators(container: &Box, current_step: usize) {
    let mut child = container.first_child();
    let mut step_index = 0;

    while let Some(widget) = child {
        // Check if this is a step box (not a connector line)
        if widget.has_css_class("step-circle") || widget.first_child().map(|c| c.has_css_class("step-circle")).unwrap_or(false) {
            if let Some(circle) = widget.first_child() {
                circle.remove_css_class("completed");
                circle.remove_css_class("current");
                circle.remove_css_class("upcoming");

                if step_index < current_step {
                    circle.add_css_class("completed");
                } else if step_index == current_step {
                    circle.add_css_class("current");
                } else {
                    circle.add_css_class("upcoming");
                }
            }
            step_index += 1;
        }

        child = widget.next_sibling();
    }
}
