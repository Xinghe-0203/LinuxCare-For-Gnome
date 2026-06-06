use gtk::prelude::*;
use gtk::{Box, Button, ComboBoxText, Frame, Grid, Label, Orientation, Scale, Switch, Adjustment};

use crate::tr;

// ---------------------------------------------------------------------------
// gsettings helpers
// ---------------------------------------------------------------------------

fn gsettings_get(schema: &str, key: &str) -> String {
    std::process::Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn gsettings_set(schema: &str, key: &str, value: &str) {
    let _ = std::process::Command::new("gsettings")
        .args(["set", schema, key, value])
        .output();
}

/// Return true when the value returned by `gsettings get` is `'true'`.
fn gsettings_bool(schema: &str, key: &str) -> bool {
    gsettings_get(schema, key) == "true"
}

/// Strip surrounding single-quotes that gsettings adds around string values.
fn strip_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

// ---------------------------------------------------------------------------
// DE detection helpers
// ---------------------------------------------------------------------------

/// Detect the current desktop environment (GNOME, Cinnamon, MATE …).
fn detect_de() -> String {
    // Prefer DESKTOP_SESSION / XDG_CURRENT_DESKTOP, fall back to gnome.
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .map(|v| v.to_lowercase())
        .unwrap_or_else(|_| "gnome".to_string())
}

/// Determine the schema that owns the `show-desktop-icons` key.
/// GNOME uses `org.gnome.desktop.background`, Nemo/Cinnamon uses `org.nemo.desktop`.
fn icons_schema() -> &'static str {
    let de = detect_de();
    if de.contains("cinnamon") || de.contains("nemo") {
        "org.nemo.desktop"
    } else {
        "org.gnome.desktop.background"
    }
}

/// List installed GTK themes via `gsettings list-schemas` is not the right
/// approach; instead we query `lsettings` or read
/// `/usr/share/themes`.  For simplicity we query the GSettings enum if
/// available, otherwise fall back to a curated list.
fn list_gtk_themes() -> Vec<String> {
    // Try to read installed themes from the standard directory.
    let mut themes = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/usr/share/themes") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Only keep directories that contain an "gtk-4.0" or "gtk-3.0" sub-folder.
            if entry.path().join("gtk-4.0").is_dir() || entry.path().join("gtk-3.0").is_dir() {
                themes.push(name);
            }
        }
    }
    // Also include user themes.
    if let Some(home) = dirs::home_dir() {
        let user_themes = home.join(".themes");
        if let Ok(entries) = std::fs::read_dir(user_themes) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().join("gtk-4.0").is_dir() || entry.path().join("gtk-3.0").is_dir() {
                    if !themes.contains(&name) {
                        themes.push(name);
                    }
                }
            }
        }
    }
    themes.sort();
    themes
}

fn list_icon_themes() -> Vec<String> {
    let mut themes = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/usr/share/icons") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.path().join("index.theme").is_dir() || entry.path().join("index.theme").is_file() {
                themes.push(name);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_icons = home.join(".icons");
        if let Ok(entries) = std::fs::read_dir(user_icons) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().join("index.theme").is_dir() || entry.path().join("index.theme").is_file() {
                    if !themes.contains(&name) {
                        themes.push(name);
                    }
                }
            }
        }
    }
    themes.sort();
    themes
}

fn list_cursor_themes() -> Vec<String> {
    let mut themes = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/usr/share/icons") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            // A cursor theme directory contains a "cursors" sub-directory.
            if path.join("cursors").is_dir() {
                themes.push(name);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let user_icons = home.join(".icons");
        if let Ok(entries) = std::fs::read_dir(user_icons) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().join("cursors").is_dir() {
                    if !themes.contains(&name) {
                        themes.push(name);
                    }
                }
            }
        }
    }
    themes.sort();
    themes
}

// ---------------------------------------------------------------------------
// Public page builder
// ---------------------------------------------------------------------------

