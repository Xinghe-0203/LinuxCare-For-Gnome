use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Box, Button, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, TextView,
};

use crate::tr;
use crate::utils::spawn_bg;

fn run_command(args: &[&str]) -> String {
    std::process::Command::new(args[0])
        .args(&args[1..])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if !stderr.is_empty() {
                format!("{}\n{}", stdout, stderr)
            } else {
                stdout
            }
        })
        .unwrap_or_else(|e| format!("Error: {}", e))
}

fn detect_pci_devices() -> String {
    let output = run_command(&["lspci", "-nn"]);
    if output.starts_with("Error") {
        "Unable to detect PCI devices. Is lspci installed?".to_string()
    } else {
        let mut devices = Vec::new();
        for line in output.lines() {
            if let Some(pos) = line.find(':') {
                devices.push(line[pos + 1..].trim().to_string());
            }
        }
        format!("PCI Devices ({}):\n{}", devices.len(), output)
    }
}

fn detect_usb_devices() -> String {
    let output = run_command(&["lsusb"]);
    if output.starts_with("Error") {
        "Unable to detect USB devices. Is lsusb installed?".to_string()
    } else {
        let count = output.lines().filter(|l| !l.trim().is_empty()).count();
        format!("USB Devices ({}):\n{}", count, output)
    }
}

fn get_loaded_modules() -> String {
    let output = run_command(&["lsmod"]);
    if output.starts_with("Error") {
        "Unable to list kernel modules. Permission denied?".to_string()
    } else {
        let count = output.lines().count().saturating_sub(1);
        format!("Loaded Kernel Modules ({}):\n{}", count, output)
    }
}

fn get_dmesg_errors() -> String {
    let output = run_command(&["dmesg"]);
    if output.starts_with("Error") {
        "Unable to read kernel messages. Permission denied?".to_string()
    } else {
        let errors: Vec<&str> = output
            .lines()
            .filter(|l| {
                l.contains("error")
                    || l.contains("Error")
                    || l.contains("fail")
                    || l.contains("Fail")
                    || l.contains("warning")
                    || l.contains("Warning")
            })
            .collect();
        if errors.is_empty() {
            "No errors found in kernel messages.".to_string()
        } else {
            format!(
                "Kernel Errors/Warnings ({}):\n{}",
                errors.len(),
                errors.join("\n")
            )
        }
    }
}

fn get_driver_recommendations() -> String {
    let mut recommendations = Vec::new();

    let output = run_command(&["lspci", "-nn"]);
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("vga") || lower.contains("3d") || lower.contains("display") {
            if lower.contains("nvidia") {
                recommendations.push("[GPU] NVIDIA detected: Install proprietary drivers\n       sudo apt install nvidia-driver-535");
            } else if lower.contains("amd") || lower.contains("radeon") {
                recommendations.push("[GPU] AMD/ATI detected: Mesa drivers recommended\n       sudo apt install mesa-vdpau-drivers");
            } else if lower.contains("intel") && lower.contains("graphics") {
                recommendations.push("[GPU] Intel HD/UHD detected: Mesa drivers recommended\n       sudo apt install intel-media-va-driver");
            }
        }
        if lower.contains("network") && lower.contains("wireless") {
            if lower.contains("realtek") {
                recommendations.push("[WiFi] Realtek WiFi detected: Check for dkms drivers\n       sudo apt install realtek-rtl88xxau-dkms");
            } else if lower.contains("intel") {
                recommendations.push("[WiFi] Intel WiFi detected: Firmware-iwlwifi may be needed\n       sudo apt install firmware-iwlwifi");
            }
        }
        if lower.contains("audio") || lower.contains("sound") {
            recommendations.push("[Audio] Sound device detected: Ensure PulseAudio/PipeWire is installed");
        }
    }

    if recommendations.is_empty() {
        "No specific driver recommendations found.\nAll detected hardware appears to have drivers loaded.".to_string()
    } else {
        format!("Driver Recommendations:\n\n{}", recommendations.join("\n\n"))
    }
}

