use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Box, Button, ComboBoxText, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, Switch,
    TextView,
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

fn run_ufw(args: &[&str]) -> String {
    let mut cmd_args = vec!["ufw"];
    cmd_args.extend_from_slice(args);
    run_command(&cmd_args)
}

fn get_ufw_status() -> String {
    let output = run_ufw(&["status", "verbose"]);
    output
}

fn is_ufw_active() -> bool {
    let output = run_command(&["ufw", "status"]);
    output.contains("active")
}

fn get_ufw_rules() -> String {
    let output = run_ufw(&["status", "numbered"]);
    output
}

fn get_ufw_app_profiles() -> String {
    let output = run_ufw(&["app", "list"]);
    output
}

pub fn build_firewall_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("firewall_management")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Firewall Status ──

    let status_frame = Frame::new(Some(tr!("ufw_status")));
    status_frame.set_css_classes(&["card"]);

    let status_grid = Grid::new();
    status_grid.set_column_spacing(12);
    status_grid.set_row_spacing(10);
    status_grid.set_margin_top(10);
    status_grid.set_margin_bottom(10);
    status_grid.set_margin_start(12);
    status_grid.set_margin_end(12);

    let status_label = Label::new(Some("Status: Checking..."));
    status_label.set_halign(gtk::Align::Start);
    status_label.set_css_classes(&["heading"]);

    let fw_enable_label = Label::new(Some("Firewall:"));
    fw_enable_label.set_halign(gtk::Align::Start);
    let fw_switch = Switch::new();
    fw_switch.set_active(is_ufw_active());
    fw_switch.set_halign(gtk::Align::Start);

    let status_detail_button = Button::with_label(&tr!("refresh"));
    status_detail_button.set_halign(gtk::Align::Start);
    status_detail_button.set_css_classes(&["suggested-action"]);

    status_grid.attach(&status_label, 0, 0, 3, 1);
    status_grid.attach(&fw_enable_label, 0, 1, 1, 1);
    status_grid.attach(&fw_switch, 1, 1, 1, 1);
    status_grid.attach(&status_detail_button, 2, 1, 1, 1);

    status_frame.set_child(Some(&status_grid));
    main_box.append(&status_frame);

    // ── Current Rules ──

    let rules_frame = Frame::new(Some(tr!("firewall_rules")));
    rules_frame.set_css_classes(&["card"]);

    let rules_scrolled = ScrolledWindow::new();
    rules_scrolled.set_min_content_height(200);
    rules_scrolled.set_max_content_height(350);

    let rules_text = TextView::new();
    rules_text.set_editable(false);
    rules_text.set_monospace(true);
    rules_text.set_vexpand(true);
    rules_text.buffer().set_text("Loading firewall rules...");

    rules_scrolled.set_child(Some(&rules_text));
    rules_scrolled.set_margin_top(10);
    rules_scrolled.set_margin_bottom(10);
    rules_scrolled.set_margin_start(12);
    rules_scrolled.set_margin_end(12);

    rules_frame.set_child(Some(&rules_scrolled));
    main_box.append(&rules_frame);

    // ── Add Rule ──

    let add_frame = Frame::new(Some(tr!("add_rule")));
    add_frame.set_css_classes(&["card"]);

    let add_grid = Grid::new();
    add_grid.set_column_spacing(12);
    add_grid.set_row_spacing(10);
    add_grid.set_margin_top(10);
    add_grid.set_margin_bottom(10);
    add_grid.set_margin_start(12);
    add_grid.set_margin_end(12);

    let action_label = Label::new(Some("Action:"));
    action_label.set_halign(gtk::Align::Start);
    let action_combo = ComboBoxText::new();
    action_combo.append_text("Allow");
    action_combo.append_text("Deny");
    action_combo.append_text("Reject");
    action_combo.append_text("Limit");
    action_combo.set_active(Some(0));
    action_combo.set_halign(gtk::Align::Start);

    let dir_label = Label::new(Some("Direction:"));
    dir_label.set_halign(gtk::Align::Start);
    let dir_combo = ComboBoxText::new();
    dir_combo.append_text("In");
    dir_combo.append_text("Out");
    dir_combo.append_text("Both");
    dir_combo.set_active(Some(0));
    dir_combo.set_halign(gtk::Align::Start);

    let port_label = Label::new(Some("Port/Service:"));
    port_label.set_halign(gtk::Align::Start);
    let port_entry = Entry::new();
    port_entry.set_hexpand(true);
    port_entry.set_placeholder_text(Some("22, 80, 443, or service name (ssh, http)"));

    let protocol_label = Label::new(Some("Protocol:"));
    protocol_label.set_halign(gtk::Align::Start);
    let protocol_combo = ComboBoxText::new();
    protocol_combo.append_text("tcp");
    protocol_combo.append_text("udp");
    protocol_combo.append_text("both");
    protocol_combo.set_active(Some(0));
    protocol_combo.set_halign(gtk::Align::Start);

    let from_label = Label::new(Some("From IP (optional):"));
    from_label.set_halign(gtk::Align::Start);
    let from_entry = Entry::new();
    from_entry.set_hexpand(true);
    from_entry.set_placeholder_text(Some("any or IP/subnet (e.g., 192.168.1.0/24)"));

    let comment_label = Label::new(Some("Comment:"));
    comment_label.set_halign(gtk::Align::Start);
    let comment_entry = Entry::new();
    comment_entry.set_hexpand(true);
    comment_entry.set_placeholder_text(Some("Description of this rule"));

    let add_rule_button = Button::with_label(&tr!("add_rule"));
    add_rule_button.set_halign(gtk::Align::Start);
    add_rule_button.set_css_classes(&["suggested-action"]);

    add_grid.attach(&action_label, 0, 0, 1, 1);
    add_grid.attach(&action_combo, 1, 0, 1, 1);
    add_grid.attach(&dir_label, 2, 0, 1, 1);
    add_grid.attach(&dir_combo, 3, 0, 1, 1);
    add_grid.attach(&port_label, 0, 1, 1, 1);
    add_grid.attach(&port_entry, 1, 1, 1, 1);
    add_grid.attach(&protocol_label, 2, 1, 1, 1);
    add_grid.attach(&protocol_combo, 3, 1, 1, 1);
    add_grid.attach(&from_label, 0, 2, 1, 1);
    add_grid.attach(&from_entry, 1, 2, 1, 1);
    add_grid.attach(&comment_label, 2, 2, 1, 1);
    add_grid.attach(&comment_entry, 3, 2, 1, 1);
    add_grid.attach(&add_rule_button, 0, 3, 4, 1);

    add_frame.set_child(Some(&add_grid));
    main_box.append(&add_frame);

    // ── Remove Rule ──

    let remove_frame = Frame::new(Some(tr!("delete_rule")));
    remove_frame.set_css_classes(&["card"]);

    let remove_grid = Grid::new();
    remove_grid.set_column_spacing(12);
    remove_grid.set_row_spacing(10);
    remove_grid.set_margin_top(10);
    remove_grid.set_margin_bottom(10);
    remove_grid.set_margin_start(12);
    remove_grid.set_margin_end(12);

    let remove_num_label = Label::new(Some("Rule Number:"));
    remove_num_label.set_halign(gtk::Align::Start);
    let remove_num_entry = Entry::new();
    remove_num_entry.set_hexpand(true);
    remove_num_entry.set_placeholder_text(Some("Enter rule number (e.g., 1, 2, 3)"));

    let remove_button = Button::with_label(&tr!("delete_rule"));
    remove_button.set_halign(gtk::Align::Start);
    remove_button.set_css_classes(&["destructive-action"]);

    let reset_button = Button::with_label(&tr!("ufw_reset"));
    reset_button.set_halign(gtk::Align::Start);
    reset_button.set_css_classes(&["destructive-action"]);

    remove_grid.attach(&remove_num_label, 0, 0, 1, 1);
    remove_grid.attach(&remove_num_entry, 1, 0, 2, 1);
    remove_grid.attach(&remove_button, 0, 1, 1, 1);
    remove_grid.attach(&reset_button, 1, 1, 1, 1);

    remove_frame.set_child(Some(&remove_grid));
    main_box.append(&remove_frame);

    // ── Rate Limiting ──

    let ratelimit_frame = Frame::new(Some("Rate Limiting"));
    ratelimit_frame.set_css_classes(&["card"]);

    let ratelimit_grid = Grid::new();
    ratelimit_grid.set_column_spacing(12);
    ratelimit_grid.set_row_spacing(10);
    ratelimit_grid.set_margin_top(10);
    ratelimit_grid.set_margin_bottom(10);
    ratelimit_grid.set_margin_start(12);
    ratelimit_grid.set_margin_end(12);

    let rl_port_label = Label::new(Some("Port:"));
    rl_port_label.set_halign(gtk::Align::Start);
    let rl_port_entry = Entry::new();
    rl_port_entry.set_hexpand(true);
    rl_port_entry.set_placeholder_text(Some("22"));
    rl_port_entry.set_text("22");

    let rl_enable_button = Button::with_label("Enable Rate Limiting");
    rl_enable_button.set_halign(gtk::Align::Start);
    rl_enable_button.set_css_classes(&["suggested-action"]);

    let rl_disable_button = Button::with_label("Disable Rate Limiting");
    rl_disable_button.set_halign(gtk::Align::Start);
    rl_disable_button.set_css_classes(&["destructive-action"]);

    ratelimit_grid.attach(&rl_port_label, 0, 0, 1, 1);
    ratelimit_grid.attach(&rl_port_entry, 1, 0, 1, 1);
    ratelimit_grid.attach(&rl_enable_button, 0, 1, 1, 1);
    ratelimit_grid.attach(&rl_disable_button, 1, 1, 1, 1);

    ratelimit_frame.set_child(Some(&ratelimit_grid));
    main_box.append(&ratelimit_frame);

    // ── Application Profiles ──

    let app_frame = Frame::new(Some(tr!("app_profiles")));
    app_frame.set_css_classes(&["card"]);

    let app_scrolled = ScrolledWindow::new();
    app_scrolled.set_min_content_height(120);
    app_scrolled.set_max_content_height(200);

    let app_text = TextView::new();
    app_text.set_editable(false);
    app_text.set_monospace(true);
    app_text.set_vexpand(true);
    app_text.buffer().set_text("Click 'Refresh Profiles' to load application profiles...");

    app_scrolled.set_child(Some(&app_text));
    app_scrolled.set_margin_top(10);
    app_scrolled.set_margin_bottom(10);
    app_scrolled.set_margin_start(12);
    app_scrolled.set_margin_end(12);

    let app_refresh_button = Button::with_label(&tr!("refresh"));
    app_refresh_button.set_halign(gtk::Align::Start);
    app_refresh_button.set_css_classes(&["suggested-action"]);

    let app_grid = Grid::new();
    app_grid.set_column_spacing(12);
    app_grid.set_row_spacing(10);
    app_grid.set_margin_top(10);
    app_grid.set_margin_bottom(10);
    app_grid.set_margin_start(12);
    app_grid.set_margin_end(12);

    app_grid.attach(&app_refresh_button, 0, 0, 1, 1);
    app_grid.attach(&app_scrolled, 0, 1, 1, 1);

    app_frame.set_child(Some(&app_grid));
    main_box.append(&app_frame);

    // ── Output Log ──

    let output_frame = Frame::new(Some("Operation Output"));
    output_frame.set_css_classes(&["card"]);

    let output_scrolled = ScrolledWindow::new();
    output_scrolled.set_min_content_height(120);
    output_scrolled.set_max_content_height(200);

    let output_text = TextView::new();
    output_text.set_editable(false);
    output_text.set_monospace(true);
    output_text.set_vexpand(true);
    output_text.buffer().set_text("Firewall operations will be logged here...");

    output_scrolled.set_child(Some(&output_text));
    output_scrolled.set_margin_top(10);
    output_scrolled.set_margin_bottom(10);
    output_scrolled.set_margin_start(12);
    output_scrolled.set_margin_end(12);

    output_frame.set_child(Some(&output_scrolled));
    main_box.append(&output_frame);

    // ── Signal handlers ──

    {
        let status_label = status_label.clone();
        let fw_switch = fw_switch.clone();
        let output_text = output_text.clone();

        fw_switch.connect_state_set(move |_, active| {
            let output_buffer = output_text.buffer();
            let status_label_c = status_label.clone();
            let active_c = active;
            spawn_bg(
                move || {
                    let result = if active_c {
                        run_ufw(&["--force", "enable"])
                    } else {
                        run_ufw(&["disable"])
                    };
                    let status = if is_ufw_active() { "ENABLED" } else { "DISABLED" };
                    (result, status, active_c)
                },
                move |(result, status, active_c)| {
                    status_label_c.set_text(&format!("Status: {}", status));
                    output_buffer.set_text(&format!(
                        "Firewall {}.\n\n{}",
                        if active_c { "enabled" } else { "disabled" },
                        result
                    ));
                },
            );
            glib::Propagation::Proceed
        });
    }

    {
        let status_label = status_label.clone();
        let rules_text = rules_text.clone();
        let fw_switch = fw_switch.clone();

        status_detail_button.connect_clicked(move |_| {
            let status_label_c = status_label.clone();
            let rules_text_c = rules_text.clone();
            let fw_switch_c = fw_switch.clone();
            spawn_bg(
                move || {
                    let status = get_ufw_status();
                    let active = is_ufw_active();
                    (status, active)
                },
                move |(status, active)| {
                    status_label_c.set_text(if active {
                        "Status: ACTIVE"
                    } else {
                        "Status: INACTIVE"
                    });
                    fw_switch_c.set_active(active);
                    rules_text_c.buffer().set_text(&status);
                },
            );
        });
    }

    {
        let rules_text = rules_text.clone();
        let action_combo = action_combo.clone();
        let dir_combo = dir_combo.clone();
        let port_entry = port_entry.clone();
        let protocol_combo = protocol_combo.clone();
        let from_entry = from_entry.clone();
        let comment_entry = comment_entry.clone();
        let output_text = output_text.clone();

        add_rule_button.connect_clicked(move |_| {
            let action = action_combo.active_text().unwrap_or_default();
            let dir = dir_combo.active_text().unwrap_or_default();
            let port = port_entry.text();
            let protocol = protocol_combo.active_text().unwrap_or_default();
            let from_ip = from_entry.text();
            let comment = comment_entry.text();

            if port.is_empty() {
                output_text.buffer().set_text("Error: Port/service cannot be empty.");
                return;
            }

            let action_str = action.to_lowercase();
            let dir_str = dir.to_lowercase();
            let port_str = port.to_string();
            let protocol_str = protocol.to_string();
            let from_ip_str = from_ip.to_string();
            let comment_str = comment.to_string();

            // ufw syntax: ufw [direction] action port/proto [from ...]
            let mut ufw_args = Vec::new();
            if dir_str != "both" {
                ufw_args.push(dir_str);
            }
            ufw_args.push(action_str);
            if protocol_str == "both" {
                ufw_args.push(port_str);
            } else {
                ufw_args.push(format!("{}/{}", port_str, protocol_str));
            }
            if !from_ip_str.is_empty() && from_ip_str != "any" {
                ufw_args.push("from".to_string());
                ufw_args.push(from_ip_str);
            }
            if !comment_str.is_empty() {
                ufw_args.push("comment".to_string());
                ufw_args.push(comment_str);
            }

            let cmd_str = ufw_args.join(" ");
            let output_buffer = output_text.buffer();
            let rules_text_c = rules_text.clone();
            spawn_bg(
                move || {
                    let result = run_ufw(&ufw_args.iter().map(|s| s.as_str()).collect::<Vec<&str>>());
                    let new_rules = get_ufw_rules();
                    (result, new_rules)
                },
                move |(result, new_rules)| {
                    output_buffer.set_text(&format!("Rule added.\n\nCommand: ufw {}\n\nResult:\n{}", cmd_str, result));
                    rules_text_c.buffer().set_text(&new_rules);
                },
            );
        });
    }

    {
        let remove_num_entry = remove_num_entry.clone();
        let rules_text = rules_text.clone();
        let output_text = output_text.clone();

        remove_button.connect_clicked(move |_| {
            let num = remove_num_entry.text().to_string();
            if num.is_empty() {
                output_text.buffer().set_text("Error: Enter a rule number to delete.");
                return;
            }
            let output_buffer = output_text.buffer();
            let rules_text_c = rules_text.clone();
            let num_c = num.clone();
            let num_cc = num_c.clone();
            spawn_bg(
                move || {
                    let result = run_ufw(&["--force", "delete", &num_c]);
                    let new_rules = get_ufw_rules();
                    (result, new_rules)
                },
                move |(result, new_rules)| {
                    output_buffer.set_text(&format!(
                        "Rule {} deleted.\n\nResult:\n{}",
                        num_cc, result
                    ));
                    rules_text_c.buffer().set_text(&new_rules);
                },
            );
        });
    }

    {
        let rules_text = rules_text.clone();
        let output_text = output_text.clone();

        reset_button.connect_clicked(move |_| {
            let output_buffer = output_text.buffer();
            let rules_text_c = rules_text.clone();
            spawn_bg(
                move || {
                    let result = run_ufw(&["--force", "reset"]);
                    let new_rules = get_ufw_rules();
                    (result, new_rules)
                },
                move |(result, new_rules)| {
                    output_buffer.set_text(&format!(
                        "All rules have been reset.\n\nResult:\n{}",
                        result
                    ));
                    rules_text_c.buffer().set_text(&new_rules);
                },
            );
        });
    }

    {
        let rl_port_entry = rl_port_entry.clone();
        let output_text = output_text.clone();
        let rules_text = rules_text.clone();

        rl_enable_button.connect_clicked(move |_| {
            let port = rl_port_entry.text().to_string();

            if port.is_empty() {
                output_text.buffer().set_text("Error: Port cannot be empty.");
                return;
            }

            let output_buffer = output_text.buffer();
            let rules_text_c = rules_text.clone();
            let port_c = port.clone();
            let port_cc = port_c.clone();
            spawn_bg(
                move || {
                    let result = run_ufw(&["limit", &port_c]);
                    let new_rules = get_ufw_rules();
                    (result, new_rules)
                },
                move |(result, new_rules)| {
                    output_buffer.set_text(&format!(
                        "Rate limiting enabled on port {} (ufw built-in rate limit).\n\nResult:\n{}",
                        port_cc, result
                    ));
                    rules_text_c.buffer().set_text(&new_rules);
                },
            );
        });
    }

    {
        let rl_port_entry = rl_port_entry.clone();
        let output_text = output_text.clone();
        let rules_text = rules_text.clone();

        rl_disable_button.connect_clicked(move |_| {
            let port = rl_port_entry.text().to_string();
            if port.is_empty() {
                output_text.buffer().set_text("Error: Enter the port to disable rate limiting on.");
                return;
            }
            let output_buffer = output_text.buffer();
            let rules_text_c = rules_text.clone();
            let port_c = port.clone();
            let port_cc = port_c.clone();
            spawn_bg(
                move || {
                    let result = run_ufw(&["--force", "delete", "limit", &port_c]);
                    let new_rules = get_ufw_rules();
                    (result, new_rules)
                },
                move |(result, new_rules)| {
                    output_buffer.set_text(&format!(
                        "Rate limiting disabled on port {}.\n\nResult:\n{}",
                        port_cc, result
                    ));
                    rules_text_c.buffer().set_text(&new_rules);
                },
            );
        });
    }

    {
        let app_text = app_text.clone();
        app_refresh_button.connect_clicked(move |_| {
            let buffer = app_text.buffer();
            buffer.set_text("Loading application profiles...");
            let app_text_c = app_text.clone();
            spawn_bg(
                move || get_ufw_app_profiles(),
                move |output| {
                    app_text_c.buffer().set_text(&output);
                },
            );
        });
    }

    // ── Auto-refresh on startup ──

    {
        let status_label = status_label.clone();
        let rules_text = rules_text.clone();
        let fw_switch = fw_switch.clone();
        spawn_bg(
            move || {
                let active = is_ufw_active();
                let rules = get_ufw_rules();
                (active, rules)
            },
            move |(active, rules)| {
                status_label.set_text(if active { "Status: ACTIVE" } else { "Status: INACTIVE" });
                fw_switch.set_active(active);
                rules_text.buffer().set_text(&rules);
            },
        );
    }

    main_box.upcast()
}