pub fn build_desktop_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("desktop")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // =======================================================================
    // 1. Desktop Icons section
    // =======================================================================

    let icons_frame = Frame::new(Some(tr!("icons")));
    icons_frame.set_css_classes(&["card"]);

    let icons_grid = Grid::new();
    icons_grid.set_column_spacing(20);
    icons_grid.set_row_spacing(10);
    icons_grid.set_margin_top(10);
    icons_grid.set_margin_bottom(10);
    icons_grid.set_margin_start(10);
    icons_grid.set_margin_end(10);

    // --- Show desktop icons switch ---
    let schema_icons = icons_schema();
    let show_icons_initial = gsettings_bool(schema_icons, "show-desktop-icons");

    let show_icons_label = Label::new(Some(tr!("show_desktop_icons")));
    show_icons_label.set_halign(gtk::Align::Start);
    let show_icons_switch = Switch::new();
    show_icons_switch.set_active(show_icons_initial);
    show_icons_switch.set_halign(gtk::Align::Start);
    icons_grid.attach(&show_icons_label, 0, 0, 1, 1);
    icons_grid.attach(&show_icons_switch, 1, 0, 1, 1);

    {
        let schema = schema_icons.to_string();
        show_icons_switch.connect_active_notify(move |switch| {
            let val = if switch.is_active() { "true" } else { "false" };
            gsettings_set(&schema, "show-desktop-icons", val);
        });
    }

    // --- Icon size (zoom level) ---
    let icon_size_label = Label::new(Some(tr!("icon_size")));
    icon_size_label.set_halign(gtk::Align::Start);

    // Nautilus / Nemo icon zoom level as a combo box with common sizes.
    let icon_size_combo = ComboBoxText::new();
    let icon_zoom_levels = ["small", "standard", "large", "extra-large"];
    for lvl in &icon_zoom_levels {
        icon_size_combo.append_text(lvl);
    }
    let icon_size_schema = if detect_de().contains("cinnamon") {
        "org.nemo.icon-view"
    } else {
        "org.gnome.nautilus.icon-view"
    };
    let current_zoom = strip_quotes(&gsettings_get(icon_size_schema, "default-zoom-level"));
    let initial_idx = icon_zoom_levels.iter().position(|&z| z == current_zoom).unwrap_or(1);
    icon_size_combo.set_active(Some(initial_idx as u32));

    icons_grid.attach(&icon_size_label, 0, 1, 1, 1);
    icons_grid.attach(&icon_size_combo, 1, 1, 1, 1);

    icon_size_combo.connect_changed(move |combo| {
        if let Some(text) = combo.active_text() {
            gsettings_set(icon_size_schema, "default-zoom-level", &text);
        }
    });

    icons_frame.set_child(Some(&icons_grid));
    main_box.append(&icons_frame);

    // =======================================================================
    // 2. Wallpaper section
    // =======================================================================

    let wallpaper_frame = Frame::new(Some(tr!("wallpaper")));
    wallpaper_frame.set_css_classes(&["card"]);

    let wallpaper_grid = Grid::new();
    wallpaper_grid.set_column_spacing(20);
    wallpaper_grid.set_row_spacing(10);
    wallpaper_grid.set_margin_top(10);
    wallpaper_grid.set_margin_bottom(10);
    wallpaper_grid.set_margin_start(10);
    wallpaper_grid.set_margin_end(10);

    // --- Current wallpaper filename ---
    let wp_uri_raw = gsettings_get("org.gnome.desktop.background", "picture-uri");
    let wp_path = strip_quotes(&wp_uri_raw)
        .trim_start_matches("file://")
        .to_string();
    let wp_display = if wp_path.is_empty() {
        tr!("no_data").to_string()
    } else {
        wp_path.clone()
    };

    let wp_current_label = Label::new(Some(&format!("{}: {}", tr!("wallpaper"), wp_display)));
    wp_current_label.set_halign(gtk::Align::Start);
    wp_current_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    wp_current_label.set_max_width_chars(60);
    wp_current_label.set_selectable(true);
    wallpaper_grid.attach(&wp_current_label, 0, 0, 2, 1);

    // --- Wallpaper mode combo ---
    let wp_mode_label = Label::new(Some(tr!("wallpaper_mode")));
    wp_mode_label.set_halign(gtk::Align::Start);

    let wp_mode_combo = ComboBoxText::new();
    // The gsettings key uses these exact option strings.
    let wp_options = [
        ("none", "None"),
        ("wall-clock", "Wall Clock"),
        ("centered", "Centered"),
        ("scaled", "Scaled"),
        ("stretched", "Stretched"),
        ("zoom", "Zoom"),
        ("spanned", "Spanned"),
    ];
    for (_, display) in &wp_options {
        wp_mode_combo.append_text(display);
    }

    let current_wp_mode = strip_quotes(&gsettings_get(
        "org.gnome.desktop.background",
        "picture-options",
    ));
    let initial_wp_idx = wp_options
        .iter()
        .position(|(val, _)| *val == current_wp_mode)
        .unwrap_or(4); // default to "stretched" if unknown
    wp_mode_combo.set_active(Some(initial_wp_idx as u32));
    wp_mode_combo.set_halign(gtk::Align::Start);

    wallpaper_grid.attach(&wp_mode_label, 0, 1, 1, 1);
    wallpaper_grid.attach(&wp_mode_combo, 1, 1, 1, 1);

    wp_mode_combo.connect_changed(move |combo| {
        if let Some(idx) = combo.active() {
            if let Some((val, _)) = wp_options.get(idx as usize) {
                gsettings_set("org.gnome.desktop.background", "picture-options", val);
            }
        }
    });

    // --- Change wallpaper button (opens gnome-control-center) ---
    let change_wallpaper_button = Button::with_label(&tr!("change_wallpaper"));
    change_wallpaper_button.set_halign(gtk::Align::Start);
    change_wallpaper_button.connect_clicked(|_| {
        let _ = std::process::Command::new("gnome-control-center")
            .arg("background")
            .spawn();
    });
    wallpaper_grid.attach(&change_wallpaper_button, 0, 2, 2, 1);

    wallpaper_frame.set_child(Some(&wallpaper_grid));
    main_box.append(&wallpaper_frame);

    // =======================================================================
    // 3. Workspaces section
    // =======================================================================

    let workspaces_frame = Frame::new(Some("Workspaces"));
    workspaces_frame.set_css_classes(&["card"]);

    let workspaces_grid = Grid::new();
    workspaces_grid.set_column_spacing(20);
    workspaces_grid.set_row_spacing(10);
    workspaces_grid.set_margin_top(10);
    workspaces_grid.set_margin_bottom(10);
    workspaces_grid.set_margin_start(10);
    workspaces_grid.set_margin_end(10);

    // --- Dynamic workspaces switch ---
    let dynamic_initial = gsettings_bool("org.gnome.mutter", "dynamic-workspaces");

    let dynamic_label = Label::new(Some(tr!("dynamic_workspaces")));
    dynamic_label.set_halign(gtk::Align::Start);
    let dynamic_switch = Switch::new();
    dynamic_switch.set_active(dynamic_initial);
    dynamic_switch.set_halign(gtk::Align::Start);
    workspaces_grid.attach(&dynamic_label, 0, 0, 1, 1);
    workspaces_grid.attach(&dynamic_switch, 1, 0, 1, 1);

    // --- Number of workspaces (only active when dynamic is OFF) ---
    let num_ws_str = strip_quotes(&gsettings_get(
        "org.gnome.desktop.wm.preferences",
        "num-workspaces",
    ));
    let num_ws_val: f64 = num_ws_str.parse().unwrap_or(4.0);

    let num_label = Label::new(Some(tr!("num_workspaces")));
    num_label.set_halign(gtk::Align::Start);
    let num_adj = Adjustment::new(num_ws_val, 1.0, 32.0, 1.0, 1.0, 0.0);
    let num_scale = Scale::new(gtk::Orientation::Horizontal, Some(&num_adj));
    num_scale.set_hexpand(true);
    num_scale.set_sensitive(!dynamic_initial);
    workspaces_grid.attach(&num_label, 0, 1, 1, 1);
    workspaces_grid.attach(&num_scale, 1, 1, 1, 1);

    // Toggle sensitivity of num_scale when dynamic switch changes.
    {
        let num_scale_weak = num_scale.downgrade();
        dynamic_switch.connect_active_notify(move |switch| {
            let is_dynamic = switch.is_active();
            gsettings_set("org.gnome.mutter", "dynamic-workspaces", if is_dynamic { "true" } else { "false" });
            if let Some(s) = num_scale_weak.upgrade() {
                s.set_sensitive(!is_dynamic);
            }
        });
    }

    // Write num-workspaces when the scale changes.
    num_scale.connect_value_changed(move |scale| {
        let val = scale.value() as u32;
        gsettings_set(
            "org.gnome.desktop.wm.preferences",
            "num-workspaces",
            &val.to_string(),
        );
    });

    workspaces_frame.set_child(Some(&workspaces_grid));
    main_box.append(&workspaces_frame);

    // =======================================================================
    // 4. Theme / Appearance section
    // =======================================================================

    let theme_frame = Frame::new(Some(tr!("appearance")));
    theme_frame.set_css_classes(&["card"]);

    let theme_grid = Grid::new();
    theme_grid.set_column_spacing(20);
    theme_grid.set_row_spacing(10);
    theme_grid.set_margin_top(10);
    theme_grid.set_margin_bottom(10);
    theme_grid.set_margin_start(10);
    theme_grid.set_margin_end(10);

    // --- GTK Theme ---
    let gtk_theme_label = Label::new(Some(&format!("{}:", tr!("theme"))));
    gtk_theme_label.set_halign(gtk::Align::Start);
    let gtk_theme_combo = ComboBoxText::new();
    let gtk_themes = list_gtk_themes();
    for t in &gtk_themes {
        gtk_theme_combo.append_text(t);
    }
    let current_gtk_theme = strip_quotes(&gsettings_get(
        "org.gnome.desktop.interface",
        "gtk-theme",
    ));
    let initial_gtk_idx = gtk_themes.iter().position(|t| t == &current_gtk_theme);
    if let Some(idx) = initial_gtk_idx {
        gtk_theme_combo.set_active(Some(idx as u32));
    }
    gtk_theme_combo.set_hexpand(true);
    theme_grid.attach(&gtk_theme_label, 0, 0, 1, 1);
    theme_grid.attach(&gtk_theme_combo, 1, 0, 1, 1);

    gtk_theme_combo.connect_changed(move |combo| {
        if let Some(text) = combo.active_text() {
            gsettings_set("org.gnome.desktop.interface", "gtk-theme", &text);
        }
    });

    // --- Icon Theme ---
    let icon_theme_label = Label::new(Some(&format!("{}:", tr!("icons"))));
    icon_theme_label.set_halign(gtk::Align::Start);
    let icon_theme_combo = ComboBoxText::new();
    let icon_themes = list_icon_themes();
    for t in &icon_themes {
        icon_theme_combo.append_text(t);
    }
    let current_icon_theme = strip_quotes(&gsettings_get(
        "org.gnome.desktop.interface",
        "icon-theme",
    ));
    let initial_icon_idx = icon_themes.iter().position(|t| t == &current_icon_theme);
    if let Some(idx) = initial_icon_idx {
        icon_theme_combo.set_active(Some(idx as u32));
    }
    icon_theme_combo.set_hexpand(true);
    theme_grid.attach(&icon_theme_label, 0, 1, 1, 1);
    theme_grid.attach(&icon_theme_combo, 1, 1, 1, 1);

    icon_theme_combo.connect_changed(move |combo| {
        if let Some(text) = combo.active_text() {
            gsettings_set("org.gnome.desktop.interface", "icon-theme", &text);
        }
    });

    // --- Cursor Theme ---
    let cursor_theme_label = Label::new(Some("Cursor Theme:"));
    cursor_theme_label.set_halign(gtk::Align::Start);
    let cursor_theme_combo = ComboBoxText::new();
    let cursor_themes = list_cursor_themes();
    for t in &cursor_themes {
        cursor_theme_combo.append_text(t);
    }
    let current_cursor_theme = strip_quotes(&gsettings_get(
        "org.gnome.desktop.interface",
        "cursor-theme",
    ));
    let initial_cursor_idx = cursor_themes.iter().position(|t| t == &current_cursor_theme);
    if let Some(idx) = initial_cursor_idx {
        cursor_theme_combo.set_active(Some(idx as u32));
    }
    cursor_theme_combo.set_hexpand(true);
    theme_grid.attach(&cursor_theme_label, 0, 2, 1, 1);
    theme_grid.attach(&cursor_theme_combo, 1, 2, 1, 1);

    cursor_theme_combo.connect_changed(move |combo| {
        if let Some(text) = combo.active_text() {
            gsettings_set("org.gnome.desktop.interface", "cursor-theme", &text);
        }
    });

    // --- Color scheme (light / dark) ---
    let color_scheme_label = Label::new(Some("Color Scheme:"));
    color_scheme_label.set_halign(gtk::Align::Start);
    let color_scheme_combo = ComboBoxText::new();
    color_scheme_combo.append_text("default");
    color_scheme_combo.append_text("prefer-dark");
    color_scheme_combo.append_text("prefer-light");

    let current_color_scheme = strip_quotes(&gsettings_get(
        "org.gnome.desktop.interface",
        "color-scheme",
    ));
    let color_options = [
        "default",
        "prefer-dark",
        "prefer-light",
    ];
    let initial_cs_idx = color_options.iter().position(|&c| c == current_color_scheme);
    if let Some(idx) = initial_cs_idx {
        color_scheme_combo.set_active(Some(idx as u32));
    } else {
        color_scheme_combo.set_active(Some(0)); // default
    }
    color_scheme_combo.set_halign(gtk::Align::Start);
    theme_grid.attach(&color_scheme_label, 0, 3, 1, 1);
    theme_grid.attach(&color_scheme_combo, 1, 3, 1, 1);

    color_scheme_combo.connect_changed(move |combo| {
        if let Some(text) = combo.active_text() {
            gsettings_set("org.gnome.desktop.interface", "color-scheme", &text);
        }
    });

    theme_frame.set_child(Some(&theme_grid));
    main_box.append(&theme_frame);

    main_box.upcast()
}
