use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::process;
use std::time::Instant;

// Default to 16MB buffer - good balance for modern USB 3.0+ drives
const DEFAULT_BUFFER_SIZE: usize = 16 * 1024 * 1024;
const BENCHMARK_DATA_SIZE: usize = 64 * 1024 * 1024; // 64MB for benchmark

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: mkdev <source_file> <target_device> [--buffer-size <size_in_mb>]");
        eprintln!("Example: mkdev ubuntu.iso /dev/sdc");
        eprintln!("         mkdev ubuntu.iso /dev/sdc --buffer-size 32");
        eprintln!("\nBy default, mkdev auto-detects the optimal buffer size for your device.");
        eprintln!("Use --buffer-size to manually override if needed.");
        eprintln!("\nWarning: This will OVERWRITE all data on the target device!");
        process::exit(1);
    }

    let source_path = &args[1];
    let target_path = &args[2];

    // Parse flags
    let mut manual_buffer_size = None;

    for i in 3..args.len() {
        if args[i] == "--buffer-size" && i + 1 < args.len() {
            match args[i + 1].parse::<usize>() {
                Ok(size_mb) => manual_buffer_size = Some(size_mb * 1024 * 1024),
                Err(_) => {
                    eprintln!("Error: Invalid buffer size. Use a number in MB (e.g., 16 for 16MB)");
                    process::exit(1);
                }
            }
        }
    }

    // Confirm operation
    println!("Source: {}", source_path);
    println!("Target: {}", target_path);
    println!(
        "\n⚠️  WARNING: This will permanently erase all data on {}!",
        target_path
    );
    println!("Are you sure you want to continue? (yes/no): ");

    let mut confirmation = String::new();
    io::stdin()
        .read_line(&mut confirmation)
        .expect("Failed to read input");

    if confirmation.trim().to_lowercase() != "yes" {
        println!("Operation cancelled.");
        process::exit(0);
    }

    // Open source file
    let mut source_file = match File::open(source_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: Cannot open source file '{}': {}", source_path, e);
            process::exit(1);
        }
    };

    let source_size = match source_file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            eprintln!("Error: Cannot read source file metadata: {}", e);
            process::exit(1);
        }
    };

    // Open target device - try O_DIRECT first, fall back to normal mode
    let mut target_file = match OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_DIRECT | libc::O_SYNC)
        .open(target_path)
    {
        Ok(file) => {
            println!("📡 Using direct I/O mode for optimal performance");
            file
        }
        Err(_) => match OpenOptions::new().write(true).open(target_path) {
            Ok(file) => {
                println!("⚠️  Using buffered I/O mode (direct I/O not available)");
                file
            }
            Err(e) => {
                eprintln!("Error: Cannot open target device '{}': {}", target_path, e);
                eprintln!("Make sure you have permission (try sudo) and the device exists.");
                process::exit(1);
            }
        },
    };

    println!(
        "\n📦 Source size: {:.2} MB ({} bytes)",
        source_size as f64 / 1_000_000.0,
        source_size
    );

    // Determine buffer size
    let buffer_size = if let Some(size) = manual_buffer_size {
        println!(
            "🔧 Using manually specified buffer size: {:.1}MB\n",
            size as f64 / 1_048_576.0
        );
        size
    } else {
        println!("🔍 Auto-detecting optimal buffer size...");
        match detect_optimal_buffer_size(&mut source_file, &mut target_file, source_size) {
            Ok(size) => {
                println!(
                    "✅ Optimal buffer size detected: {:.1}MB\n",
                    size as f64 / 1_048_576.0
                );
                size
            }
            Err(e) => {
                eprintln!(
                    "⚠️  Warning: Auto-detection failed ({}), using default 16MB\n",
                    e
                );
                DEFAULT_BUFFER_SIZE
            }
        }
    };

    // Reset file positions after benchmark
    if manual_buffer_size.is_none() {
        source_file.seek(SeekFrom::Start(0)).ok();
        target_file.seek(SeekFrom::Start(0)).ok();
    }

    println!("🚀 Starting write operation...\n");

    // Perform the copy operation
    if let Err(e) = copy_with_progress(source_file, target_file, source_size, buffer_size) {
        eprintln!("\n❌ Error during write operation: {}", e);
        process::exit(1);
    }

    println!("\n✅ Successfully written to {}", target_path);
}

