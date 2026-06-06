use gtk::glib;
use gtk::prelude::*;
use gtk::{Box, Button, CheckButton, Frame, Grid, Label, Orientation};

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
        .and_then(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if o.status.success() {
                Some(stdout)
            } else if stderr.is_empty() {
                Some(stdout)
            } else {
                Some(format!("{}\n{}", stdout, stderr))
            }
        })
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn parse_du_size(output: &str) -> String {
    let line = output.lines().last().unwrap_or("0");
    let size_part = line.split_whitespace().next().unwrap_or("0");
    if size_part == "0" || size_part.is_empty() {
        "0 B".to_string()
    } else {
        size_part.to_string()
    }
}

fn du_to_kb(s: &str) -> f64 {
    let s = s.trim();
    if s == "0" || s.is_empty() {
        return 0.0;
    }
    let (num_part, suffix) = if let Some(stripped) = s.strip_suffix('T') {
        (stripped, 1_073_741_824.0)
    } else if let Some(stripped) = s.strip_suffix('G') {
        (stripped, 1_048_576.0)
    } else if let Some(stripped) = s.strip_suffix('M') {
        (stripped, 1024.0)
    } else if let Some(stripped) = s.strip_suffix('K') {
        (stripped, 1.0)
    } else {
        (s, 1.0)
    };
    num_part.parse::<f64>().unwrap_or(0.0) * suffix
}

fn kb_to_human(kb: f64) -> String {
    if kb >= 1_048_576.0 {
        format!("{:.2} GB", kb / 1_048_576.0)
    } else if kb >= 1024.0 {
        format!("{:.1} MB", kb / 1024.0)
    } else if kb >= 1.0 {
        format!("{:.0} KB", kb)
    } else {
        "0 B".to_string()
    }
}

fn scan_apt_cache() -> String {
    let out = run_cmd("du -sh /var/cache/apt/archives/ 2>/dev/null");
    parse_du_size(&out)
}

fn scan_snap_cache() -> String {
    let out = run_cmd("du -sh ~/snap/ /var/snap/ 2>/dev/null");
    let mut total_kb: f64 = 0.0;
    for line in out.lines() {
        if let Some(size_str) = line.split_whitespace().next() {
            total_kb += du_to_kb(size_str);
        }
    }
    kb_to_human(total_kb)
}

fn scan_flatpak_cache() -> String {
    let out = run_cmd("du -sh ~/.local/share/flatpak/ 2>/dev/null");
    parse_du_size(&out)
}

fn scan_log_files() -> String {
    let out = run_cmd("find /var/log -name '*.log' -exec du -ch {} + 2>/dev/null | tail -1");
    parse_du_size(&out)
}

fn scan_temp_files() -> String {
    let out = run_cmd("du -sh /tmp/ ~/.cache/ 2>/dev/null");
    let mut total_kb: f64 = 0.0;
    for line in out.lines() {
        if let Some(size_str) = line.split_whitespace().next() {
            total_kb += du_to_kb(size_str);
        }
    }
    kb_to_human(total_kb)
}

fn scan_browser_cache() -> String {
    let out = run_cmd("du -sh ~/.cache/mozilla/ ~/.cache/google-chrome/ ~/.cache/chromium/ ~/.cache/BraveSoftware/ ~/.cache/microsoft-edge/ ~/.cache/vivaldi/ 2>/dev/null");
    let mut total_kb: f64 = 0.0;
    for line in out.lines() {
        if let Some(size_str) = line.split_whitespace().next() {
            total_kb += du_to_kb(size_str);
        }
    }
    kb_to_human(total_kb)
}

fn scan_cleanup_sizes() -> Vec<(&'static str, String)> {
    vec![
        ("APT cache", scan_apt_cache()),
        ("Snap cache", scan_snap_cache()),
        ("Flatpak cache", scan_flatpak_cache()),
        ("Log files", scan_log_files()),
        ("Temporary files", scan_temp_files()),
        ("Browser cache", scan_browser_cache()),
    ]
}

fn calculate_total(sizes: &[(&str, String)]) -> String {
    let total_kb: f64 = sizes.iter().map(|(_, s)| du_to_kb(s)).sum();
    kb_to_human(total_kb)
}

