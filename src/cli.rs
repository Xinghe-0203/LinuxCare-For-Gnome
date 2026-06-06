mod utils;

use std::env;
use std::process;
use utils::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    match args[1].as_str() {
        "desktop" => handle_desktop(),
        "system" => handle_system(),
        "software" => handle_software(&args[2..]),
        "disk" => handle_disk(),
        "cleanup" => handle_cleanup(&args[2..]),
        "menu" => handle_menu(),
        "shortcut" => handle_shortcut(),
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-v" => print_version(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("LinuxCare - Linux System Management Tool");
    println!();
    println!("Usage: linuxcare-cli <command> [options]");
    println!();
    println!("Commands:");
    println!("  desktop     Desktop management (icons, wallpaper, workspaces)");
    println!("  system      System settings");
    println!("  software    Software management (list, search, install, remove)");
    println!("  disk        Disk management and usage");
    println!("  cleanup     System cleanup (cache, logs, temp files)");
    println!("  menu        Menu management");
    println!("  shortcut    Shortcut management");
    println!("  help        Show this help message");
    println!("  version     Show version information");
    println!();
    println!("Examples:");
    println!("  linuxcare-cli software list");
    println!("  linuxcare-cli software search firefox");
    println!("  linuxcare-cli cleanup apt");
    println!("  linuxcare-cli disk usage");
}

fn print_version() {
    println!("LinuxCare v0.1.0");
}

fn handle_desktop() {
    println!("Desktop Management");
    println!("=================");
    println!();
    println!("This command provides desktop management functionality.");
    println!("Use the GUI version for full desktop management features.");
    println!();
    println!("Available options:");
    println!("  --show-icons     Show desktop icons");
    println!("  --hide-icons     Hide desktop icons");
    println!("  --set-wallpaper  Set wallpaper (requires file path)");
}

fn handle_system() {
    println!("System Settings");
    println!("===============");
    println!();
    
    // Show system information
    println!("System Information:");
    println!("------------------");
    
    if let Ok(output) = run_command("uname", &["-a"]) {
        println!("Kernel: {}", output.trim());
    }
    
    if let Ok(output) = run_command("lsb_release", &["-d"]) {
        println!("Distribution: {}", output.trim());
    }
    
    if let Ok(output) = run_command("free", &["-h"]) {
        println!();
        println!("Memory:");
        println!("{}", output);
    }
    
    if let Ok(output) = run_command("lscpu", &[]) {
        println!();
        println!("CPU:");
        for line in output.lines().take(13) {
            println!("{}", line);
        }
    }
}

