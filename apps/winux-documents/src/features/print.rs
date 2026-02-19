//! Document printing functionality

use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::window::AppState;

/// Print the current document
pub fn print_document(window: &adw::ApplicationWindow, state: &AppState) {
    let document = match &state.document {
        Some(doc) => doc,
        None => {
            show_error(window, "No document loaded");
            return;
        }
    };

    // Create print operation
    let print_operation = gtk::PrintOperation::new();

    // Set up print settings
    print_operation.set_n_pages(document.page_count() as i32);
    print_operation.set_use_full_page(false);
    print_operation.set_unit(gtk::Unit::Points);

    // Get document title for the job name
    if let Some(title) = document.title() {
        print_operation.set_job_name(&title);
    }

    // Store document reference for draw_page callback
    // Note: In a real implementation, this would need more careful handling
    // of the document reference across async boundaries

    // Connect to draw-page signal
    // This is a simplified implementation - full implementation would
    // need to handle the document reference properly
    print_operation.connect_draw_page(move |_op, print_context, page_num| {
        // Get Cairo context from print context
        let context = match print_context.cairo_context() {
            Some(ctx) => ctx,
            None => return,
        };

        // The actual rendering would happen here
        // In a full implementation, we'd render the document page to the Cairo context

        // Placeholder: Draw page number
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        context.set_font_size(12.0);
        context.move_to(72.0, 72.0); // 1 inch margins
        let _ = context.show_text(&format!("Page {}", page_num + 1));
    });

    // Connect to begin-print to set up pages
    print_operation.connect_begin_print(move |op, _print_context| {
        // Could adjust number of pages here based on print context size
        let _ = op;
    });

    // Run the print dialog
    let result = print_operation.run(
        gtk::PrintOperationAction::PrintDialog,
        Some(window),
    );

    match result {
        Ok(gtk::PrintOperationResult::Error) => {
            show_error(window, "Printing failed");
        }
        Ok(gtk::PrintOperationResult::Cancel) => {
            // User cancelled, do nothing
        }
        Ok(gtk::PrintOperationResult::Apply) => {
            // Print completed successfully
        }
        Ok(gtk::PrintOperationResult::InProgress) => {
            // Printing in progress
        }
        Err(e) => {
            show_error(window, &format!("Print error: {}", e));
        }
        _ => {}
    }
}

/// Print a specific page range
pub fn print_page_range(
    window: &adw::ApplicationWindow,
    state: &AppState,
    start_page: usize,
    end_page: usize,
) {
    let document = match &state.document {
        Some(doc) => doc,
        None => {
            show_error(window, "No document loaded");
            return;
        }
    };

    let total_pages = document.page_count();
    let start = start_page.min(total_pages.saturating_sub(1));
    let end = end_page.min(total_pages.saturating_sub(1));

    if start > end {
        show_error(window, "Invalid page range");
        return;
    }

    let print_operation = gtk::PrintOperation::new();
    print_operation.set_n_pages((end - start + 1) as i32);

    // Set print settings
    let settings = gtk::PrintSettings::new();
    settings.set_n_copies(1);
    print_operation.set_print_settings(Some(&settings));

    print_operation.connect_draw_page(move |_op, print_context, page_num| {
        let actual_page = start + page_num as usize;

        if let Some(context) = print_context.cairo_context() {
            // Render page
            context.set_source_rgb(0.0, 0.0, 0.0);
            context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            context.set_font_size(12.0);
            context.move_to(72.0, 72.0);
            let _ = context.show_text(&format!("Page {}", actual_page + 1));
        }
    });

    let _ = print_operation.run(
        gtk::PrintOperationAction::PrintDialog,
        Some(window),
    );
}

/// Print current page only
pub fn print_current_page(window: &adw::ApplicationWindow, state: &AppState) {
    let page = state.current_page;
    print_page_range(window, state, page, page);
}

/// Export document to PDF (for non-PDF formats)
pub fn export_to_pdf(
    window: &adw::ApplicationWindow,
    state: &AppState,
    output_path: &std::path::Path,
) -> Result<(), String> {
    let document = state.document.as_ref()
        .ok_or_else(|| "No document loaded".to_string())?;

    // Get first page size for PDF dimensions
    let (width, height) = document.page_size(0)
        .unwrap_or((612.0, 792.0)); // Letter size default

    // Create PDF surface
    let surface = cairo::PdfSurface::new(
        width,
        height,
        output_path,
    ).map_err(|e| format!("Failed to create PDF surface: {}", e))?;

    let context = cairo::Context::new(&surface)
        .map_err(|e| format!("Failed to create Cairo context: {}", e))?;

    // Render each page
    for page_num in 0..document.page_count() {
        // Get page size
        let (page_width, page_height) = document.page_size(page_num)
            .unwrap_or((width, height));

        // Set PDF page size
        surface.set_size(page_width, page_height)
            .map_err(|e| format!("Failed to set page size: {}", e))?;

        // Render page content
        if let Some(page_surface) = document.render_page(page_num, 1.0) {
            context.set_source_surface(&page_surface, 0.0, 0.0)
                .map_err(|e| format!("Failed to set source surface: {}", e))?;
            context.paint()
                .map_err(|e| format!("Failed to paint page: {}", e))?;
        }

        // Add new page (except for last page)
        if page_num < document.page_count() - 1 {
            context.show_page()
                .map_err(|e| format!("Failed to add new page: {}", e))?;
        }
    }

    // Finish the PDF
    surface.finish();

    Ok(())
}

/// Show print preview dialog
pub fn show_print_preview(window: &adw::ApplicationWindow, state: &AppState) {
    let document = match &state.document {
        Some(doc) => doc,
        None => {
            show_error(window, "No document loaded");
            return;
        }
    };

    let print_operation = gtk::PrintOperation::new();
    print_operation.set_n_pages(document.page_count() as i32);

    print_operation.connect_draw_page(move |_op, print_context, page_num| {
        if let Some(context) = print_context.cairo_context() {
            context.set_source_rgb(0.0, 0.0, 0.0);
            context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            context.set_font_size(12.0);
            context.move_to(72.0, 72.0);
            let _ = context.show_text(&format!("Page {}", page_num + 1));
        }
    });

    let _ = print_operation.run(
        gtk::PrintOperationAction::Preview,
        Some(window),
    );
}

/// Print settings dialog
pub struct PrintSettings {
    pub copies: i32,
    pub collate: bool,
    pub reverse: bool,
    pub pages: PrintPages,
    pub page_set: PageSet,
    pub scale: PrintScale,
}

#[derive(Clone, Copy)]
pub enum PrintPages {
    All,
    Current,
    Range(usize, usize),
    Selection,
}

#[derive(Clone, Copy)]
pub enum PageSet {
    All,
    Even,
    Odd,
}

#[derive(Clone, Copy)]
pub enum PrintScale {
    None,
    Shrink,
    Fit,
    Custom(f64),
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            copies: 1,
            collate: true,
            reverse: false,
            pages: PrintPages::All,
            page_set: PageSet::All,
            scale: PrintScale::Fit,
        }
    }
}

fn show_error(window: &adw::ApplicationWindow, message: &str) {
    let dialog = adw::MessageDialog::new(
        Some(window),
        Some("Print Error"),
        Some(message),
    );
    dialog.add_response("ok", "OK");
    dialog.present();
}
