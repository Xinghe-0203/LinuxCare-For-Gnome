use crate::tr;
use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Box, Button, Entry, Frame, Grid, Label, Orientation, ScrolledWindow, Switch, TextView,
};

fn run_cmd(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|e| format!("Error: {}", e))
}

fn run_cmd_err(cmd: &str, args: &[&str]) -> (bool, String) {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            (o.status.success(), if stderr.is_empty() { stdout } else { stderr })
        })
        .unwrap_or_else(|e| (false, format!("Error: {}", e)));
    output
}

fn get_current_user_info() -> String {
    let whoami = run_cmd("whoami", &[]);
    let id_out = run_cmd("id", &[]);
    let hostname = run_cmd("hostname", &[]);
    let home = run_cmd("getent", &["passwd", whoami.trim()])
        .lines()
        .next()
        .and_then(|l| l.split(':').nth(5))
        .unwrap_or("unknown")
        .to_string();
    let shell = run_cmd("getent", &["passwd", whoami.trim()])
        .lines()
        .next()
        .and_then(|l| l.split(':').nth(6))
        .unwrap_or("unknown")
        .to_string();

    format!(
        "Username:     {}\nHostname:     {}\nHome Dir:     {}\nShell:        {}\n{}",
        whoami.trim(),
        hostname.trim(),
        home,
        shell,
        id_out.trim()
    )
}

fn get_user_list() -> String {
    let output = run_cmd("getent", &["passwd"]);
    let header = format!(
        "{:<16} {:>8} {:>8} {:<30} {:<20}",
        "USERNAME", "UID", "GID", "HOME", "SHELL"
    );
    let separator = "─".repeat(86);

    let mut lines = vec![header, separator];
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 7 {
            let username = parts[0];
            let uid = parts[2];
            let gid = parts[3];
            let home = parts[5];
            let shell = parts[6];
            lines.push(format!(
                "{:<16} {:>8} {:>8} {:<30} {:<20}",
                username, uid, gid, home, shell
            ));
        }
    }

    if lines.len() == 2 {
        lines.push("No users found.".to_string());
    }

    lines.join("\n")
}

fn get_group_list() -> String {
    let output = run_cmd("getent", &["group"]);
    let header = format!("{:<24} {:>8} {}", "GROUP", "GID", "MEMBERS");
    let separator = "─".repeat(80);

    let mut lines = vec![header, separator];
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            let group = parts[0];
            let gid = parts[2];
            let members = if parts[3].is_empty() {
                "(none)".to_string()
            } else {
                parts[3].to_string()
            };
            lines.push(format!("{:<24} {:>8} {}", group, gid, members));
        }
    }

    if lines.len() == 2 {
        lines.push("No groups found.".to_string());
    }

    lines.join("\n")
}

fn get_logged_in_users() -> String {
    let output = run_cmd("who", &[]);
    let header = format!("{:<16} {:<12} {:<20} {}", "USER", "TERMINAL", "FROM", "TIME");
    let separator = "─".repeat(70);

    let mut lines = vec![header, separator];
    for line in output.lines() {
        if !line.trim().is_empty() {
            lines.push(line.to_string());
        }
    }

    if lines.len() == 2 {
        lines.push("No users currently logged in.".to_string());
    }

    lines.join("\n")
}