fn handle_software(args: &[String]) {
    if args.is_empty() {
        println!("Software Management");
        println!("==================");
        println!();
        println!("Usage: linuxcare-cli software <action> [package]");
        println!();
        println!("Actions:");
        println!("  list              List installed packages");
        println!("  search <query>    Search for packages");
        println!("  install <pkg>     Install a package");
        println!("  remove <pkg>      Remove a package");
        println!("  update            Update package list");
        println!("  upgrade           Upgrade all packages");
        return;
    }
    
    match args[0].as_str() {
        "list" => {
            println!("Installed Packages:");
            println!("------------------");
            
            if command_exists("dpkg") {
                println!();
                println!("APT Packages:");
                if let Ok(packages) = get_apt_packages() {
                    for pkg in packages.iter().take(20) {
                        println!("  {} ({})", pkg.name, pkg.version);
                    }
                    println!("  ... and {} more", packages.len().saturating_sub(20));
                }
            }
            
            if command_exists("flatpak") {
                println!();
                println!("Flatpak Packages:");
                if let Ok(packages) = get_flatpak_packages() {
                    for pkg in packages.iter().take(20) {
                        println!("  {} ({})", pkg.name, pkg.version);
                    }
                    println!("  ... and {} more", packages.len().saturating_sub(20));
                }
            }
            
            if command_exists("snap") {
                println!();
                println!("Snap Packages:");
                if let Ok(packages) = get_snap_packages() {
                    for pkg in packages.iter().take(20) {
                        println!("  {} ({})", pkg.name, pkg.version);
                    }
                    println!("  ... and {} more", packages.len().saturating_sub(20));
                }
            }
        }
        "search" => {
            if args.len() < 2 {
                eprintln!("Error: Please provide a search query");
                process::exit(1);
            }
            
            let query = &args[1];
            println!("Searching for: {}", query);
            println!();
            
            if command_exists("apt") {
                println!("APT Results:");
                if let Ok(output) = run_command("apt", &["search", query]) {
                    for line in output.lines().take(10) {
                        println!("  {}", line);
                    }
                }
            }
        }
        "install" => {
            if args.len() < 2 {
                eprintln!("Error: Please provide a package name");
                process::exit(1);
            }
            
            let pkg = &args[1];
            println!("Installing: {}", pkg);
            
            if command_exists("apt") {
                if let Err(e) = run_command("sudo", &["apt", "install", "-y", pkg]) {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            } else if command_exists("flatpak") {
                if let Err(e) = run_command("flatpak", &["install", "-y", pkg]) {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
            
            println!("Package installed successfully!");
        }
        "remove" => {
            if args.len() < 2 {
                eprintln!("Error: Please provide a package name");
                process::exit(1);
            }
            
            let pkg = &args[1];
            println!("Removing: {}", pkg);
            
            if command_exists("apt") {
                if let Err(e) = run_command("sudo", &["apt", "remove", "-y", pkg]) {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            } else if command_exists("flatpak") {
                if let Err(e) = run_command("flatpak", &["uninstall", "-y", pkg]) {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
            
            println!("Package removed successfully!");
        }
        "update" => {
            println!("Updating package list...");
            
            if command_exists("apt") {
                if let Err(e) = run_command("sudo", &["apt", "update"]) {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
            
            println!("Package list updated!");
        }
        "upgrade" => {
            println!("Upgrading all packages...");
            
            if command_exists("apt") {
                if let Err(e) = run_command("sudo", &["apt", "upgrade", "-y"]) {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
            
            println!("All packages upgraded!");
        }
        _ => {
            eprintln!("Unknown action: {}", args[0]);
            process::exit(1);
        }
    }
}

fn handle_disk() {
    println!("Disk Management");
    println!("===============");
    println!();
    
    if let Ok(disks) = get_disk_usage() {
        println!("Disk Usage:");
        println!("-----------");
        println!("{:<15} {:<8} {:<8} {:<8} {:<8} {}", 
            "Device", "Size", "Used", "Avail", "Use%", "Mount");
        println!("{}", "-".repeat(60));
        
        for disk in disks {
            println!("{:<15} {:<8} {:<8} {:<8} {:<8} {}", 
                disk.device, disk.size, disk.used, disk.available, 
                disk.use_percent, disk.mount_point);
        }
    }
}

fn handle_cleanup(args: &[String]) {
    println!("System Cleanup");
    println!("==============");
    println!();
    
    if args.is_empty() {
        println!("Usage: linuxcare-cli cleanup <type>");
        println!();
        println!("Types:");
        println!("  apt       Clean APT cache");
        println!("  snap      Clean Snap cache");
        println!("  flatpak   Clean Flatpak cache");
        println!("  logs      Clean log files");
        println!("  temp      Clean temporary files");
        println!("  all       Clean all caches");
        return;
    }
    
    match args[0].as_str() {
        "apt" => {
            println!("Cleaning APT cache...");
            match clean_apt_cache() {
                Ok(_) => println!("APT cache cleaned!"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "snap" => {
            println!("Cleaning Snap cache...");
            match clean_snap_cache() {
                Ok(_) => println!("Snap cache cleaned!"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "flatpak" => {
            println!("Cleaning Flatpak cache...");
            match clean_flatpak_cache() {
                Ok(_) => println!("Flatpak cache cleaned!"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "logs" => {
            println!("Cleaning log files...");
            match clean_log_files() {
                Ok(_) => println!("Log files cleaned!"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "temp" => {
            println!("Cleaning temporary files...");
            match clean_temp_files() {
                Ok(_) => println!("Temporary files cleaned!"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "all" => {
            println!("Cleaning all caches...");
            
            println!("1. Cleaning APT cache...");
            let _ = clean_apt_cache();
            
            println!("2. Cleaning Snap cache...");
            let _ = clean_snap_cache();
            
            println!("3. Cleaning Flatpak cache...");
            let _ = clean_flatpak_cache();
            
            println!("4. Cleaning log files...");
            let _ = clean_log_files();
            
            println!("5. Cleaning temporary files...");
            let _ = clean_temp_files();
            
            println!();
            println!("All caches cleaned!");
        }
        _ => {
            eprintln!("Unknown cleanup type: {}", args[0]);
            process::exit(1);
        }
    }
}

fn handle_menu() {
    println!("Menu Management");
    println!("===============");
    println!();
    println!("This command provides menu management functionality.");
    println!("Use the GUI version for full menu management features.");
    println!();
    println!("Available options:");
    println!("  --list           List menu items");
    println!("  --add            Add a menu item");
    println!("  --remove         Remove a menu item");
}

fn handle_shortcut() {
    println!("Shortcut Management");
    println!("===================");
    println!();
    println!("This command provides shortcut management functionality.");
    println!("Use the GUI version for full shortcut management features.");
    println!();
    println!("Available options:");
    println!("  --list           List shortcuts");
    println!("  --add            Add a shortcut");
    println!("  --remove         Remove a shortcut");
}
