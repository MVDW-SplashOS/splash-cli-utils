use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: sig <signal_type> [--all|-a] <process_id|process_name>");
        eprintln!("Example: sig int 3626");
        eprintln!("         sig 9 3626");
        eprintln!("         sig kill chrome");
        eprintln!("         sig kill --all chrome");
        eprintln!("         sig kill -a chrome");
        process::exit(1);
    }

    let signal_name = &args[1];
    let mut all_flag = false;
    let target: &String;

    // Check for --all or -a flag
    if args.len() == 4 && (args[2] == "--all" || args[2] == "-a") {
        all_flag = true;
        target = &args[3];
    } else if args.len() == 3 {
        target = &args[2];
    } else {
        eprintln!("Error: Invalid arguments");
        eprintln!("Usage: sig <signal_type> [--all|-a] <process_id|process_name>");
        process::exit(1);
    }

    // Try to parse signal as a number first, otherwise match by name
    let signal = if let Ok(num) = signal_name.parse::<i32>() {
        num
    } else {
        match signal_name.to_lowercase().as_str() {
            "int" | "interrupt" | "sigint" => 2,
            "term" | "terminate" | "sigterm" => 15,
            "kill" | "sigkill" => 9,
            "hup" | "hangup" | "sighup" => 1,
            "quit" | "sigquit" => 3,
            "usr1" | "sigusr1" => 10,
            "usr2" | "sigusr2" => 12,
            "stop" | "sigstop" => 19,
            "cont" | "sigcont" => 18,
            _ => {
                eprintln!("Error: Unknown signal '{}'", signal_name);
                eprintln!("Supported signals: int, term, kill, hup, quit, usr1, usr2, stop, cont");
                eprintln!("Or use a signal number directly (e.g., 9 for SIGKILL)");
                process::exit(1);
            }
        }
    };

    // Check if target is a PID or process name
    if let Ok(pid) = target.parse::<i32>() {
        // It's a PID
        send_signal_to_pid(pid, signal, signal_name);
    } else {
        // It's a process name
        let pids = find_processes_by_name(target);

        if pids.is_empty() {
            eprintln!("Error: No processes found with name '{}'", target);
            process::exit(1);
        }

        if pids.len() > 1 && !all_flag {
            println!("Found {} processes with name '{}':", pids.len(), target);
            for pid in &pids {
                println!("  PID: {}", pid);
            }
            eprintln!("\nUse --all or -a flag to send signal to all processes");
            process::exit(1);
        }

        // Send signal to all found processes
        let mut success_count = 0;
        for pid in &pids {
            if send_signal_to_pid(*pid, signal, signal_name) {
                success_count += 1;
            }
        }

        if success_count > 0 {
            println!("Successfully sent signal to {} process(es)", success_count);
        }
    }
}

fn send_signal_to_pid(pid: i32, signal: i32, signal_name: &str) -> bool {
    unsafe {
        let result = libc::kill(pid, signal);
        if result == 0 {
            println!("Signal {} sent to process {}", signal_name, pid);
            true
        } else {
            let errno = *libc::__errno_location();
            match errno {
                libc::ESRCH => eprintln!("Error: No process with PID {} found", pid),
                libc::EPERM => {
                    eprintln!("Error: Permission denied to send signal to process {}", pid)
                }
                libc::EINVAL => eprintln!("Error: Invalid signal number"),
                _ => eprintln!(
                    "Error: Failed to send signal to PID {} (errno: {})",
                    pid, errno
                ),
            }
            false
        }
    }
}

fn find_processes_by_name(name: &str) -> Vec<i32> {
    let mut pids = Vec::new();

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if let Ok(pid) = file_name.parse::<i32>() {
                    // Read the cmdline file to get the process name
                    let cmdline_path = format!("/proc/{}/cmdline", pid);
                    if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
                        // cmdline uses null bytes as separators
                        let cmd = cmdline.split('\0').next().unwrap_or("");

                        // Extract just the executable name
                        let exe_name = cmd.split('/').last().unwrap_or(cmd);

                        if exe_name.contains(name) || cmd.contains(name) {
                            pids.push(pid);
                        }
                    }
                }
            }
        }
    }

    pids
}
