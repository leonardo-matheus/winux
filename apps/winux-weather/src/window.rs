// Main window for Winux Weather

use gtk4::prelude::*;
use gtk4::{Application, Box, Button, Entry, Label, Orientation, ScrolledWindow, Spinner};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher, StatusPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::views::{CurrentWeatherView, HourlyView, DailyView, DetailsView};
use crate::api::{OpenMeteoClient, Location};
use crate::data::{WeatherData, SavedLocations};
use crate::ui::WeatherBackground;

pub fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    // Search entry for locations
    let search_entry = Entry::builder()
        .placeholder_text("Buscar cidade...")
        .width_chars(20)
        .build();
    search_entry.add_css_class("search");

    // Location button
    let location_btn = Button::from_icon_name("mark-location-symbolic");
    location_btn.set_tooltip_text(Some("Usar minha localizacao"));

    // Refresh button
    let refresh_btn = Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some("Atualizar"));

    header.pack_start(&search_entry);
    header.pack_end(&refresh_btn);
    header.pack_end(&location_btn);

    // Main container with dynamic background
    let main_overlay = gtk4::Overlay::new();

    // Background widget
    let background = WeatherBackground::new();
    main_overlay.set_child(Some(&background.widget()));

    // Content stack
    let stack = ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    // Current weather view
    let current_view = CurrentWeatherView::new();
    stack.add_titled(&current_view.widget(), Some("current"), "Agora")
        .set_icon_name(Some("weather-clear-symbolic"));

    // Hourly forecast view
    let hourly_view = HourlyView::new();
    stack.add_titled(&hourly_view.widget(), Some("hourly"), "Horas")
        .set_icon_name(Some("document-open-recent-symbolic"));

    // Daily forecast view
    let daily_view = DailyView::new();
    stack.add_titled(&daily_view.widget(), Some("daily"), "Dias")
        .set_icon_name(Some("x-office-calendar-symbolic"));

    // Details view
    let details_view = DetailsView::new();
    stack.add_titled(&details_view.widget(), Some("details"), "Detalhes")
        .set_icon_name(Some("view-list-symbolic"));

    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    // Loading indicator
    let loading_box = Box::new(Orientation::Vertical, 12);
    loading_box.set_valign(gtk4::Align::Center);
    loading_box.set_halign(gtk4::Align::Center);

    let spinner = Spinner::new();
    spinner.set_size_request(48, 48);

    let loading_label = Label::new(Some("Carregando clima..."));
    loading_label.add_css_class("dim-label");

    loading_box.append(&spinner);
    loading_box.append(&loading_label);

    // Content box
    let content_box = Box::new(Orientation::Vertical, 0);
    content_box.append(&stack);

    main_overlay.add_overlay(&content_box);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Clima")
        .default_width(450)
        .default_height(700)
        .content(&main_overlay)
        .build();

    window.set_titlebar(Some(&header));

    // Apply custom CSS
    apply_css();

    // Shared state
    let weather_data: Rc<RefCell<Option<WeatherData>>> = Rc::new(RefCell::new(None));
    let current_location: Rc<RefCell<Option<Location>>> = Rc::new(RefCell::new(None));
    let saved_locations = Rc::new(RefCell::new(SavedLocations::load()));

    // Clone views for callbacks
    let current_view = Rc::new(current_view);
    let hourly_view = Rc::new(hourly_view);
    let daily_view = Rc::new(daily_view);
    let details_view = Rc::new(details_view);
    let background = Rc::new(background);

    // Load default location or last used
    let default_location = {
        let locations = saved_locations.borrow();
        locations.get_default().cloned()
    };

    if let Some(location) = default_location {
        *current_location.borrow_mut() = Some(location.clone());
        load_weather_data(
            location,
            weather_data.clone(),
            current_view.clone(),
            hourly_view.clone(),
            daily_view.clone(),
            details_view.clone(),
            background.clone(),
            spinner.clone(),
        );
    } else {
        // Use Sao Paulo as default
        let default_loc = Location {
            name: "Sao Paulo".to_string(),
            country: "Brasil".to_string(),
            latitude: -23.5505,
            longitude: -46.6333,
        };
        *current_location.borrow_mut() = Some(default_loc.clone());
        load_weather_data(
            default_loc,
            weather_data.clone(),
            current_view.clone(),
            hourly_view.clone(),
            daily_view.clone(),
            details_view.clone(),
            background.clone(),
            spinner.clone(),
        );
    }

    // Search entry activation
    {
        let weather_data = weather_data.clone();
        let current_location = current_location.clone();
        let current_view = current_view.clone();
        let hourly_view = hourly_view.clone();
        let daily_view = daily_view.clone();
        let details_view = details_view.clone();
        let background = background.clone();
        let spinner = spinner.clone();
        let saved_locations = saved_locations.clone();

        search_entry.connect_activate(move |entry| {
            let query = entry.text().to_string();
            if query.is_empty() {
                return;
            }

            let weather_data = weather_data.clone();
            let current_location = current_location.clone();
            let current_view = current_view.clone();
            let hourly_view = hourly_view.clone();
            let daily_view = daily_view.clone();
            let details_view = details_view.clone();
            let background = background.clone();
            let spinner = spinner.clone();
            let saved_locations = saved_locations.clone();

            spinner.start();

            glib::spawn_future_local(async move {
                match OpenMeteoClient::search_location(&query).await {
                    Ok(locations) if !locations.is_empty() => {
                        let location = locations.into_iter().next().unwrap();

                        // Save location
                        {
                            let mut locs = saved_locations.borrow_mut();
                            locs.add(location.clone());
                            locs.save();
                        }

                        *current_location.borrow_mut() = Some(location.clone());

                        load_weather_data(
                            location,
                            weather_data,
                            current_view,
                            hourly_view,
                            daily_view,
                            details_view,
                            background,
                            spinner,
                        );
                    }
                    Ok(_) => {
                        spinner.stop();
                        tracing::warn!("No locations found for query: {}", query);
                    }
                    Err(e) => {
                        spinner.stop();
                        tracing::error!("Failed to search location: {}", e);
                    }
                }
            });
        });
    }

    // Refresh button
    {
        let weather_data = weather_data.clone();
        let current_location = current_location.clone();
        let current_view = current_view.clone();
        let hourly_view = hourly_view.clone();
        let daily_view = daily_view.clone();
        let details_view = details_view.clone();
        let background = background.clone();
        let spinner = spinner.clone();

        refresh_btn.connect_clicked(move |_| {
            if let Some(location) = current_location.borrow().clone() {
                load_weather_data(
                    location,
                    weather_data.clone(),
                    current_view.clone(),
                    hourly_view.clone(),
                    daily_view.clone(),
                    details_view.clone(),
                    background.clone(),
                    spinner.clone(),
                );
            }
        });
    }

    // Geolocation button
    {
        let weather_data = weather_data.clone();
        let current_location = current_location.clone();
        let current_view = current_view.clone();
        let hourly_view = hourly_view.clone();
        let daily_view = daily_view.clone();
        let details_view = details_view.clone();
        let background = background.clone();
        let spinner = spinner.clone();

        location_btn.connect_clicked(move |_| {
            let weather_data = weather_data.clone();
            let current_location = current_location.clone();
            let current_view = current_view.clone();
            let hourly_view = hourly_view.clone();
            let daily_view = daily_view.clone();
            let details_view = details_view.clone();
            let background = background.clone();
            let spinner = spinner.clone();

            spinner.start();

            glib::spawn_future_local(async move {
                match crate::api::get_current_location().await {
                    Ok(location) => {
                        *current_location.borrow_mut() = Some(location.clone());
                        load_weather_data(
                            location,
                            weather_data,
                            current_view,
                            hourly_view,
                            daily_view,
                            details_view,
                            background,
                            spinner,
                        );
                    }
                    Err(e) => {
                        spinner.stop();
                        tracing::error!("Failed to get location: {}", e);
                    }
                }
            });
        });
    }

    // Auto-refresh every 30 minutes
    {
        let weather_data = weather_data.clone();
        let current_location = current_location.clone();
        let current_view = current_view.clone();
        let hourly_view = hourly_view.clone();
        let daily_view = daily_view.clone();
        let details_view = details_view.clone();
        let background = background.clone();
        let spinner = spinner.clone();

        glib::timeout_add_seconds_local(1800, move || {
            if let Some(location) = current_location.borrow().clone() {
                load_weather_data(
                    location,
                    weather_data.clone(),
                    current_view.clone(),
                    hourly_view.clone(),
                    daily_view.clone(),
                    details_view.clone(),
                    background.clone(),
                    spinner.clone(),
                );
            }
            glib::ControlFlow::Continue
        });
    }

    window.present();
}

fn load_weather_data(
    location: Location,
    weather_data: Rc<RefCell<Option<WeatherData>>>,
    current_view: Rc<CurrentWeatherView>,
    hourly_view: Rc<HourlyView>,
    daily_view: Rc<DailyView>,
    details_view: Rc<DetailsView>,
    background: Rc<WeatherBackground>,
    spinner: Spinner,
) {
    spinner.start();

    glib::spawn_future_local(async move {
        match OpenMeteoClient::get_weather(location.latitude, location.longitude).await {
            Ok(data) => {
                // Update views
                current_view.update(&data, &location);
                hourly_view.update(&data);
                daily_view.update(&data);
                details_view.update(&data);
                background.update(&data);

                *weather_data.borrow_mut() = Some(data);
            }
            Err(e) => {
                tracing::error!("Failed to fetch weather: {}", e);
            }
        }
        spinner.stop();
    });
}

fn apply_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("../style.css"));

    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