pub fn build_driver_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("driver_management")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Hardware Detection ──

    let hw_frame = Frame::new(Some(tr!("hardware_detection")));
    hw_frame.set_css_classes(&["card"]);

    let hw_grid = Grid::new();
    hw_grid.set_column_spacing(12);
    hw_grid.set_row_spacing(10);
    hw_grid.set_margin_top(10);
    hw_grid.set_margin_bottom(10);
    hw_grid.set_margin_start(12);
    hw_grid.set_margin_end(12);

    let pci_button = Button::with_label(&tr!("pci_devices"));
    pci_button.set_halign(gtk::Align::Start);
    pci_button.set_css_classes(&["suggested-action"]);

    let usb_button = Button::with_label(&tr!("usb_devices"));
    usb_button.set_halign(gtk::Align::Start);
    usb_button.set_css_classes(&["suggested-action"]);

    let all_button = Button::with_label("Scan All Hardware");
    all_button.set_halign(gtk::Align::Start);

    hw_grid.attach(&pci_button, 0, 0, 1, 1);
    hw_grid.attach(&usb_button, 1, 0, 1, 1);
    hw_grid.attach(&all_button, 2, 0, 1, 1);

    hw_frame.set_child(Some(&hw_grid));
    main_box.append(&hw_frame);

    // ── Hardware Output ──

    let hw_output_frame = Frame::new(Some("Hardware Information"));
    hw_output_frame.set_css_classes(&["card"]);

    let hw_output_scrolled = ScrolledWindow::new();
    hw_output_scrolled.set_min_content_height(200);
    hw_output_scrolled.set_max_content_height(350);

    let hw_output_text = TextView::new();
    hw_output_text.set_editable(false);
    hw_output_text.set_monospace(true);
    hw_output_text.set_vexpand(true);
    hw_output_text.buffer().set_text("Click 'Scan' to detect hardware devices...");

    hw_output_scrolled.set_child(Some(&hw_output_text));
    hw_output_scrolled.set_margin_top(10);
    hw_output_scrolled.set_margin_bottom(10);
    hw_output_scrolled.set_margin_start(12);
    hw_output_scrolled.set_margin_end(12);

    hw_output_frame.set_child(Some(&hw_output_scrolled));
    main_box.append(&hw_output_frame);

    // ── Installed Drivers ──

    let driver_frame = Frame::new(Some(tr!("loaded_modules")));
    driver_frame.set_css_classes(&["card"]);

    let driver_grid = Grid::new();
    driver_grid.set_column_spacing(12);
    driver_grid.set_row_spacing(10);
    driver_grid.set_margin_top(10);
    driver_grid.set_margin_bottom(10);
    driver_grid.set_margin_start(12);
    driver_grid.set_margin_end(12);

    let refresh_driver_button = Button::with_label(&tr!("refresh"));
    refresh_driver_button.set_halign(gtk::Align::Start);
    refresh_driver_button.set_css_classes(&["suggested-action"]);

    let driver_scrolled = ScrolledWindow::new();
    driver_scrolled.set_min_content_height(150);
    driver_scrolled.set_max_content_height(300);

    let driver_text = TextView::new();
    driver_text.set_editable(false);
    driver_text.set_monospace(true);
    driver_text.set_vexpand(true);
    driver_text.buffer().set_text("Click 'Refresh Modules' to view loaded drivers...");

    driver_scrolled.set_child(Some(&driver_text));
    driver_scrolled.set_margin_top(10);
    driver_scrolled.set_margin_bottom(10);
    driver_scrolled.set_margin_start(12);
    driver_scrolled.set_margin_end(12);

    driver_grid.attach(&refresh_driver_button, 0, 0, 1, 1);
    driver_grid.attach(&driver_scrolled, 0, 1, 1, 1);

    driver_frame.set_child(Some(&driver_grid));
    main_box.append(&driver_frame);

    // ── Install/Update Drivers ──

    let install_frame = Frame::new(Some(tr!("install_driver")));
    install_frame.set_css_classes(&["card"]);

    let install_grid = Grid::new();
    install_grid.set_column_spacing(12);
    install_grid.set_row_spacing(10);
    install_grid.set_margin_top(10);
    install_grid.set_margin_bottom(10);
    install_grid.set_margin_start(12);
    install_grid.set_margin_end(12);

    let pkg_label = Label::new(Some("Driver Package:"));
    pkg_label.set_halign(gtk::Align::Start);
    let pkg_entry = Entry::new();
    pkg_entry.set_hexpand(true);
    pkg_entry.set_placeholder_text(Some("e.g., nvidia-driver-535, firmware-iwlwifi"));

    let install_button = Button::with_label("Install");
    install_button.set_halign(gtk::Align::Start);
    install_button.set_css_classes(&["suggested-action"]);

    let update_button = Button::with_label("Update All Packages");
    update_button.set_halign(gtk::Align::Start);

    let search_button = Button::with_label("Search Package");
    search_button.set_halign(gtk::Align::Start);

    install_grid.attach(&pkg_label, 0, 0, 1, 1);
    install_grid.attach(&pkg_entry, 1, 0, 3, 1);
    install_grid.attach(&install_button, 0, 1, 1, 1);
    install_grid.attach(&update_button, 1, 1, 1, 1);
    install_grid.attach(&search_button, 2, 1, 1, 1);

    install_frame.set_child(Some(&install_grid));
    main_box.append(&install_frame);

    // ── Driver Status & Recommendations ──

    let recommend_frame = Frame::new(Some("Driver Status & Recommendations"));
    recommend_frame.set_css_classes(&["card"]);

    let recommend_scrolled = ScrolledWindow::new();
    recommend_scrolled.set_min_content_height(150);
    recommend_scrolled.set_max_content_height(300);

    let recommend_text = TextView::new();
    recommend_text.set_editable(false);
    recommend_text.set_monospace(true);
    recommend_text.set_vexpand(true);
    recommend_text.buffer().set_text("Click 'Analyze Drivers' to get recommendations...");

    recommend_scrolled.set_child(Some(&recommend_text));
    recommend_scrolled.set_margin_top(10);
    recommend_scrolled.set_margin_bottom(10);
    recommend_scrolled.set_margin_start(12);
    recommend_scrolled.set_margin_end(12);

    let recommend_grid = Grid::new();
    recommend_grid.set_column_spacing(12);
    recommend_grid.set_row_spacing(10);
    recommend_grid.set_margin_top(10);
    recommend_grid.set_margin_bottom(10);
    recommend_grid.set_margin_start(12);
    recommend_grid.set_margin_end(12);

    let analyze_button = Button::with_label("Analyze Drivers");
    analyze_button.set_halign(gtk::Align::Start);
    analyze_button.set_css_classes(&["suggested-action"]);

    let dmesg_button = Button::with_label("Check Kernel Errors");
    dmesg_button.set_halign(gtk::Align::Start);

    recommend_grid.attach(&analyze_button, 0, 0, 1, 1);
    recommend_grid.attach(&dmesg_button, 1, 0, 1, 1);
    recommend_grid.attach(&recommend_scrolled, 0, 1, 2, 1);

    recommend_frame.set_child(Some(&recommend_grid));
    main_box.append(&recommend_frame);

    // ── Signal handlers ──

    {
        let hw_output_text = hw_output_text.clone();
        pci_button.connect_clicked(move |_| {
            let buffer = hw_output_text.buffer();
            buffer.set_text("Scanning PCI devices...");
            let hw_output_text_c = hw_output_text.clone();
            spawn_bg(
                move || detect_pci_devices(),
                move |output| {
                    hw_output_text_c.buffer().set_text(&output);
                },
            );
        });
    }

    {
        let hw_output_text = hw_output_text.clone();
        usb_button.connect_clicked(move |_| {
            let buffer = hw_output_text.buffer();
            buffer.set_text("Scanning USB devices...");
            let hw_output_text_c = hw_output_text.clone();
            spawn_bg(
                move || detect_usb_devices(),
                move |output| {
                    hw_output_text_c.buffer().set_text(&output);
                },
            );
        });
    }

    {
        let hw_output_text = hw_output_text.clone();
        all_button.connect_clicked(move |_| {
            let buffer = hw_output_text.buffer();
            buffer.set_text("Scanning all hardware...");
            let hw_output_text_c = hw_output_text.clone();
            spawn_bg(
                move || {
                    let pci = detect_pci_devices();
                    let usb = detect_usb_devices();
                    format!("{}\n\n{}", pci, usb)
                },
                move |output| {
                    hw_output_text_c.buffer().set_text(&output);
                },
            );
        });
    }

    {
        let driver_text = driver_text.clone();
        refresh_driver_button.connect_clicked(move |_| {
            let buffer = driver_text.buffer();
            buffer.set_text("Loading kernel modules...");
            let driver_text_cc = driver_text.clone();
            spawn_bg(
                move || get_loaded_modules(),
                move |output| {
                    driver_text_cc.buffer().set_text(&output);
                },
            );
        });
    }

    {
        let pkg_entry = pkg_entry.clone();
        let recommend_text = recommend_text.clone();
        install_button.connect_clicked(move |_| {
            let pkg = pkg_entry.text().to_string();
            if pkg.is_empty() {
                recommend_text.buffer().set_text("Error: Enter a package name to install.");
                return;
            }
            let buffer = recommend_text.buffer();
            buffer.set_text(&format!("Installing {}...", pkg));
            let recommend_text_c = recommend_text.clone();
            let pkg_c = pkg.clone();
            let pkg_cc = pkg_c.clone();
            spawn_bg(
                move || run_command(&["pkexec", "apt", "install", "-y", &pkg_c]),
                move |output| {
                    recommend_text_c.buffer().set_text(&format!(
                        "Install result for {}:\n\n{}", pkg_cc, output
                    ));
                },
            );
        });
    }

    {
        let recommend_text = recommend_text.clone();
        update_button.connect_clicked(move |_| {
            let buffer = recommend_text.buffer();
            buffer.set_text("Updating all packages...");
            let recommend_text_c = recommend_text.clone();
            spawn_bg(
                move || {
                    let update_output = run_command(&["pkexec", "apt", "update"]);
                    let upgrade_output = run_command(&["pkexec", "apt", "upgrade", "-y"]);
                    (update_output, upgrade_output)
                },
                move |(update_output, upgrade_output)| {
                    recommend_text_c.buffer().set_text(&format!(
                        "Update result:\n{}\n\nUpgrade result:\n{}",
                        update_output, upgrade_output
                    ));
                },
            );
        });
    }

    {
        let pkg_entry = pkg_entry.clone();
        let recommend_text = recommend_text.clone();
        search_button.connect_clicked(move |_| {
            let pkg = pkg_entry.text().to_string();
            if pkg.is_empty() {
                recommend_text.buffer().set_text("Error: Enter a package name to search.");
                return;
            }
            let buffer = recommend_text.buffer();
            buffer.set_text(&format!("Searching for '{}'...", pkg));
            let recommend_text_c = recommend_text.clone();
            let pkg_c = pkg.clone();
            let pkg_cc = pkg_c.clone();
            spawn_bg(
                move || run_command(&["apt", "search", &pkg_c]),
                move |output| {
                    recommend_text_c.buffer().set_text(&format!(
                        "Search results for '{}' :\n\n{}", pkg_cc, output
                    ));
                },
            );
        });
    }

    {
        let recommend_text = recommend_text.clone();
        analyze_button.connect_clicked(move |_| {
            let buffer = recommend_text.buffer();
            buffer.set_text("Analyzing hardware and drivers...");
            let recommend_text_c = recommend_text.clone();
            spawn_bg(
                move || get_driver_recommendations(),
                move |output| {
                    recommend_text_c.buffer().set_text(&output);
                },
            );
        });
    }

    {
        let recommend_text = recommend_text.clone();
        dmesg_button.connect_clicked(move |_| {
            let buffer = recommend_text.buffer();
            buffer.set_text("Checking kernel messages...");
            let recommend_text_c = recommend_text.clone();
            spawn_bg(
                move || get_dmesg_errors(),
                move |output| {
                    recommend_text_c.buffer().set_text(&output);
                },
            );
        });
    }

    // ── Periodic refresh of modules ──

    {
        let driver_text = driver_text.clone();
        glib::timeout_add_seconds_local(30, move || {
            let driver_text_c = driver_text.clone();
            spawn_bg(
                move || get_loaded_modules(),
                move |output| {
                    driver_text_c.buffer().set_text(&output);
                },
            );
            glib::ControlFlow::Continue
        });
    }

    main_box.upcast()
}
