use gtk::glib;
use gtk::prelude::*;
use gtk::{Box, Button, CheckButton, Frame, Grid, Label, Orientation, ScrolledWindow, TextView};

use crate::tr;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

fn run_cmd(cmd: &str) -> String {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .ok()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            if o.status.success() {
                stdout
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                format!("{}\n{}", stdout, stderr)
            }
        })
        .unwrap_or_default()
        .trim()
        .to_string()
}

struct OptimItem {
    name: String,
    description: String,
    current_value: String,
    recommended_value: String,
    category: String,
    action: String,
}

fn get_sysctl_value(key: &str) -> String {
    run_cmd(&format!("sysctl -n {} 2>/dev/null", key))
}

fn scan_kernel_tuning() -> Vec<OptimItem> {
    let mut items = Vec::new();

    // vm.swappiness
    let current = get_sysctl_value("vm.swappiness");
    let recommended = "10".to_string();
    if current != recommended && !current.is_empty() {
        items.push(OptimItem {
            name: "vm.swappiness".to_string(),
            description: "Lower swappiness reduces swap usage for better desktop responsiveness".to_string(),
            current_value: current,
            recommended_value: recommended.clone(),
            category: "Kernel".to_string(),
            action: format!("sysctl -w vm.swappiness={}", recommended),
        });
    }

    // vm.dirty_ratio
    let current = get_sysctl_value("vm.dirty_ratio");
    let recommended = "15".to_string();
    if current != recommended && !current.is_empty() {
        items.push(OptimItem {
            name: "vm.dirty_ratio".to_string(),
            description: "Controls max percentage of memory for dirty pages before writeback".to_string(),
            current_value: current,
            recommended_value: recommended.clone(),
            category: "Kernel".to_string(),
            action: format!("sysctl -w vm.dirty_ratio={}", recommended),
        });
    }

    // net.core.somaxconn
    let current = get_sysctl_value("net.core.somaxconn");
    let recommended = "65535".to_string();
    if current != recommended && !current.is_empty() {
        items.push(OptimItem {
            name: "net.core.somaxconn".to_string(),
            description: "Maximum socket listen backlog for high-traffic servers".to_string(),
            current_value: current,
            recommended_value: recommended.clone(),
            category: "Kernel".to_string(),
            action: format!("sysctl -w net.core.somaxconn={}", recommended),
        });
    }

    // fs.inotify.max_user_watches
    let current = get_sysctl_value("fs.inotify.max_user_watches");
    let recommended = "524288".to_string();
    if current != recommended && !current.is_empty() {
        items.push(OptimItem {
            name: "fs.inotify.max_user_watches".to_string(),
            description: "Increase inotify watches for large projects (VSCode, file managers)".to_string(),
            current_value: current,
            recommended_value: recommended.clone(),
            category: "Kernel".to_string(),
            action: format!("sysctl -w fs.inotify.max_user_watches={}", recommended),
        });
    }

    items
}

fn scan_service_management() -> Vec<OptimItem> {
    let mut items = Vec::new();
    let unnecessary_services = ["cups", "bluetooth", "avahi-daemon", "ModemManager"];

    for svc in &unnecessary_services {
        let unit = format!("{}.service", svc);
        let enabled = run_cmd(&format!(
            "systemctl is-enabled {} 2>/dev/null",
            unit
        ));
        if enabled.trim() == "enabled" {
            items.push(OptimItem {
                name: svc.to_string(),
                description: format!("Service '{}' is enabled but may not be needed on a desktop system", svc),
                current_value: "enabled".to_string(),
                recommended_value: "disabled".to_string(),
                category: "Services".to_string(),
                action: format!("systemctl disable --now {}", unit),
            });
        }
    }

    items
}

fn scan_kernel_modules() -> Vec<OptimItem> {
    let mut items = Vec::new();
    let unused_modules = ["nfs", "nfsd", "cramfs", "freevxfs", "hfs", "hfsplus", "jffs2", "udf"];

    for mod_name in &unused_modules {
        let check = run_cmd(&format!("lsmod | grep -w {} 2>/dev/null", mod_name));
        let probe_result = run_cmd(&format!("modprobe -n -v {} 2>&1", mod_name));
        let is_loadable = !probe_result.contains("not found") && !probe_result.is_empty();

        if is_loadable {
            if check.is_empty() {
                items.push(OptimItem {
                    name: format!("blacklist {}", mod_name),
                    description: format!("Module '{}' is available but unused; blacklisting prevents auto-loading", mod_name),
                    current_value: "not loaded".to_string(),
                    recommended_value: "blacklisted".to_string(),
                    category: "Kernel".to_string(),
                    action: format!("echo 'blacklist {}' | sudo tee /etc/modules-load.d/blacklist-{}.conf", mod_name, mod_name),
                });
            }
        }
    }

    items
}