fn clean_apt_cache() -> String {
    let out = run_cmd("sudo apt-get clean 2>&1");
    if out.is_empty() {
        "APT cache cleaned.".to_string()
    } else {
        format!("APT: {}", out.lines().next().unwrap_or("done"))
    }
}

fn clean_snap_cache() -> String {
    let out = run_cmd(
        r#"snap list --all 2>/dev/null | awk '$NF == "disabled" {print $1, $3}' | while read snapname revision; do sudo snap remove "$snapname" --revision="$revision" 2>&1; done"#,
    );
    if out.is_empty() {
        "No disabled Snap packages to remove.".to_string()
    } else {
        format!("Snap cleanup done. Removed:\n{}", out)
    }
}

fn clean_flatpak_cache() -> String {
    let out = run_cmd("flatpak uninstall --unused -y 2>&1");
    if out.is_empty() || out.contains("Nothing unused") {
        "No unused Flatpak packages.".to_string()
    } else {
        "Flatpak unused packages removed.".to_string()
    }
}

fn clean_log_files() -> String {
    let out = run_cmd("sudo journalctl --vacuum-size=50M 2>&1");
    if out.is_empty() {
        "Journal logs vacuumed to 50M.".to_string()
    } else {
        format!("Logs: {}", out.lines().last().unwrap_or("done"))
    }
}

fn clean_temp_files() -> String {
    let out = run_cmd("rm -rf /tmp/* ~/.cache/thumbnails/* 2>&1");
    if out.is_empty() {
        "Temporary files cleaned.".to_string()
    } else {
        format!("Temp cleanup: {}", out.lines().next().unwrap_or("done"))
    }
}

fn clean_browser_cache() -> String {
    let out = run_cmd("rm -rf ~/.cache/mozilla/ ~/.cache/google-chrome/ ~/.cache/chromium/ ~/.cache/BraveSoftware/ ~/.cache/microsoft-edge/ ~/.cache/vivaldi/ 2>&1");
    if out.is_empty() {
        "Browser cache cleaned.".to_string()
    } else {
        format!("Browser cache: {}", out.lines().next().unwrap_or("done"))
    }
}

fn scan_duplicates() -> (u64, String) {
    let script = r#"
tmpfile=$(mktemp)
find ~/Downloads ~/Documents ~/.cache -type f -size +100k 2>/dev/null | head -1000 | xargs md5sum 2>/dev/null | sort > "$tmpfile"
dup_hashes=$(awk '{print $1}' "$tmpfile" | uniq -d)
dup_count=0
dup_size=0
for hash in $dup_hashes; do
    files=$(grep "^$hash" "$tmpfile" | tail -n +2 | awk '{print $2}')
    for f in $files; do
        sz=$(stat -c%s "$f" 2>/dev/null || echo 0)
        dup_size=$((dup_size + sz))
        dup_count=$((dup_count + 1))
    done
done
rm -f "$tmpfile"
echo "$dup_count $dup_size"
"#;
    let out = run_cmd(script);
    let parts: Vec<&str> = out.split_whitespace().collect();
    let count: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let size_bytes: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let size_str = if size_bytes >= 1_073_741_824 {
        format!("{:.2} GB", size_bytes as f64 / 1_073_741_824.0)
    } else if size_bytes >= 1_048_576 {
        format!("{:.1} MB", size_bytes as f64 / 1_048_576.0)
    } else if size_bytes >= 1024 {
        format!("{:.0} KB", size_bytes as f64 / 1024.0)
    } else {
        format!("{} B", size_bytes)
    };
    (count, size_str)
}

fn remove_duplicates() -> String {
    let script = r#"
tmpfile=$(mktemp)
find ~/Downloads ~/Documents ~/.cache -type f -size +100k 2>/dev/null | head -1000 | xargs md5sum 2>/dev/null | sort > "$tmpfile"
dup_hashes=$(awk '{print $1}' "$tmpfile" | uniq -d)
removed=0
for hash in $dup_hashes; do
    grep "^$hash" "$tmpfile" | tail -n +2 | awk '{print $2}' | while IFS= read -r f; do
        rm -f "$f" && echo "1"
    done
done | {
    count=0
    while IFS= read -r line; do
        count=$((count + 1))
    done
    echo "$count"
}
rm -f "$tmpfile"
"#;
    let out = run_cmd(script);
    let count: u64 = out.trim().lines().last().and_then(|l| l.parse().ok()).unwrap_or(0);
    if count > 0 {
        format!("Removed {} duplicate files.", count)
    } else {
        "No duplicate files removed.".to_string()
    }
}