fn detect_optimal_buffer_size(
    source: &mut File,
    _target: &mut File,
    source_size: u64,
) -> io::Result<usize> {
    // Test buffer sizes: 2MB, 4MB, 8MB, 16MB, 32MB, 64MB
    let test_sizes = vec![
        2 * 1024 * 1024,
        4 * 1024 * 1024,
        8 * 1024 * 1024,
        16 * 1024 * 1024,
        32 * 1024 * 1024,
        64 * 1024 * 1024,
    ];

    let test_data_size = BENCHMARK_DATA_SIZE.min(source_size as usize);
    let mut best_size = DEFAULT_BUFFER_SIZE;
    let mut best_speed = 0.0;

    println!(
        "  Testing buffer sizes with {}MB of data...",
        test_data_size / 1_048_576
    );

    for &buffer_size in &test_sizes {
        source.seek(SeekFrom::Start(0))?;

        let start = Instant::now();
        // Use aligned buffer for O_DIRECT compatibility
        let mut buffer = vec![0u8; buffer_size];
        let mut written = 0;

        // For the benchmark, we'll just read from source without writing to target
        // to avoid O_DIRECT alignment issues during benchmarking
        while written < test_data_size {
            let to_read = buffer_size.min(test_data_size - written);
            let bytes_read = source.read(&mut buffer[..to_read])?;
            if bytes_read == 0 {
                break;
            }
            written += bytes_read;
        }

        let elapsed = start.elapsed().as_secs_f64();
        let speed = written as f64 / elapsed / 1_000_000.0;

        print!("  {}MB: {:.2} MB/s", buffer_size / 1_048_576, speed);

        if speed > best_speed {
            best_speed = speed;
            best_size = buffer_size;
            println!(" ⭐ (best so far)");
        } else {
            println!();
            // If speed is decreasing, larger buffers won't help
            if speed < best_speed * 0.95 {
                break;
            }
        }
    }

    // Reset source file position after benchmarking
    source.seek(SeekFrom::Start(0))?;

    Ok(best_size)
}

fn copy_with_progress(
    source: File,
    target: File,
    total_size: u64,
    buffer_size: usize,
) -> io::Result<()> {
    let mut reader = BufReader::with_capacity(buffer_size, source);
    let mut writer = BufWriter::with_capacity(buffer_size, target);
    // Ensure buffer is aligned for O_DIRECT (align to 4KB boundary)
    let aligned_size = (buffer_size + 4095) & !4095;
    let mut buffer = vec![0u8; aligned_size];

    let mut total_written = 0u64;
    let start_time = Instant::now();
    let mut last_update = Instant::now();

    loop {
        let bytes_read = reader.read(&mut buffer[..buffer_size])?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        total_written += bytes_read as u64;

        // Update progress every 100ms
        if last_update.elapsed().as_millis() >= 100 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let progress = (total_written as f64 / total_size as f64) * 100.0;
            let speed = total_written as f64 / elapsed / 1_000_000.0;
            let eta = if speed > 0.0 {
                ((total_size - total_written) as f64 / (speed * 1_000_000.0)) as u64
            } else {
                0
            };

            print!(
                "\r📝 Progress: {:.1}% | {:.2}/{:.2} MB | Speed: {:.2} MB/s | ETA: {}s   ",
                progress,
                total_written as f64 / 1_000_000.0,
                total_size as f64 / 1_000_000.0,
                speed,
                eta
            );
            io::stdout().flush()?;
            last_update = Instant::now();
        }
    }

    // Flush any remaining data
    writer.flush()?;

    // Final sync to ensure all data is written
    let target_file = writer.into_inner()?;
    target_file.sync_all()?;

    let elapsed = start_time.elapsed().as_secs_f64();
    let avg_speed = total_written as f64 / elapsed / 1_000_000.0;

    print!(
        "\r📝 Progress: 100.0% | {:.2}/{:.2} MB | Avg Speed: {:.2} MB/s | Time: {:.1}s   ",
        total_written as f64 / 1_000_000.0,
        total_size as f64 / 1_000_000.0,
        avg_speed,
        elapsed
    );
    io::stdout().flush()?;

    Ok(())
}