fn scan_swap_optimization() -> Vec<OptimItem> {
    let mut items = Vec::new();

    // Check zram
    let zram_check = run_cmd("ls /dev/zram* 2>/dev/null");
    if zram_check.is_empty() {
        let has_zram = run_cmd("which zramctl 2>/dev/null");
        if !has_zram.is_empty() {
            items.push(OptimItem {
                name: "zram swap".to_string(),
                description: "zram provides compressed RAM-based swap, faster than disk swap".to_string(),
                current_value: "not configured".to_string(),
                recommended_value: "enabled".to_string(),
                category: "Swap".to_string(),
                action: "sudo modprobe zram && sudo zramctl /dev/zram0 --algorithm lz4 --size $(free -m | awk '/Mem:/{printf \"%dM\", $2/2}') --type swap && sudo mkswap /dev/zram0 && sudo swapon /dev/zram0".to_string(),
            });
        }
    }

    items
}

fn scan_filesystem_optimization() -> Vec<OptimItem> {
    let mut items = Vec::new();

    // Check TRIM for SSDs
    let has_ssd = run_cmd("lsblk -d -o ROTA 2>/dev/null | grep -w 0");
    if !has_ssd.is_empty() {
        let trim_timer = run_cmd("systemctl is-enabled fstrim.timer 2>/dev/null");
        let trim_service = run_cmd("systemctl is-active fstrim.timer 2>/dev/null");
        if trim_timer.trim() != "enabled" && trim_service.trim() != "active" {
            items.push(OptimItem {
                name: "fstrim.timer".to_string(),
                description: "Regular TRIM improves SSD performance and lifespan".to_string(),
                current_value: "disabled".to_string(),
                recommended_value: "enabled".to_string(),
                category: "Filesystem".to_string(),
                action: "sudo systemctl enable --now fstrim.timer".to_string(),
            });
        }
    }

    // Check noatime
    let mounts = run_cmd("awk '$2 == \"/\" { print }' /proc/mounts | grep -v noatime");
    if !mounts.is_empty() {
        items.push(OptimItem {
            name: "noatime mount option".to_string(),
            description: "noatime reduces disk writes by not recording file access times".to_string(),
            current_value: "with atime".to_string(),
            recommended_value: "noatime".to_string(),
            category: "Filesystem".to_string(),
            action: "# Requires editing /etc/fstab - add 'noatime' to root partition mount options".to_string(),
        });
    }

    items
}

fn scan_network_optimization() -> Vec<OptimItem> {
    let mut items = Vec::new();

    // TCP receive buffer
    let rmem_max = get_sysctl_value("net.core.rmem_max");
    let recommended_rmem = "16777216".to_string();
    if rmem_max != recommended_rmem && !rmem_max.is_empty() {
        items.push(OptimItem {
            name: "net.core.rmem_max".to_string(),
            description: "Increase TCP receive buffer for high-speed network connections".to_string(),
            current_value: rmem_max,
            recommended_value: recommended_rmem.clone(),
            category: "Network".to_string(),
            action: format!("sysctl -w net.core.rmem_max={}", recommended_rmem),
        });
    }

    // TCP send buffer
    let wmem_max = get_sysctl_value("net.core.wmem_max");
    let recommended_wmem = "16777216".to_string();
    if wmem_max != recommended_wmem && !wmem_max.is_empty() {
        items.push(OptimItem {
            name: "net.core.wmem_max".to_string(),
            description: "Increase TCP send buffer for high-speed network connections".to_string(),
            current_value: wmem_max,
            recommended_value: recommended_wmem.clone(),
            category: "Network".to_string(),
            action: format!("sysctl -w net.core.wmem_max={}", recommended_wmem),
        });
    }

    // TCP congestion control
    let tcp_congestion = get_sysctl_value("net.ipv4.tcp_congestion_control");
    let recommended_congestion = "bbr".to_string();
    if tcp_congestion != recommended_congestion && !tcp_congestion.is_empty() {
        items.push(OptimItem {
            name: "net.ipv4.tcp_congestion_control".to_string(),
            description: "BBR congestion control improves throughput on modern networks".to_string(),
            current_value: tcp_congestion,
            recommended_value: recommended_congestion.clone(),
            category: "Network".to_string(),
            action: format!(
                "sysctl -w net.ipv4.tcp_congestion_control={} && echo {} | sudo tee /etc/sysctl.d/99-tcp-bbr.conf",
                recommended_congestion, recommended_congestion
            ),
        });
    }

    items
}

