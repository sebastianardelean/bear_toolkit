/*
VSZ (Virtual Memory Size) refers to the total amount of virtual memory that a process is using. This includes both the portion that is in RAM and the portion that has been swapped out to disk. It also includes any shared libraries that the process is using.

RSS (Resident Set Size) refers to the portion of a process's memory that is held in RAM. This includes both the code and data segments of the process. It does not include memory that has been swapped out to disk.

Uss: This is the amount of memory that is committed to physical memory and is unique to a process; it is not shared with any other. It is the amount of memory that would be freed if the process were to terminate. USS is a useful metric for determining the actual memory usage of a process, as it doesn't include shared memory that might be double-counted when looking at memory usage across multiple processes.


Pss: This splits the accounting of shared pages that are committed to physical memory between all the processes that have them mapped. For example, if an area of library code is 12 pages long and is shared by six processes, each will accumulate two pages in Pss. Thus, if you add the Pss numbers for all processes, you will get the actual amount of memory being used by those processes. In other words,Pss is the number we have been looking for.

cat /proc/18756/smaps | grep -i pss |  awk '{Total+=$2} END {print Total/1024/1024" GB"}

TODO:
1. Until signal is caught, every x ms
 */

use clap::{Args, Parser, Subcommand};
use core::panic;
use std::{fs, io::Error, io::ErrorKind, thread, time};

#[derive(Parser, Clone)]
#[command(version,about,long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    pid: i32,

    /// Print value in KB
    #[arg(short, long)]
    kb: bool,

    /// Print value in MB
    #[arg(short, long)]
    mb: bool,

    /// Print value in GB
    #[arg(short, long)]
    gb: bool,

    /// Get a specific number of samples.
    #[arg(short, long, default_value_t = 1)]
    samples: usize,

    /// Read every specified number of ms.
    #[arg(short, long, default_value_t = 10)]
    interval: u64,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let process_pid = args.pid;

    if args.samples == 1 {
        let mem_used = extract_pss_memory_kb(process_pid);
        match mem_used {
            Ok(val) => pretty_print(val as f64, args),
            Err(e) => panic!("Error: {}", e),
        }
    } else {
        let mut samples: Vec<u64> = Vec::new();
        let time_sleep = time::Duration::from_millis(args.interval);
        let mut count = 0;
        while count < args.samples {
            let sample = extract_pss_memory_kb(process_pid);
            match sample {
                Ok(val) => {
                    samples.push(val);
                    let now = time::Instant::now();
                    thread::sleep(time_sleep);
                    assert!(now.elapsed() >= time_sleep);
                }
                Err(e) => panic!("Error: {}", e),
            }
            count += 1;
        }
        let max = *samples.iter().max().unwrap_or(&0);

        let min = *samples.iter().min().unwrap_or(&0);

        let sum: u64 = samples.iter().sum();
        let average = if !samples.is_empty() {
            sum as f64 / samples.len() as f64
        } else {
            0.0
        };

        print!("Max: ");
        pretty_print(max as f64, args.clone());
        print!("Min: ");
        pretty_print(min as f64, args.clone());
        print!("Average: ");
        pretty_print(average as f64, args.clone());
    }

    Ok(())
}

fn pretty_print(value: f64, args: Cli) {
    if args.kb {
        println!("Total Pss: {:.6} KB", value);
    } else {
        if args.mb {
            let total_pss_mb = value / 1024.0;
            println!("Total Pss: {:.6} MB", total_pss_mb);
        } else {
            if args.gb {
                let total_pss_gb = value / 1024.0 / 1024.0;
                println!("Total Pss: {:.6} GB", total_pss_gb);
            } else {
                println!("Total Pss: {:.6} KB", value);
            }
        }
    }
}

fn extract_pss_memory_kb(pid: i32) -> Result<u64, std::io::Error> {
    let smaps_fd_path: String = format!("/proc/{}/smaps", pid);
    let smaps_content = fs::read_to_string(smaps_fd_path).unwrap_or_else(|err| {
        panic!("File not found: {}", err);
    });

    let mut total_pss_kb: u64 = 0;

    for line in smaps_content.lines() {
        if line.to_lowercase().contains("pss") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(value) = parts.get(1) {
                if let Ok(num) = value.parse::<u64>() {
                    total_pss_kb += num;
                } else {
                    return Err(Error::new(ErrorKind::Other, "oh no!"));
                }
            } else {
                return Err(Error::new(ErrorKind::Other, "Oh no no no no no!"));
            }
        }
    }
    Ok(total_pss_kb)
}
