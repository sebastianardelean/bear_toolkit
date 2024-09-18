pub mod cli {
    use bprocmemm::process::process::*;
    use clap::{Args, Parser, Subcommand};

    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    #[command(propagate_version = true)]
    pub struct Cli {
        /// Pid of the process
        #[arg(short, long)]
        pub pid: u32,

        // /// Print memory regions
        // #[arg(short, long)]
        // mem: bool,
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Subcommand)]
    pub enum Commands {
        /// Print memory regions
        Print,

        /// Read memory region
        ReadRegion(ReadArgsRegion),

        /// Read memory from address
        ReadAddress(ReadArgsAddress),

        /// Write data into memory region
        WriteRegion(WriteArgsRegion),

        /// Write data to memory address
        WriteAddress(WriteArgsAddress),

        /// Change protection of a memory region
        ChangeProtection(ChangeProtectionArgs),

        /// Allocate memory
        AllocateMemory(AllocateMemoryArgs),
    }
    #[derive(Args)]
    pub struct AllocateMemoryArgs {
        #[arg(name = "address", help = "Memory address to allocate.", value_parser = parse_hex_usize)]
        pub address: usize,
        #[arg(name = "size", help = "Memory size to allocate")]
        pub size: usize,
    }

    #[derive(Args)]
    pub struct ChangeProtectionArgs {
        #[arg(name = "region", help = "Index of the memory region to read.")]
        pub region: usize,
        #[arg(
            name = "protection",
            help = "String of protections separated by |. Available options are:NOACCESS|READONLY|READWRITE|WRITECOPY|EXECUTE|EXECUTE_READ|EXECUTE_READWRITE|EXECUTE_WRITECOPY|GUARD|NOCACHE|WRITECOMBINE"
        )]
        pub protection: String,
    }

    #[derive(Args)]
    pub struct ReadArgsRegion {
        #[arg(name = "region", help = "Index of the memory region to read.")]
        pub region: usize,
    }

    #[derive(Args)]
    pub struct ReadArgsAddress {
        #[arg(name = "address", help = "Base address from memory.", value_parser = parse_hex_usize)]
        pub address: usize,
        #[arg(name = "size", help = "Data length to read.")]
        pub size: usize,
    }

    #[derive(Args)]
    pub struct WriteArgsRegion {
        #[arg(name = "region", help = "Index of the memory region to read.")]
        pub region: usize,

        #[arg(name = "data", help = "Data to be written in process.",value_parser = parse_hex_vector)]
        pub data: Vec<u8>,
    }

    #[derive(Args)]
    pub struct WriteArgsAddress {
        #[arg(name = "address", help = "Base address from memory.", value_parser = parse_hex_usize)]
        pub address: usize,

        #[arg(name = "data", help = "Data to be written in process.",value_parser = parse_hex_vector)]
        pub data: Vec<u8>,
    }

    fn parse_hex_usize(value: &str) -> Result<usize, String> {
        usize::from_str_radix(value, 16).map_err(|e| e.to_string())
    }

    fn parse_hex_vector(value: &str) -> Result<u8, String> {
        // Convert hexadecimal string to u8
        u8::from_str_radix(value, 16).map_err(|e| e.to_string())
    }

    pub fn execute() {
        let args = Cli::parse();
        let process_pid = args.pid;
        let process: Proc = Proc::new(process_pid);

        let mut mem_regions = process.read_memory_regions();

        match &args.command {
            Commands::Print => {
                mem_regions.iter().enumerate().for_each(|(index, mem)| {
                    println!("Memory region {}: {:#?}", index, mem.show_memory_regions())
                });
            }

            Commands::ReadRegion(read_args) => match mem_regions.get(read_args.region) {
                Some(reg) => {
                    let read_data = process.read_memory_region(reg.clone());

                    match read_data {
                        Ok(mem) => {
                            mem.iter().for_each(|byte| print!("{:02x}", byte));
                        }
                        Err(e) => {
                            eprintln!("Failed to read memory region: {}", e);
                        }
                    }
                }
                None => eprintln!("There is no memory region."),
            },

            Commands::ReadAddress(read_args) => {
                let read_data = process.read_address(read_args.address, read_args.size);

                match read_data {
                    Ok(mem) => {
                        mem.iter().for_each(|byte| print!("{:02x}", byte));
                    }
                    Err(e) => {
                        eprintln!("Failed to read memory region: {}", e);
                    }
                }
            }

            Commands::WriteRegion(write_args) => match mem_regions.get(write_args.region) {
                Some(reg) => {
                    let result = process.write_region(reg.clone(), &write_args.data);
                    match result {
                        Ok(_) => println!("Successfully written data"),
                        Err(e) => eprintln!("Failed to write memory region: {}", e),
                    };
                }
                None => eprintln!("No memory region!"),
            },

            Commands::WriteAddress(write_args) => {
                let result = process.write_address(write_args.address, &write_args.data);
                match result {
                    Ok(_) => println!("Successfully written data"),
                    Err(e) => eprintln!("Failed to write memory region: {}", e),
                };
            }

            Commands::ChangeProtection(protection_args) => {
                match mem_regions.get(protection_args.region) {
                    Some(reg) => {
                        let result =
                            process.set_protection(reg.clone(), protection_args.protection.clone());
                        match result {
                            Ok(_) => println!("Successfully changed protection"),
                            Err(e) => {
                                eprintln!("Failed to change protection of memory region: {}", e)
                            }
                        };
                    }
                    None => eprintln!("No memory region!"),
                }
            }

            Commands::AllocateMemory(allocate_memory_args) => {
                let result = process
                    .allocate_memory(allocate_memory_args.address, allocate_memory_args.size);
                match result {
                    Ok(mem_reg) => {
                        println!("Successfully allocated memory!");
                        mem_regions.push(mem_reg)
                    }
                    Err(e) => eprintln!("Failed to allocate memory region: {}", e),
                };
            }
        }
    }
}