fn scan_power_management() -> Vec<OptimItem> {
    let mut items = Vec::new();

    // Check TLP
    let tlp_installed = run_cmd("which tlp 2>/dev/null");
    if tlp_installed.is_empty() {
        items.push(OptimItem {
            name: "TLP".to_string(),
            description: "TLP provides automatic laptop power optimization".to_string(),
            current_value: "not installed".to_string(),
            recommended_value: "installed".to_string(),
            category: "Power".to_string(),
            action: "sudo apt-get install -y tlp".to_string(),
        });
    } else {
        let tlp_active = run_cmd("systemctl is-active tlp 2>/dev/null");
        if tlp_active.trim() != "active" {
            items.push(OptimItem {
                name: "TLP service".to_string(),
                description: "TLP is installed but not running".to_string(),
                current_value: "inactive".to_string(),
                recommended_value: "active".to_string(),
                category: "Power".to_string(),
                action: "sudo systemctl enable --now tlp".to_string(),
            });
        }
    }

    // Check laptop-mode-tools
    let lmt_installed = run_cmd("which laptop-mode-tools 2>/dev/null");
    let lmt_service = run_cmd("systemctl is-enabled laptop-mode 2>/dev/null");
    if lmt_installed.is_empty() && lmt_service.trim() != "enabled" {
        items.push(OptimItem {
            name: "laptop-mode-tools".to_string(),
            description: "Alternative power management tool for laptops".to_string(),
            current_value: "not installed".to_string(),
            recommended_value: "optional".to_string(),
            category: "Power".to_string(),
            action: "sudo apt-get install -y laptop-mode-tools".to_string(),
        });
    }

    items
}

fn scan_memory_optimization() -> Vec<OptimItem> {
    let mut items = Vec::new();

    // Check zram
    let zram_check = run_cmd("ls /dev/zram* 2>/dev/null");
    if zram_check.is_empty() {
        let zram_module = run_cmd("modprobe zram 2>&1");
        if zram_module.is_empty() || !zram_module.contains("not found") {
            items.push(OptimItem {
                name: "zram".to_string(),
                description: "zram provides compressed RAM-based swap, faster than disk-based swap".to_string(),
                current_value: "not configured".to_string(),
                recommended_value: "enabled".to_string(),
                category: "Memory".to_string(),
                action: "sudo modprobe zram && sudo zramctl /dev/zram0 --algorithm lz4 --size $(free -m | awk '/Mem:/{printf \"%dM\", $2/2}') --type swap && sudo mkswap /dev/zram0 && sudo swapon /dev/zram0".to_string(),
            });
        }
    }

    // Check zswap
    let zswap_check = get_sysctl_value("vm.zswap.enabled");
    if zswap_check != "1" && !zswap_check.is_empty() {
        items.push(OptimItem {
            name: "zswap".to_string(),
            description: "zswap is a compressed write-back cache for swap pages in RAM".to_string(),
            current_value: zswap_check,
            recommended_value: "1".to_string(),
            category: "Memory".to_string(),
            action: "sudo sysctl -w vm.zswap.enabled=1".to_string(),
        });
    }

    // Check huge pages
    let hugepages = get_sysctl_value("vm.nr_hugepages");
    let total_mem_kb = run_cmd("awk '/MemTotal/{print $2}' /proc/meminfo");
    let total_mem_mb: u64 = total_mem_kb.parse::<u64>().unwrap_or(0) / 1024;
    if total_mem_mb >= 8192 && (hugepages == "0" || hugepages.is_empty()) {
        items.push(OptimItem {
            name: "vm.nr_hugepages".to_string(),
            description: "Huge pages can improve performance for memory-intensive applications".to_string(),
            current_value: hugepages,
            recommended_value: "1024".to_string(),
            category: "Memory".to_string(),
            action: "sudo sysctl -w vm.nr_hugepages=1024".to_string(),
        });
    }

    items
}

