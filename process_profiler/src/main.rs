/*
VSZ (Virtual Memory Size) refers to the total amount of virtual memory that a process is using. This includes both the portion that is in RAM and the portion that has been swapped out to disk. It also includes any shared libraries that the process is using.

RSS (Resident Set Size) refers to the portion of a process's memory that is held in RAM. This includes both the code and data segments of the process. It does not include memory that has been swapped out to disk.

Uss: This is the amount of memory that is committed to physical memory and is unique to a process; it is not shared with any other. It is the amount of memory that would be freed if the process were to terminate. USS is a useful metric for determining the actual memory usage of a process, as it doesn't include shared memory that might be double-counted when looking at memory usage across multiple processes.


Pss: This splits the accounting of shared pages that are committed to physical memory between all the processes that have them mapped. For example, if an area of library code is 12 pages long and is shared by six processes, each will accumulate two pages in Pss. Thus, if you add the Pss numbers for all processes, you will get the actual amount of memory being used by those processes. In other words,Pss is the number we have been looking for.

I'm probably interested in RSS.
So, if I cat /proc/[PID]/statm, the second value is rss (index from 1) and to convert the value to KB I have to multiply with page size

or VmRSS from /proc/[PID]/status

or

cat /proc/18756/smaps | grep -i pss |  awk '{Total+=$2} END {print Total/1024/1024" GB"}
 */

use clap::{Args, Parser, Subcommand};
use core::panic;
use std::{fs, io::Error, io::ErrorKind};

#[derive(Parser)]
#[command(version,about,long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    pid: i32,

    #[command(subcommand)]
    unit: Units,
}

#[derive(Subcommand)]
enum Units {
    /// Print in KB
    K,
    /// Print in MB
    M,
    /// Print in GB
    G,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let process_pid = args.pid;

    let mem_used = extract_pss_memory_kb(process_pid);

    match mem_used {
        Ok(val) => match &args.unit {
            Units::K => {
                println!("Total Pss: {} KB", val);
            }
            Units::M => {
                let total_pss_mb = val as f64 / 1024.0;
                println!("Total Pss: {:.6} MB", total_pss_mb);
            }
            Units::G => {
                let total_pss_gb = val as f64 / 1024.0 / 1024.0;
                println!("Total Pss: {:.6} GB", total_pss_gb);
            }
        },
        Err(e) => panic!("Error: {}", e),
    }

    Ok(())
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