pub fn build_user_page() -> gtk::Widget {
    let main_box = Box::new(Orientation::Vertical, 20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let title = Label::new(Some(tr!("user_management")));
    title.set_css_classes(&["title-1"]);
    main_box.append(&title);

    // ── Current User Info ──

    let current_user_frame = Frame::new(Some(tr!("current_user_info")));
    current_user_frame.set_css_classes(&["card"]);

    let current_user_scrolled = ScrolledWindow::new();
    current_user_scrolled.set_min_content_height(120);
    current_user_scrolled.set_max_content_height(200);

    let current_user_text = TextView::new();
    current_user_text.set_editable(false);
    current_user_text.set_monospace(true);
    current_user_text.set_wrap_mode(gtk::WrapMode::WordChar);
    current_user_text.set_left_margin(8);
    current_user_text.set_top_margin(8);

    let current_user_buffer = current_user_text.buffer();
    current_user_buffer.set_text(&get_current_user_info());

    current_user_scrolled.set_child(Some(&current_user_text));
    current_user_scrolled.set_margin_top(10);
    current_user_scrolled.set_margin_bottom(10);
    current_user_scrolled.set_margin_start(12);
    current_user_scrolled.set_margin_end(12);

    current_user_frame.set_child(Some(&current_user_scrolled));
    main_box.append(&current_user_frame);

    // ── User List ──

    let user_list_frame = Frame::new(Some(tr!("system_users")));
    user_list_frame.set_css_classes(&["card"]);

    let user_list_scrolled = ScrolledWindow::new();
    user_list_scrolled.set_min_content_height(200);
    user_list_scrolled.set_vexpand(true);

    let user_list_text = TextView::new();
    user_list_text.set_editable(false);
    user_list_text.set_monospace(true);
    user_list_text.set_wrap_mode(gtk::WrapMode::None);
    user_list_text.set_left_margin(8);
    user_list_text.set_top_margin(8);

    let user_list_buffer = user_list_text.buffer();
    user_list_buffer.set_text(&get_user_list());

    user_list_scrolled.set_child(Some(&user_list_text));
    user_list_scrolled.set_margin_top(10);
    user_list_scrolled.set_margin_bottom(10);
    user_list_scrolled.set_margin_start(12);
    user_list_scrolled.set_margin_end(12);

    user_list_frame.set_child(Some(&user_list_scrolled));
    main_box.append(&user_list_frame);

    // ── Add User ──

    let add_user_frame = Frame::new(Some(tr!("add_user")));
    add_user_frame.set_css_classes(&["card"]);

    let add_user_grid = Grid::new();
    add_user_grid.set_column_spacing(12);
    add_user_grid.set_row_spacing(10);
    add_user_grid.set_margin_top(10);
    add_user_grid.set_margin_bottom(10);
    add_user_grid.set_margin_start(12);
    add_user_grid.set_margin_end(12);

    let add_username_label = Label::new(Some(tr!("username")));
    add_username_label.set_halign(gtk::Align::Start);
    let add_username_entry = Entry::new();
    add_username_entry.set_hexpand(true);
    add_username_entry.set_placeholder_text(Some("newuser"));

    let add_fullname_label = Label::new(Some(tr!("full_name")));
    add_fullname_label.set_halign(gtk::Align::Start);
    let add_fullname_entry = Entry::new();
    add_fullname_entry.set_hexpand(true);
    add_fullname_entry.set_placeholder_text(Some("John Doe"));

    let add_shell_label = Label::new(Some(tr!("shell")));
    add_shell_label.set_halign(gtk::Align::Start);
    let add_shell_entry = Entry::new();
    add_shell_entry.set_hexpand(true);
    add_shell_entry.set_text("/bin/bash");
    add_shell_entry.set_placeholder_text(Some("/bin/bash"));

    let add_home_label = Label::new(Some(tr!("create_home_dir")));
    add_home_label.set_halign(gtk::Align::Start);
    let add_home_switch = Switch::new();
    add_home_switch.set_active(true);
    add_home_switch.set_halign(gtk::Align::Start);

    let add_system_label = Label::new(Some(tr!("system_account")));
    add_system_label.set_halign(gtk::Align::Start);
    let add_system_switch = Switch::new();
    add_system_switch.set_active(false);
    add_system_switch.set_halign(gtk::Align::Start);

    let add_user_button = Button::with_label(tr!("create_user"));
    add_user_button.set_css_classes(&["suggested-action"]);

    let add_user_status = Label::new(Some(""));
    add_user_status.set_halign(gtk::Align::Start);
    add_user_status.set_wrap(true);

    add_user_grid.attach(&add_username_label, 0, 0, 1, 1);
    add_user_grid.attach(&add_username_entry, 1, 0, 2, 1);
    add_user_grid.attach(&add_fullname_label, 0, 1, 1, 1);
    add_user_grid.attach(&add_fullname_entry, 1, 1, 2, 1);
    add_user_grid.attach(&add_shell_label, 0, 2, 1, 1);
    add_user_grid.attach(&add_shell_entry, 1, 2, 2, 1);
    add_user_grid.attach(&add_home_label, 0, 3, 1, 1);
    add_user_grid.attach(&add_home_switch, 1, 3, 1, 1);
    add_user_grid.attach(&add_system_label, 0, 4, 1, 1);
    add_user_grid.attach(&add_system_switch, 1, 4, 1, 1);
    add_user_grid.attach(&add_user_button, 0, 5, 3, 1);
    add_user_grid.attach(&add_user_status, 0, 6, 3, 1);

    add_user_frame.set_child(Some(&add_user_grid));
    main_box.append(&add_user_frame);

    // ── Modify User ──

    let modify_user_frame = Frame::new(Some(tr!("modify_user")));
    modify_user_frame.set_css_classes(&["card"]);

    let modify_user_grid = Grid::new();
    modify_user_grid.set_column_spacing(12);
    modify_user_grid.set_row_spacing(10);
    modify_user_grid.set_margin_top(10);
    modify_user_grid.set_margin_bottom(10);
    modify_user_grid.set_margin_start(12);
    modify_user_grid.set_margin_end(12);

    let mod_target_label = Label::new(Some(tr!("username")));
    mod_target_label.set_halign(gtk::Align::Start);
    let mod_target_entry = Entry::new();
    mod_target_entry.set_hexpand(true);
    mod_target_entry.set_placeholder_text(Some("username to modify"));

    let mod_shell_label = Label::new(Some(tr!("new_shell")));
    mod_shell_label.set_halign(gtk::Align::Start);
    let mod_shell_entry = Entry::new();
    mod_shell_entry.set_hexpand(true);
    mod_shell_entry.set_placeholder_text(Some("/bin/zsh"));

    let mod_home_label = Label::new(Some(tr!("new_home_dir")));
    mod_home_label.set_halign(gtk::Align::Start);
    let mod_home_entry = Entry::new();
    mod_home_entry.set_hexpand(true);
    mod_home_entry.set_placeholder_text(Some("/home/newdir"));

    let mod_fullname_label = Label::new(Some(tr!("new_full_name")));
    mod_fullname_label.set_halign(gtk::Align::Start);
    let mod_fullname_entry = Entry::new();
    mod_fullname_entry.set_hexpand(true);
    mod_fullname_entry.set_placeholder_text(Some("New Name"));

    let mod_groups_label = Label::new(Some(tr!("groups")));
    mod_groups_label.set_halign(gtk::Align::Start);
    let mod_groups_entry = Entry::new();
    mod_groups_entry.set_hexpand(true);
    mod_groups_entry.set_placeholder_text(Some("sudo,docker,adm"));

    let mod_move_home_label = Label::new(Some(tr!("move_home")));
    mod_move_home_label.set_halign(gtk::Align::Start);
    let mod_move_home_switch = Switch::new();
    mod_move_home_switch.set_active(false);
    mod_move_home_switch.set_halign(gtk::Align::Start);

    let mod_locked_label = Label::new(Some(tr!("lock_account")));
    mod_locked_label.set_halign(gtk::Align::Start);
    let mod_locked_switch = Switch::new();
    mod_locked_switch.set_active(false);
    mod_locked_switch.set_halign(gtk::Align::Start);

    let mod_apply_button = Button::with_label(tr!("apply_changes"));
    mod_apply_button.set_css_classes(&["suggested-action"]);

    let mod_unlock_button = Button::with_label(tr!("unlock_account"));
    mod_unlock_button.set_halign(gtk::Align::Start);

    let mod_delete_button = Button::with_label(tr!("delete_user"));
    mod_delete_button.set_css_classes(&["destructive-action"]);
    mod_delete_button.set_halign(gtk::Align::Start);

    let mod_status = Label::new(Some(""));
    mod_status.set_halign(gtk::Align::Start);
    mod_status.set_wrap(true);

    modify_user_grid.attach(&mod_target_label, 0, 0, 1, 1);
    modify_user_grid.attach(&mod_target_entry, 1, 0, 2, 1);
    modify_user_grid.attach(&mod_shell_label, 0, 1, 1, 1);
    modify_user_grid.attach(&mod_shell_entry, 1, 1, 2, 1);
    modify_user_grid.attach(&mod_home_label, 0, 2, 1, 1);
    modify_user_grid.attach(&mod_home_entry, 1, 2, 2, 1);
    modify_user_grid.attach(&mod_fullname_label, 0, 3, 1, 1);
    modify_user_grid.attach(&mod_fullname_entry, 1, 3, 2, 1);
    modify_user_grid.attach(&mod_groups_label, 0, 4, 1, 1);
    modify_user_grid.attach(&mod_groups_entry, 1, 4, 2, 1);
    modify_user_grid.attach(&mod_move_home_label, 0, 5, 1, 1);
    modify_user_grid.attach(&mod_move_home_switch, 1, 5, 1, 1);
    modify_user_grid.attach(&mod_locked_label, 0, 6, 1, 1);
    modify_user_grid.attach(&mod_locked_switch, 1, 6, 1, 1);
    modify_user_grid.attach(&mod_apply_button, 0, 7, 1, 1);
    modify_user_grid.attach(&mod_unlock_button, 1, 7, 1, 1);
    modify_user_grid.attach(&mod_delete_button, 2, 7, 1, 1);
    modify_user_grid.attach(&mod_status, 0, 8, 3, 1);

    modify_user_frame.set_child(Some(&modify_user_grid));
    main_box.append(&modify_user_frame);

    // ── Group Management ──

    let group_frame = Frame::new(Some(tr!("group_management")));
    group_frame.set_css_classes(&["card"]);

    let group_inner_box = Box::new(Orientation::Vertical, 10);
    group_inner_box.set_margin_top(10);
    group_inner_box.set_margin_bottom(10);
    group_inner_box.set_margin_start(12);
    group_inner_box.set_margin_end(12);

    let group_list_scrolled = ScrolledWindow::new();
    group_list_scrolled.set_min_content_height(180);
    group_list_scrolled.set_max_content_height(300);

    let group_list_text = TextView::new();
    group_list_text.set_editable(false);
    group_list_text.set_monospace(true);
    group_list_text.set_wrap_mode(gtk::WrapMode::None);
    group_list_text.set_left_margin(8);
    group_list_text.set_top_margin(8);

    let group_list_buffer = group_list_text.buffer();
    group_list_buffer.set_text(&get_group_list());

    group_list_scrolled.set_child(Some(&group_list_text));
    group_inner_box.append(&group_list_scrolled);

    let group_action_grid = Grid::new();
    group_action_grid.set_column_spacing(12);
    group_action_grid.set_row_spacing(10);

    let grp_add_label = Label::new(Some(tr!("new_group")));
    grp_add_label.set_halign(gtk::Align::Start);
    let grp_add_entry = Entry::new();
    grp_add_entry.set_hexpand(true);
    grp_add_entry.set_placeholder_text(Some("groupname"));

    let grp_add_button = Button::with_label(tr!("add_group"));
    grp_add_button.set_css_classes(&["suggested-action"]);

    let grp_add_user_label = Label::new(Some(tr!("add_user_to_group")));
    grp_add_user_label.set_halign(gtk::Align::Start);
    let grp_add_user_entry = Entry::new();
    grp_add_user_entry.set_hexpand(true);
    grp_add_user_entry.set_placeholder_text(Some("username"));

    let grp_add_to_button = Button::with_label(tr!("add_to_group"));
    grp_add_to_button.set_css_classes(&["suggested-action"]);

    let grp_add_to_group_label = Label::new(Some(tr!("groups")));
    grp_add_to_group_label.set_halign(gtk::Align::Start);
    let grp_add_to_group_entry = Entry::new();
    grp_add_to_group_entry.set_hexpand(true);
    grp_add_to_group_entry.set_placeholder_text(Some("groupname"));

    let grp_remove_label = Label::new(Some(tr!("remove_from_group")));
    grp_remove_label.set_halign(gtk::Align::Start);
    let grp_remove_user_entry = Entry::new();
    grp_remove_user_entry.set_hexpand(true);
    let grp_remove_group_entry = Entry::new();
    grp_remove_group_entry.set_hexpand(true);
    grp_remove_group_entry.set_placeholder_text(Some("groupname"));

    let grp_remove_button = Button::with_label(tr!("remove_from_group"));
    grp_remove_button.set_css_classes(&["destructive-action"]);

    let grp_status = Label::new(Some(""));
    grp_status.set_halign(gtk::Align::Start);
    grp_status.set_wrap(true);

    group_action_grid.attach(&grp_add_label, 0, 0, 1, 1);
    group_action_grid.attach(&grp_add_entry, 1, 0, 2, 1);
    group_action_grid.attach(&grp_add_button, 3, 0, 1, 1);
    group_action_grid.attach(&grp_add_user_label, 0, 1, 1, 1);
    group_action_grid.attach(&grp_add_user_entry, 1, 1, 1, 1);
    group_action_grid.attach(&grp_add_to_group_label, 2, 1, 1, 1);
    group_action_grid.attach(&grp_add_to_group_entry, 3, 1, 1, 1);
    group_action_grid.attach(&grp_add_to_button, 4, 1, 1, 1);
    group_action_grid.attach(&grp_remove_label, 0, 2, 1, 1);
    group_action_grid.attach(&grp_remove_user_entry, 1, 2, 1, 1);
    group_action_grid.attach(&grp_remove_group_entry, 2, 2, 1, 1);
    group_action_grid.attach(&grp_remove_button, 3, 2, 1, 1);
    group_action_grid.attach(&grp_status, 0, 3, 5, 1);

    group_inner_box.append(&group_action_grid);

    group_frame.set_child(Some(&group_inner_box));
    main_box.append(&group_frame);

    // ── User Sessions ──

    let sessions_frame = Frame::new(Some(tr!("user_sessions")));
    sessions_frame.set_css_classes(&["card"]);

    let sessions_scrolled = ScrolledWindow::new();
    sessions_scrolled.set_min_content_height(150);
    sessions_scrolled.set_max_content_height(250);

    let sessions_text = TextView::new();
    sessions_text.set_editable(false);
    sessions_text.set_monospace(true);
    sessions_text.set_wrap_mode(gtk::WrapMode::None);
    sessions_text.set_left_margin(8);
    sessions_text.set_top_margin(8);

    let sessions_buffer = sessions_text.buffer();
    sessions_buffer.set_text(&get_logged_in_users());

    sessions_scrolled.set_child(Some(&sessions_text));
    sessions_scrolled.set_margin_top(10);
    sessions_scrolled.set_margin_bottom(10);
    sessions_scrolled.set_margin_start(12);
    sessions_scrolled.set_margin_end(12);

    sessions_frame.set_child(Some(&sessions_scrolled));
    main_box.append(&sessions_frame);

    // ── Refresh Button ──

    let refresh_frame = Frame::new(Some(tr!("actions")));
    refresh_frame.set_css_classes(&["card"]);

    let refresh_grid = Grid::new();
    refresh_grid.set_column_spacing(12);
    refresh_grid.set_row_spacing(10);
    refresh_grid.set_margin_top(10);
    refresh_grid.set_margin_bottom(10);
    refresh_grid.set_margin_start(12);
    refresh_grid.set_margin_end(12);

    let refresh_button = Button::with_label(tr!("refresh_all"));
    refresh_button.set_halign(gtk::Align::Start);
    refresh_button.set_css_classes(&["suggested-action"]);

    let passwd_button = Button::with_label(tr!("change_password"));
    passwd_button.set_halign(gtk::Align::Start);

    let chsh_button = Button::with_label(tr!("change_shell"));
    chsh_button.set_halign(gtk::Align::Start);

    refresh_grid.attach(&refresh_button, 0, 0, 1, 1);
    refresh_grid.attach(&passwd_button, 1, 0, 1, 1);
    refresh_grid.attach(&chsh_button, 2, 0, 1, 1);

    refresh_frame.set_child(Some(&refresh_grid));
    main_box.append(&refresh_frame);

    // ── Event Handlers ──

    // Add User
    {
        let add_username_entry = add_username_entry.clone();
        let add_fullname_entry = add_fullname_entry.clone();
        let add_shell_entry = add_shell_entry.clone();
        let add_home_switch = add_home_switch.clone();
        let add_system_switch = add_system_switch.clone();
        let add_user_status = add_user_status.clone();
        let user_list_buffer = user_list_buffer.clone();
        let current_user_buffer = current_user_buffer.clone();
        let sessions_buffer = sessions_buffer.clone();

        add_user_button.connect_clicked(move |_| {
            let username = add_username_entry.text().to_string();
            if username.is_empty() {
                add_user_status.set_text("Error: Username cannot be empty.");
                return;
            }

            let mut args = vec!["useradd"];

            if add_home_switch.is_active() {
                args.push("-m");
            } else {
                args.push("-M");
            }

            let shell = add_shell_entry.text().to_string();
            if !shell.is_empty() {
                args.push("-s");
                args.push(&shell);
            }

            let fullname = add_fullname_entry.text().to_string();
            if !fullname.is_empty() {
                args.push("-c");
                args.push(&fullname);
            }

            if add_system_switch.is_active() {
                args.push("-r");
            }

            args.push(&username);

            let (success, msg) = run_cmd_err("sudo", &args);
            if success {
                add_user_status.set_text(&format!("User '{}' created successfully.", username));
                add_username_entry.set_text("");
                add_fullname_entry.set_text("");
                user_list_buffer.set_text(&get_user_list());
                current_user_buffer.set_text(&get_current_user_info());
                sessions_buffer.set_text(&get_logged_in_users());
            } else {
                add_user_status.set_text(&format!("Failed to create user: {}", msg));
            }
        });
    }

    // Modify User
    {
        let mod_target_entry = mod_target_entry.clone();
        let mod_shell_entry = mod_shell_entry.clone();
        let mod_home_entry = mod_home_entry.clone();
        let mod_fullname_entry = mod_fullname_entry.clone();
        let mod_groups_entry = mod_groups_entry.clone();
        let mod_move_home_switch = mod_move_home_switch.clone();
        let mod_locked_switch = mod_locked_switch.clone();
        let mod_status = mod_status.clone();
        let user_list_buffer = user_list_buffer.clone();

        mod_apply_button.connect_clicked(move |_| {
            let username = mod_target_entry.text().to_string();
            if username.is_empty() {
                mod_status.set_text("Error: Username cannot be empty.");
                return;
            }

            let mut any_change = false;

            let shell = mod_shell_entry.text().to_string();
            if !shell.is_empty() {
                run_cmd_err("sudo", &["usermod", "-s", &shell, &username]);
                any_change = true;
            }

            let fullname = mod_fullname_entry.text().to_string();
            if !fullname.is_empty() {
                run_cmd_err("sudo", &["usermod", "-c", &fullname, &username]);
                any_change = true;
            }

            let home = mod_home_entry.text().to_string();
            if !home.is_empty() {
                let mut args = vec!["usermod", "-d", &home];
                if mod_move_home_switch.is_active() {
                    args.push("-m");
                }
                args.push(&username);
                run_cmd_err("sudo", &args);
                any_change = true;
            }

            let groups = mod_groups_entry.text().to_string();
            if !groups.is_empty() {
                run_cmd_err("sudo", &["usermod", "-aG", &groups, &username]);
                any_change = true;
            }

            if mod_locked_switch.is_active() {
                run_cmd_err("sudo", &["usermod", "-L", &username]);
                any_change = true;
            }

            if any_change {
                mod_status.set_text(&format!("Changes applied to '{}'.", username));
                user_list_buffer.set_text(&get_user_list());
            } else {
                mod_status.set_text("No changes specified.");
            }
        });
    }

    // Unlock User
    {
        let mod_target_entry = mod_target_entry.clone();
        let mod_status = mod_status.clone();
        let mod_locked_switch = mod_locked_switch.clone();

        mod_unlock_button.connect_clicked(move |_| {
            let username = mod_target_entry.text().to_string();
            if username.is_empty() {
                mod_status.set_text("Error: Enter a username to unlock.");
                return;
            }

            let (success, msg) = run_cmd_err("sudo", &["usermod", "-U", &username]);
            if success {
                mod_status.set_text(&format!("User '{}' unlocked.", username));
                mod_locked_switch.set_active(false);
            } else {
                mod_status.set_text(&format!("Failed to unlock: {}", msg));
            }
        });
    }

    // Delete User
    {
        let mod_target_entry = mod_target_entry.clone();
        let mod_status = mod_status.clone();
        let user_list_buffer = user_list_buffer.clone();
        let current_user_buffer = current_user_buffer.clone();
        let sessions_buffer = sessions_buffer.clone();

        mod_delete_button.connect_clicked(move |_| {
            let username = mod_target_entry.text().to_string();
            if username.is_empty() {
                mod_status.set_text("Error: Enter a username to delete.");
                return;
            }

            let dialog = gtk::MessageDialog::builder()
                .modal(true)
                .message_type(gtk::MessageType::Warning)
                .text("Confirm User Deletion")
                .secondary_text(&format!(
                    "Are you sure you want to delete user '{}' and their home directory?\n\nThis action cannot be undone.",
                    username
                ))
                .buttons(gtk::ButtonsType::YesNo)
                .build();
            let mod_target_entry = mod_target_entry.clone();
            let mod_status = mod_status.clone();
            let user_list_buffer = user_list_buffer.clone();
            let current_user_buffer = current_user_buffer.clone();
            let sessions_buffer = sessions_buffer.clone();
            let username_c = username.clone();
            dialog.connect_response(move |dlg, response| {
                dlg.close();
                if response == gtk::ResponseType::Yes {
                    let (success, msg) = run_cmd_err("sudo", &["userdel", "-r", &username_c]);
                    if success {
                        mod_status.set_text(&format!("User '{}' deleted.", username_c));
                        mod_target_entry.set_text("");
                        user_list_buffer.set_text(&get_user_list());
                        current_user_buffer.set_text(&get_current_user_info());
                        sessions_buffer.set_text(&get_logged_in_users());
                    } else {
                        mod_status.set_text(&format!("Failed to delete user: {}", msg));
                    }
                }
            });
        });
    }

    // Add Group
    {
        let grp_add_entry = grp_add_entry.clone();
        let grp_status = grp_status.clone();
        let group_list_buffer = group_list_buffer.clone();

        grp_add_button.connect_clicked(move |_| {
            let groupname = grp_add_entry.text().to_string();
            if groupname.is_empty() {
                grp_status.set_text("Error: Group name cannot be empty.");
                return;
            }

            let (success, msg) = run_cmd_err("sudo", &["groupadd", &groupname]);
            if success {
                grp_status.set_text(&format!("Group '{}' created.", groupname));
                grp_add_entry.set_text("");
                group_list_buffer.set_text(&get_group_list());
            } else {
                grp_status.set_text(&format!("Failed to create group: {}", msg));
            }
        });
    }

    // Add User to Group
    {
        let grp_add_user_entry = grp_add_user_entry.clone();
        let grp_add_to_group_entry = grp_add_to_group_entry.clone();
        let grp_status = grp_status.clone();
        let group_list_buffer = group_list_buffer.clone();

        grp_add_to_button.connect_clicked(move |_| {
            let username = grp_add_user_entry.text().to_string();
            let groupname = grp_add_to_group_entry.text().to_string();
            if username.is_empty() || groupname.is_empty() {
                grp_status.set_text("Error: Both username and group name are required.");
                return;
            }

            let (success, msg) =
                run_cmd_err("sudo", &["usermod", "-aG", &groupname, &username]);
            if success {
                grp_status.set_text(&format!(
                    "User '{}' added to group '{}'.",
                    username, groupname
                ));
                grp_add_user_entry.set_text("");
                grp_add_to_group_entry.set_text("");
                group_list_buffer.set_text(&get_group_list());
            } else {
                grp_status.set_text(&format!("Failed to add user to group: {}", msg));
            }
        });
    }

    // Remove User from Group
    {
        let grp_remove_user_entry = grp_remove_user_entry.clone();
        let grp_remove_group_entry = grp_remove_group_entry.clone();
        let grp_status = grp_status.clone();
        let group_list_buffer = group_list_buffer.clone();

        grp_remove_button.connect_clicked(move |_| {
            let username = grp_remove_user_entry.text().to_string();
            let groupname = grp_remove_group_entry.text().to_string();
            if username.is_empty() || groupname.is_empty() {
                grp_status.set_text("Error: Both username and group name are required.");
                return;
            }

            let (success, msg) =
                run_cmd_err("sudo", &["gpasswd", "-d", &username, &groupname]);
            if success {
                grp_status.set_text(&format!(
                    "User '{}' removed from group '{}'.",
                    username, groupname
                ));
                grp_remove_user_entry.set_text("");
                grp_remove_group_entry.set_text("");
                group_list_buffer.set_text(&get_group_list());
            } else {
                grp_status.set_text(&format!("Failed to remove user from group: {}", msg));
            }
        });
    }

    // Refresh
    {
        let current_user_buffer = current_user_buffer.clone();
        let user_list_buffer = user_list_buffer.clone();
        let group_list_buffer = group_list_buffer.clone();
        let sessions_buffer = sessions_buffer.clone();

        refresh_button.connect_clicked(move |_| {
            current_user_buffer.set_text(&get_current_user_info());
            user_list_buffer.set_text(&get_user_list());
            group_list_buffer.set_text(&get_group_list());
            sessions_buffer.set_text(&get_logged_in_users());
        });
    }

    // Change Password (launch passwd)
    {
        passwd_button.connect_clicked(move |_| {
            let _ = std::process::Command::new("gnome-terminal").args(["--", "passwd"]).spawn();
        });
    }

    // Change Shell (launch chsh)
    {
        chsh_button.connect_clicked(move |_| {
            let _ = std::process::Command::new("gnome-terminal").args(["--", "chsh"]).spawn();
        });
    }

    // ── Periodic refresh for sessions ──

    {
        let sessions_buffer_c = sessions_buffer.clone();
        glib::timeout_add_seconds_local(15, move || {
            sessions_buffer_c.set_text(&get_logged_in_users());
            glib::ControlFlow::Continue
        });
    }

    main_box.upcast()
}
