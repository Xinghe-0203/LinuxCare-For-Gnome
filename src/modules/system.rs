use std::env;
use std::fs;

use gtk::glib;
use gtk::prelude::*;
use gtk::{Box, Button, Frame, Grid, Label, Orientation, SpinButton, Switch, Adjustment};

use crate::tr;

// ---------------------------------------------------------------------------
// GSettings helpers
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

// ---------------------------------------------------------------------------
// Simple command helpers
// ---------------------------------------------------------------------------

fn read_file_trimmed(path: &str) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn cmd_output(program: &str, args: &[&str]) -> String {
    std::process::Command::new(program)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Fractional scaling helpers (experimental-features is a string list)
// ---------------------------------------------------------------------------

const SCALE_FEATURE: &str = "scale-monitor-framebuffer";

fn fractional_scaling_enabled() -> bool {
    let raw = gsettings_get("org.gnome.mutter", "experimental-features");
    // raw looks like: ['scale-monitor-framebuffer'] or []
    raw.contains(SCALE_FEATURE)
}

fn set_fractional_scaling(enabled: bool) {
    let current = gsettings_get("org.gnome.mutter", "experimental-features");
    let has = current.contains(SCALE_FEATURE);
    if enabled && !has {
        let new_val = if current.trim() == "[]" || current.trim().is_empty() {
            format!("[ '{}']", SCALE_FEATURE)
        } else {
            // Append to existing list – strip trailing ']', add comma
            let trimmed = current.trim_end_matches(']').trim();
            format!("{}, '{}']", trimmed, SCALE_FEATURE)
        };
        gsettings_set("org.gnome.mutter", "experimental-features", &new_val);
    } else if !enabled && has {
        // Remove the feature from the list
        let new_val = current
            .replace(&format!("'{}'", SCALE_FEATURE), "")
            .replace(&format!("'{}' ", SCALE_FEATURE), "")
            .replace(&format!(" '{}'", SCALE_FEATURE), "")
            .replace(", ,", ",")
            .replace("[,", "[")
            .replace(", ]", "]")
            .replace("[ ]", "[]");
        // Clean up edge cases
        let new_val = if new_val.contains(",]") {
            new_val.replace(",]", "]")
        } else if new_val.contains("[ ,") {
            new_val.replace("[ ,", "[")
        } else {
            new_val
        };
        let new_val = if new_val.trim() == "[" { "[]".to_string() } else { new_val };
        gsettings_set("org.gnome.mutter", "experimental-features", &new_val);
    }
}

// ---------------------------------------------------------------------------
// Build the system page
// ---------------------------------------------------------------------------

pub fn build_system_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("system")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ===================================================================
    // System Info section
    // ===================================================================
    let info_frame = Frame::new(Some(tr!("system_info")));
    info_frame.set_css_classes(&["card"]);

    let info_grid = Grid::new();
    info_grid.set_column_spacing(20);
    info_grid.set_row_spacing(10);
    info_grid.set_margin_top(10);
    info_grid.set_margin_bottom(10);
    info_grid.set_margin_start(10);
    info_grid.set_margin_end(10);

    let hostname = read_file_trimmed("/proc/sys/kernel/hostname");
    let kernel = cmd_output("uname", &["-r"]);
    let arch = cmd_output("uname", &["-m"]);
    let de = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "N/A".to_string());
    let uptime = cmd_output("uptime", &["-p"]);

    let info_rows: [(&str, &str); 5] = [
        (tr!("hostname"), &hostname),
        (tr!("kernel"), &kernel),
        (tr!("architecture"), &arch),
        (tr!("desktop_environment"), &de),
        (tr!("uptime"), &uptime),
    ];

    for (row, (label_text, value_text)) in info_rows.iter().enumerate() {
        let lbl = Label::new(Some(label_text));
        lbl.set_halign(gtk::Align::Start);
        lbl.set_css_classes(&["dim-label"]);
        let val = Label::new(Some(value_text));
        val.set_halign(gtk::Align::Start);
        val.set_selectable(true);
        info_grid.attach(&lbl, 0, row as i32, 1, 1);
        info_grid.attach(&val, 1, row as i32, 1, 1);
    }

    info_frame.set_child(Some(&info_grid));
    main_box.append(&info_frame);

    // ===================================================================
    // Power Management section
    // ===================================================================
    let power_frame = Frame::new(Some(tr!("power_management")));
    power_frame.set_css_classes(&["card"]);

    let power_grid = Grid::new();
    power_grid.set_column_spacing(20);
    power_grid.set_row_spacing(10);
    power_grid.set_margin_top(10);
    power_grid.set_margin_bottom(10);
    power_grid.set_margin_start(10);
    power_grid.set_margin_end(10);

    // --- Auto suspend ---
    let suspend_label = Label::new(Some(tr!("auto_suspend")));
    suspend_label.set_halign(gtk::Align::Start);
    let suspend_switch = Switch::new();
    suspend_switch.set_halign(gtk::Align::Start);
    // Read current value: 'suspend' means enabled, anything else means disabled
    {
        let val = gsettings_get(
            "org.gnome.settings-daemon.plugins.power",
            "sleep-inactive-ac-type",
        );
        suspend_switch.set_active(val == "'suspend'");
    }
    {
        let sw = suspend_switch.clone();
        sw.connect_state_set(move |_switch, active| {
            let new_val = if active { "suspend" } else { "nothing" };
            gsettings_set(
                "org.gnome.settings-daemon.plugins.power",
                "sleep-inactive-ac-type",
                new_val,
            );
            glib::Propagation::Proceed
        });
    }
    power_grid.attach(&suspend_label, 0, 0, 1, 1);
    power_grid.attach(&suspend_switch, 1, 0, 1, 1);

    // --- Suspend timeout (minutes -> seconds for gsettings) ---
    let timeout_label = Label::new(Some(tr!("suspend_after")));
    timeout_label.set_halign(gtk::Align::Start);
    // gsettings stores seconds; UI shows minutes
    let timeout_secs: f64 = {
        let raw = gsettings_get(
            "org.gnome.settings-daemon.plugins.power",
            "sleep-inactive-ac-timeout",
        );
        raw.parse::<i32>().unwrap_or(1800).max(0) as f64
    };
    let timeout_mins = timeout_secs / 60.0;
    let timeout_adj = Adjustment::new(timeout_mins, 1.0, 360.0, 1.0, 5.0, 0.0);
    let timeout_spin = SpinButton::new(Some(&timeout_adj), 1.0, 0);
    timeout_spin.set_halign(gtk::Align::Start);
    {
        let adj = timeout_adj.clone();
        adj.connect_value_changed(move |a| {
            let minutes = a.value() as i32;
            let seconds = (minutes * 60).to_string();
            gsettings_set(
                "org.gnome.settings-daemon.plugins.power",
                "sleep-inactive-ac-timeout",
                &seconds,
            );
        });
    }
    power_grid.attach(&timeout_label, 0, 1, 1, 1);
    power_grid.attach(&timeout_spin, 1, 1, 1, 1);

    // --- Show battery percentage ---
    let battery_label = Label::new(Some(tr!("show_battery")));
    battery_label.set_halign(gtk::Align::Start);
    let battery_switch = Switch::new();
    battery_switch.set_halign(gtk::Align::Start);
    {
        let val = gsettings_get("org.gnome.desktop.interface", "show-battery-percentage");
        battery_switch.set_active(val == "true");
    }
    {
        let sw = battery_switch.clone();
        sw.connect_state_set(move |_switch, active| {
            gsettings_set(
                "org.gnome.desktop.interface",
                "show-battery-percentage",
                if active { "true" } else { "false" },
            );
            glib::Propagation::Proceed
        });
    }
    power_grid.attach(&battery_label, 0, 2, 1, 1);
    power_grid.attach(&battery_switch, 1, 2, 1, 1);

    power_frame.set_child(Some(&power_grid));
    main_box.append(&power_frame);

    // ===================================================================
    // Display section
    // ===================================================================
    let display_frame = Frame::new(Some(tr!("resolution")));
    display_frame.set_css_classes(&["card"]);

    let display_grid = Grid::new();
    display_grid.set_column_spacing(20);
    display_grid.set_row_spacing(10);
    display_grid.set_margin_top(10);
    display_grid.set_margin_bottom(10);
    display_grid.set_margin_start(10);
    display_grid.set_margin_end(10);

    // --- Night light ---
    let nightlight_label = Label::new(Some(tr!("night_light")));
    nightlight_label.set_halign(gtk::Align::Start);
    let nightlight_switch = Switch::new();
    nightlight_switch.set_halign(gtk::Align::Start);
    {
        let val = gsettings_get(
            "org.gnome.settings-daemon.plugins.color",
            "night-light-enabled",
        );
        nightlight_switch.set_active(val == "true");
    }
    {
        let sw = nightlight_switch.clone();
        sw.connect_state_set(move |_switch, active| {
            gsettings_set(
                "org.gnome.settings-daemon.plugins.color",
                "night-light-enabled",
                if active { "true" } else { "false" },
            );
            glib::Propagation::Proceed
        });
    }
    display_grid.attach(&nightlight_label, 0, 0, 1, 1);
    display_grid.attach(&nightlight_switch, 1, 0, 1, 1);

    // --- Fractional scaling ---
    let scaling_label = Label::new(Some(tr!("fractional_scaling")));
    scaling_label.set_halign(gtk::Align::Start);
    let scaling_switch = Switch::new();
    scaling_switch.set_halign(gtk::Align::Start);
    scaling_switch.set_active(fractional_scaling_enabled());
    {
        let sw = scaling_switch.clone();
        sw.connect_state_set(move |_switch, active| {
            set_fractional_scaling(active);
            glib::Propagation::Proceed
        });
    }
    display_grid.attach(&scaling_label, 0, 1, 1, 1);
    display_grid.attach(&scaling_switch, 1, 1, 1, 1);

    display_frame.set_child(Some(&display_grid));
    main_box.append(&display_frame);

    // ===================================================================
    // Sound section
    // ===================================================================
    let sound_frame = Frame::new(Some(tr!("system_sounds")));
    sound_frame.set_css_classes(&["card"]);

    let sound_grid = Grid::new();
    sound_grid.set_column_spacing(20);
    sound_grid.set_row_spacing(10);
    sound_grid.set_margin_top(10);
    sound_grid.set_margin_bottom(10);
    sound_grid.set_margin_start(10);
    sound_grid.set_margin_end(10);

    // --- System sounds ---
    let sounds_label = Label::new(Some(tr!("system_sounds")));
    sounds_label.set_halign(gtk::Align::Start);
    let sounds_switch = Switch::new();
    sounds_switch.set_halign(gtk::Align::Start);
    {
        let val = gsettings_get("org.gnome.desktop.sound", "event-sounds");
        sounds_switch.set_active(val == "true");
    }
    {
        let sw = sounds_switch.clone();
        sw.connect_state_set(move |_switch, active| {
            gsettings_set(
                "org.gnome.desktop.sound",
                "event-sounds",
                if active { "true" } else { "false" },
            );
            glib::Propagation::Proceed
        });
    }
    sound_grid.attach(&sounds_label, 0, 0, 1, 1);
    sound_grid.attach(&sounds_switch, 1, 0, 1, 1);

    // --- Alert sound ---
    let alert_label = Label::new(Some(tr!("alert_sound")));
    alert_label.set_halign(gtk::Align::Start);
    let alert_switch = Switch::new();
    alert_switch.set_halign(gtk::Align::Start);
    {
        let val = gsettings_get("org.gnome.desktop.sound", "input-feedback-sounds");
        alert_switch.set_active(val == "true");
    }
    {
        let sw = alert_switch.clone();
        sw.connect_state_set(move |_switch, active| {
            gsettings_set(
                "org.gnome.desktop.sound",
                "input-feedback-sounds",
                if active { "true" } else { "false" },
            );
            glib::Propagation::Proceed
        });
    }
    sound_grid.attach(&alert_label, 0, 1, 1, 1);
    sound_grid.attach(&alert_switch, 1, 1, 1, 1);

    sound_frame.set_child(Some(&sound_grid));
    main_box.append(&sound_frame);

    // ===================================================================
    // Open GNOME Settings button
    // ===================================================================
    let open_settings_button = Button::with_label(&tr!("open_gnome_settings"));
    open_settings_button.set_css_classes(&["suggested-action"]);
    open_settings_button.set_halign(gtk::Align::Center);
    open_settings_button.set_margin_top(20);
    open_settings_button.connect_clicked(|_| {
        let _ = std::process::Command::new("gnome-control-center").spawn();
    });
    main_box.append(&open_settings_button);

    main_box.upcast()
}