fn perform_full_scan() -> Vec<OptimItem> {
    let mut items = Vec::new();
    items.extend(scan_kernel_tuning());
    items.extend(scan_service_management());
    items.extend(scan_kernel_modules());
    items.extend(scan_filesystem_optimization());
    items.extend(scan_network_optimization());
    items.extend(scan_power_management());
    items.extend(scan_memory_optimization());
    items
}

fn spawn_blocking_with_result<T, F, C>(work: F, callback: C)
where
    T: Send + 'static,
    F: Send + 'static,
    F: FnOnce() -> T,
    C: FnOnce(T) + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = work();
        let _ = tx.send(result);
    });
    let mut callback = Some(callback);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(result) => {
                if let Some(cb) = callback.take() {
                    cb(result);
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                if let Some(cb) = callback.take() {
                    // Channel closed without sending — shouldn't happen but handle gracefully
                    drop(cb);
                }
                glib::ControlFlow::Break
            }
        }
    });
}

pub fn build_optimizer_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Title
    let title = Label::new(Some(tr!("system_optimization")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // Info frame
    let info_frame = Frame::new(Some(tr!("optimizer")));
    info_frame.set_css_classes(&["card"]);

    let info_label = Label::new(Some(
        "This module scans your system for performance optimizations. \
         Review each item and select the ones you want to apply. \
         Optimizations include kernel tuning, service management, \
         filesystem and network improvements, and power management.",
    ));
    info_label.set_wrap(true);
    info_label.set_max_width_chars(80);
    info_label.set_halign(gtk::Align::Start);
    info_label.set_margin_top(8);
    info_label.set_margin_bottom(8);
    info_label.set_margin_start(12);
    info_label.set_margin_end(12);

    info_frame.set_child(Some(&info_label));
    main_box.append(&info_frame);

    // Items display frame
    let items_frame = Frame::new(Some(tr!("optimization_items")));
    items_frame.set_css_classes(&["card"]);

    let items_scrolled = ScrolledWindow::new();
    items_scrolled.set_min_content_height(250);
    items_scrolled.set_vexpand(true);

    let items_text_view = TextView::new();
    items_text_view.set_editable(false);
    items_text_view.set_monospace(true);
    items_text_view.set_wrap_mode(gtk::WrapMode::None);
    items_text_view.set_left_margin(8);
    items_text_view.set_top_margin(8);

    let items_buffer = items_text_view.buffer();
    items_buffer.set_text(tr!("loading"));

    items_scrolled.set_child(Some(&items_text_view));
    items_scrolled.set_margin_top(10);
    items_scrolled.set_margin_bottom(10);
    items_scrolled.set_margin_start(12);
    items_scrolled.set_margin_end(12);

    items_frame.set_child(Some(&items_scrolled));
    main_box.append(&items_frame);

    // Checkbuttons frame
    let checks_frame = Frame::new(Some(tr!("optimization_items")));
    checks_frame.set_css_classes(&["card"]);

    let checks_grid = Grid::new();
    checks_grid.set_column_spacing(12);
    checks_grid.set_row_spacing(6);
    checks_grid.set_margin_top(10);
    checks_grid.set_margin_bottom(10);
    checks_grid.set_margin_start(12);
    checks_grid.set_margin_end(12);

    checks_frame.set_child(Some(&checks_grid));
    main_box.append(&checks_frame);

    // Action buttons
    let buttons_grid = Grid::new();
    buttons_grid.set_column_spacing(12);
    buttons_grid.set_row_spacing(10);
    buttons_grid.set_margin_top(10);
    buttons_grid.set_margin_bottom(10);
    buttons_grid.set_margin_start(12);
    buttons_grid.set_margin_end(12);

    let scan_button = Button::with_label(tr!("scan_optimizations"));
    scan_button.set_css_classes(&["suggested-action"]);

    let apply_button = Button::with_label(tr!("apply_selected"));
    apply_button.set_css_classes(&["suggested-action"]);
    apply_button.set_sensitive(false);

    let status_label = Label::new(Some(tr!("loading")));
    status_label.set_halign(gtk::Align::Start);
    status_label.set_css_classes(&["dim-label"]);
    status_label.set_hexpand(true);

    buttons_grid.attach(&scan_button, 0, 0, 1, 1);
    buttons_grid.attach(&apply_button, 1, 0, 1, 1);
    buttons_grid.attach(&status_label, 2, 0, 1, 1);

    main_box.append(&buttons_grid);

    // Results frame
    let results_frame = Frame::new(Some(tr!("optimization_results")));
    results_frame.set_css_classes(&["card"]);

    let results_scrolled = ScrolledWindow::new();
    results_scrolled.set_min_content_height(150);
    results_scrolled.set_max_content_height(300);

    let results_text_view = TextView::new();
    results_text_view.set_editable(false);
    results_text_view.set_monospace(true);
    results_text_view.set_wrap_mode(gtk::WrapMode::Word);
    results_text_view.set_left_margin(8);
    results_text_view.set_top_margin(8);

    let results_buffer = results_text_view.buffer();
    results_buffer.set_text("");

    results_scrolled.set_child(Some(&results_text_view));
    results_scrolled.set_margin_top(10);
    results_scrolled.set_margin_bottom(10);
    results_scrolled.set_margin_start(12);
    results_scrolled.set_margin_end(12);

    results_frame.set_child(Some(&results_scrolled));
    main_box.append(&results_frame);

    // Shared state for scanned items and check buttons
    let scanned_items: Rc<RefCell<Vec<OptimItem>>> = Rc::new(RefCell::new(Vec::new()));
    let check_buttons: Rc<RefCell<Vec<CheckButton>>> = Rc::new(RefCell::new(Vec::new()));

    // ── Scan function ──

    let perform_scan = {
        let items_buffer = items_buffer.clone();
        let status_label = status_label.clone();
        let checks_grid = checks_grid.clone();
        let apply_button = apply_button.clone();
        let scanned_items = scanned_items.clone();
        let check_buttons = check_buttons.clone();

        move || {
            let items_buffer = items_buffer.clone();
            let status_label = status_label.clone();
            let checks_grid = checks_grid.clone();
            let apply_button = apply_button.clone();
            let scanned_items = scanned_items.clone();
            let check_buttons = check_buttons.clone();

            status_label.set_text("Scanning...");
            apply_button.set_sensitive(false);
            items_buffer.set_text("Scanning system for optimizations...");

            // Clear old check buttons
            {
                let mut btns = check_buttons.borrow_mut();
                for btn in btns.iter() {
                    checks_grid.remove(btn);
                }
                btns.clear();
            }

            spawn_blocking_with_result(
                perform_full_scan,
                move |items| {
                    let count = items.len();

                    if items.is_empty() {
                        items_buffer.set_text("No optimizations found. Your system is already well-tuned!");
                        status_label.set_text("Scan complete. No items found.");
                        return;
                    }

                    // Build text view content
                    let header = format!(
                        "{:<22} {:<38} {:<20} {:<20}",
                        "CATEGORY", "ITEM", "CURRENT", "RECOMMENDED"
                    );
                    let separator = "\u{2500}".repeat(header.len());
                    let mut lines = vec![header, separator];

                    for item in &items {
                        let category: String = item.category.chars().take(22).collect();
                        let category = if item.category.chars().count() > 22 {
                            format!("{}...", category)
                        } else {
                            category
                        };
                        let name: String = item.name.chars().take(38).collect();
                        let name = if item.name.chars().count() > 38 {
                            format!("{}...", name)
                        } else {
                            name
                        };
                        let current: String = item.current_value.chars().take(20).collect();
                        let current = if item.current_value.chars().count() > 20 {
                            format!("{}...", current)
                        } else {
                            current
                        };
                        let recommended: String = item.recommended_value.chars().take(20).collect();
                        let recommended = if item.recommended_value.chars().count() > 20 {
                            format!("{}...", recommended)
                        } else {
                            recommended
                        };
                        lines.push(format!(
                            "{:<22} {:<38} {:<20} {:<20}",
                            category, name, current, recommended
                        ));
                    }

                    items_buffer.set_text(&lines.join("\n"));

                    // Build check buttons
                    {
                        let mut btns = check_buttons.borrow_mut();
                        for (i, item) in items.iter().enumerate() {
                            let check = CheckButton::with_label(&format!(
                                "[{}] {} - {}",
                                item.category, item.name, item.description
                            ));
                            check.set_active(true);
                            checks_grid.attach(&check, 0, i as i32, 1, 1);
                            btns.push(check);
                        }
                    }

                    *scanned_items.borrow_mut() = items;
                    apply_button.set_sensitive(true);
                    status_label.set_text(&format!("Found {} optimization items.", count));
                },
            );
        }
    };

    // ── Wire up "Scan" button ──

    {
        let perform_scan = perform_scan.clone();
        scan_button.connect_clicked(move |_| {
            perform_scan();
        });
    }

    // ── Wire up "Apply Selected" button ──

    {
        let results_buffer = results_buffer.clone();
        let status_label = status_label.clone();
        let scan_button = scan_button.clone();
        let scanned_items = scanned_items.clone();
        let check_buttons = check_buttons.clone();

        apply_button.connect_clicked(move |btn| {
            // Collect owned data while holding the borrow, then drop it
            let (selected_names, selected_commands) = {
                let items = scanned_items.borrow();
                let btns = check_buttons.borrow();

                let names: Vec<String> = items
                    .iter()
                    .zip(btns.iter())
                    .filter(|(_, check)| check.is_active())
                    .map(|(item, _)| item.name.clone())
                    .collect();

                let cmds: Vec<String> = items
                    .iter()
                    .zip(btns.iter())
                    .filter(|(_, check)| check.is_active())
                    .map(|(item, _)| item.action.clone())
                    .collect();

                (names, cmds)
            }; // borrows dropped here

            if selected_names.is_empty() {
                results_buffer.set_text("No items selected. Please check the optimizations you want to apply.");
                return;
            }

            btn.set_sensitive(false);
            scan_button.set_sensitive(false);
            status_label.set_text("Applying optimizations...");
            results_buffer.set_text("Applying selected optimizations...\n\n");

            let apply_button_weak = btn.downgrade();
            let scan_button_c = scan_button.clone();
            let status_label_c = status_label.clone();
            let results_buffer_c = results_buffer.clone();
            spawn_blocking_with_result(
                move || {
                    let mut results = Vec::new();
                    for (i, cmd) in selected_commands.iter().enumerate() {
                        if cmd.starts_with('#') {
                            results.push(format!(
                                "{}: [MANUAL ACTION REQUIRED]\n{}",
                                &selected_names[i],
                                cmd
                            ));
                            continue;
                        }
                        let needs_root = cmd.contains("sudo ") || cmd.starts_with("sysctl");
                        let output = if needs_root {
                            // Use double quotes so $() subshells expand properly
                            let escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"").replace('`', "\\`");
                            run_cmd(&format!("pkexec sh -c \"{}\"", escaped))
                        } else {
                            run_cmd(cmd)
                        };
                        if output.is_empty() {
                            results.push(format!("[OK] {}", &selected_names[i]));
                        } else {
                            results.push(format!("[OK] {}\n{}", &selected_names[i], output));
                        }
                    }
                    results
                },
                move |results| {
                    let mut output = String::from("Optimization Results:\n");
                    output.push_str(&"\u{2500}".repeat(50));
                    output.push('\n');
                    for result in &results {
                        output.push_str(result);
                        output.push_str("\n\n");
                    }
                    output.push_str(&"\u{2500}".repeat(50));
                    output.push('\n');
                    output.push_str(&format!("Completed {} optimizations.", results.len()));

                    results_buffer_c.set_text(&output);
                    if let Some(btn) = apply_button_weak.upgrade() {
                        btn.set_sensitive(true);
                    }
                    scan_button_c.set_sensitive(true);
                    status_label_c.set_text(&format!(
                        "Applied {} optimization(s).",
                        results.len()
                    ));
                },
            );
        });
    }

    // ── Initial scan on page load using glib::idle_add_local_once ──

    {
        let perform_scan = perform_scan.clone();
        glib::idle_add_local_once(move || {
            perform_scan();
        });
    }

    main_box.upcast()
}