/// Spawn a blocking task on a background thread and run a callback on the main thread.
fn spawn_bg<T: Send + 'static, F: FnOnce() -> T + Send + 'static, C: FnOnce(T) + 'static>(
    work: F,
    callback: C,
) {
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
                    drop(cb);
                }
                glib::ControlFlow::Break
            }
        }
    });
}

pub fn build_cleanup_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("cleanup")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Cleanup Categories with checkboxes ──

    let categories_frame = Frame::new(Some(tr!("cleanup_categories")));
    categories_frame.set_css_classes(&["card"]);

    let categories_grid = Grid::new();
    categories_grid.set_column_spacing(20);
    categories_grid.set_row_spacing(10);
    categories_grid.set_margin_top(10);
    categories_grid.set_margin_bottom(10);
    categories_grid.set_margin_start(10);
    categories_grid.set_margin_end(10);

    let category_names = [
        "APT cache",
        "Snap cache",
        "Flatpak cache",
        "Log files",
        "Temporary files",
        "Browser cache",
    ];

    let check_buttons: Vec<CheckButton> = category_names
        .iter()
        .map(|name| {
            let check = CheckButton::with_label(name);
            check.set_active(true);
            check
        })
        .collect();

    let size_labels: Vec<Label> = (0..category_names.len())
        .map(|_| {
            let label = Label::new(Some("Scanning..."));
            label.set_halign(gtk::Align::Start);
            label.set_css_classes(&["dim-label"]);
            label
        })
        .collect();

    for (i, (check, size_label)) in check_buttons.iter().zip(size_labels.iter()).enumerate() {
        categories_grid.attach(check, 0, i as i32, 1, 1);
        categories_grid.attach(size_label, 1, i as i32, 1, 1);
    }

    categories_frame.set_child(Some(&categories_grid));
    main_box.append(&categories_frame);

    // ── Total Space & Actions ──

    let total_frame = Frame::new(Some(tr!("total_space")));
    total_frame.set_css_classes(&["card"]);

    let total_grid = Grid::new();
    total_grid.set_column_spacing(20);
    total_grid.set_row_spacing(10);
    total_grid.set_margin_top(10);
    total_grid.set_margin_bottom(10);
    total_grid.set_margin_start(10);
    total_grid.set_margin_end(10);

    let total_label = Label::new(Some(&format!("{} Scanning...", tr!("disk_usage"))));
    total_label.set_css_classes(&["title-2"]);
    total_label.set_halign(gtk::Align::Start);

    let status_label = Label::new(None);
    status_label.set_halign(gtk::Align::Start);
    status_label.set_css_classes(&["dim-label"]);
    status_label.set_wrap(true);
    status_label.set_max_width_chars(60);

    let clean_button = Button::with_label(&tr!("clean_selected"));
    clean_button.set_css_classes(&["suggested-action"]);
    clean_button.set_halign(gtk::Align::End);

    let scan_button = Button::with_label(&tr!("scan_again"));
    scan_button.set_halign(gtk::Align::End);

    total_grid.attach(&total_label, 0, 0, 1, 1);
    total_grid.attach(&status_label, 0, 1, 3, 1);
    total_grid.attach(&clean_button, 1, 0, 1, 1);
    total_grid.attach(&scan_button, 2, 0, 1, 1);

    total_frame.set_child(Some(&total_grid));
    main_box.append(&total_frame);

    // ── Duplicate Files ──

    let duplicate_frame = Frame::new(Some(tr!("duplicate_files")));
    duplicate_frame.set_css_classes(&["card"]);

    let duplicate_grid = Grid::new();
    duplicate_grid.set_column_spacing(20);
    duplicate_grid.set_row_spacing(10);
    duplicate_grid.set_margin_top(10);
    duplicate_grid.set_margin_bottom(10);
    duplicate_grid.set_margin_start(10);
    duplicate_grid.set_margin_end(10);

    let duplicate_label = Label::new(Some(tr!("find_remove_duplicates")));
    duplicate_label.set_halign(gtk::Align::Start);

    let scan_duplicates_button = Button::with_label(&tr!("scan_duplicates"));
    scan_duplicates_button.set_halign(gtk::Align::Start);
    scan_duplicates_button.set_css_classes(&["suggested-action"]);

    let duplicate_info_label = Label::new(Some("Click 'Scan for Duplicates' to begin."));
    duplicate_info_label.set_halign(gtk::Align::Start);
    duplicate_info_label.set_css_classes(&["dim-label"]);
    duplicate_info_label.set_wrap(true);
    duplicate_info_label.set_max_width_chars(60);

    let remove_duplicates_button = Button::with_label(&tr!("remove_duplicates"));
    remove_duplicates_button.set_halign(gtk::Align::Start);
    remove_duplicates_button.set_css_classes(&["destructive-action"]);
    remove_duplicates_button.set_sensitive(false);

    duplicate_grid.attach(&duplicate_label, 0, 0, 2, 1);
    duplicate_grid.attach(&scan_duplicates_button, 0, 1, 1, 1);
    duplicate_grid.attach(&duplicate_info_label, 1, 1, 1, 1);
    duplicate_grid.attach(&remove_duplicates_button, 0, 2, 2, 1);

    duplicate_frame.set_child(Some(&duplicate_grid));
    main_box.append(&duplicate_frame);

    // ── Shared state ──

    let sizes_state: Rc<RefCell<Vec<(&'static str, String)>>> = Rc::new(RefCell::new(
        category_names
            .iter()
            .map(|n| (*n, "0 B".to_string()))
            .collect(),
    ));

    // ── Scan function ──

    let perform_scan = {
        let size_labels = size_labels.clone();
        let total_label = total_label.clone();
        let status_label = status_label.clone();
        let sizes_state = sizes_state.clone();
        let scan_button = scan_button.clone();
        let clean_button = clean_button.clone();

        move || {
            let size_labels = size_labels.clone();
            let total_label = total_label.clone();
            let status_label = status_label.clone();
            let sizes_state = sizes_state.clone();
            let scan_button_c = scan_button.clone();
            let clean_button_c = clean_button.clone();

            scan_button.set_sensitive(false);
            clean_button.set_sensitive(false);
            status_label.set_text("Scanning cache sizes...");

            for label in size_labels.iter() {
                label.set_text("Scanning...");
            }
            total_label.set_text(&format!("{} Scanning...", tr!("disk_usage")));

            spawn_bg(
                || scan_cleanup_sizes(),
                move |scan_result| {
                    {
                        let mut state = sizes_state.borrow_mut();
                        *state = scan_result;
                    }

                    let state = sizes_state.borrow();
                    for (i, (_, size)) in state.iter().enumerate() {
                        if let Some(label) = size_labels.get(i) {
                            label.set_text(size);
                        }
                    }

                    let total = calculate_total(&state);
                    total_label.set_text(&format!("{} {}", tr!("disk_usage"), total));
                    status_label.set_text("Scan complete.");
                    scan_button_c.set_sensitive(true);
                    clean_button_c.set_sensitive(true);
                },
            );
        }
    };

    // ── Wire up "Scan Again" button ──

    {
        let perform_scan = perform_scan.clone();
        scan_button.connect_clicked(move |_| {
            perform_scan();
        });
    }

    // ── Wire up "Clean Selected" button ──

    {
        let check_buttons = check_buttons.clone();
        let size_labels = size_labels.clone();
        let total_label = total_label.clone();
        let status_label = status_label.clone();
        let sizes_state = sizes_state.clone();
        let clean_button_c = clean_button.clone();
        let scan_button_c = scan_button.clone();

        clean_button.connect_clicked(move |_| {
            let selected: Vec<(&'static str, bool)> = category_names
                .iter()
                .zip(check_buttons.iter())
                .map(|(name, check)| (*name, check.is_active()))
                .collect();

            let has_any = selected.iter().any(|(_, active)| *active);
            if !has_any {
                status_label.set_text("No categories selected.");
                return;
            }

            clean_button_c.set_sensitive(false);
            scan_button_c.set_sensitive(false);
            status_label.set_text("Cleaning selected categories...");

            let size_labels = size_labels.clone();
            let total_label = total_label.clone();
            let status_label = status_label.clone();
            let sizes_state = sizes_state.clone();
            let clean_button_c = clean_button_c.clone();
            let scan_button_c = scan_button_c.clone();

            spawn_bg(
                move || {
                    let mut msgs = Vec::new();
                    for (name, active) in &selected {
                        if !active {
                            continue;
                        }
                        let msg = match *name {
                            "APT cache" => clean_apt_cache(),
                            "Snap cache" => clean_snap_cache(),
                            "Flatpak cache" => clean_flatpak_cache(),
                            "Log files" => clean_log_files(),
                            "Temporary files" => clean_temp_files(),
                            "Browser cache" => clean_browser_cache(),
                            _ => String::new(),
                        };
                        msgs.push(msg);
                    }
                    msgs
                },
                move |clean_result| {
                    // Re-scan after cleaning
                    let size_labels2 = size_labels.clone();
                    let total_label2 = total_label.clone();
                    let status_label2 = status_label.clone();
                    let sizes_state2 = sizes_state.clone();
                    let clean_button_c2 = clean_button_c.clone();
                    let scan_button_c2 = scan_button_c.clone();
                    let summary = clean_result.join(" | ");

                    spawn_bg(
                        || scan_cleanup_sizes(),
                        move |new_sizes| {
                            {
                                let mut state = sizes_state2.borrow_mut();
                                *state = new_sizes;
                            }

                            let state = sizes_state2.borrow();
                            for (i, (_, size)) in state.iter().enumerate() {
                                if let Some(label) = size_labels2.get(i) {
                                    label.set_text(size);
                                }
                            }

                            let total = calculate_total(&state);
                            total_label2.set_text(&format!("{} {}", tr!("disk_usage"), total));
                            status_label2.set_text(&format!("Done. {}", summary));
                            clean_button_c2.set_sensitive(true);
                            scan_button_c2.set_sensitive(true);
                        },
                    );
                },
            );
        });
    }

    // ── Wire up "Scan for Duplicates" button ──

    {
        let duplicate_info_label = duplicate_info_label.clone();
        let remove_duplicates_button = remove_duplicates_button.clone();
        let scan_duplicates_button_c = scan_duplicates_button.clone();

        scan_duplicates_button.connect_clicked(move |_| {
            duplicate_info_label.set_text("Scanning for duplicates (may take a moment)...");
            scan_duplicates_button_c.set_sensitive(false);
            remove_duplicates_button.set_sensitive(false);

            let duplicate_info_label = duplicate_info_label.clone();
            let remove_duplicates_button = remove_duplicates_button.clone();
            let scan_duplicates_button_c = scan_duplicates_button_c.clone();

            spawn_bg(
                scan_duplicates,
                move |result| {
                    let (count, size_str) = result;
                    if count == 0 {
                        duplicate_info_label.set_text("No duplicate files found.");
                        remove_duplicates_button.set_sensitive(false);
                    } else {
                        duplicate_info_label
                            .set_text(&format!("{} duplicate files ({})", count, size_str));
                        remove_duplicates_button.set_sensitive(true);
                    }
                    scan_duplicates_button_c.set_sensitive(true);
                },
            );
        });
    }

    // ── Wire up "Remove Duplicates" button ──

    {
        let duplicate_info_label = duplicate_info_label.clone();
        let remove_duplicates_button_c = remove_duplicates_button.clone();
        let scan_duplicates_button_c = scan_duplicates_button.clone();

        remove_duplicates_button.connect_clicked(move |_| {
            duplicate_info_label.set_text("Removing duplicates...");
            remove_duplicates_button_c.set_sensitive(false);
            scan_duplicates_button_c.set_sensitive(false);

            let duplicate_info_label = duplicate_info_label.clone();
            let remove_duplicates_button_c = remove_duplicates_button_c.clone();
            let scan_duplicates_button_c = scan_duplicates_button_c.clone();

            spawn_bg(
                remove_duplicates,
                move |result| {
                    duplicate_info_label.set_text(&result);
                    remove_duplicates_button_c.set_sensitive(false);
                    scan_duplicates_button_c.set_sensitive(true);
                },
            );
        });
    }

    // ── Initial scan on page load ──

    perform_scan();

    main_box.upcast()
}
