//! Parse /proc/meminfo and display information about memory usage.
//! License: BSD 2-Clause License

#![allow(non_upper_case_globals)]
#![cfg(target_os = "linux")]

use clap::Parser;
use std::fs;

const MEMINFO_PATH: &str = "/proc/meminfo";
const WHITE_COLOR: &str = "\x1b[1;37m";
const END_COLOR: &str = "\x1b[0m";

// Convert to bytes
macro_rules! to_bytes {
    ($size:expr, $unit:expr) => {
        ($size as f64 * $unit)
    };
}

// Convert to a specific size
macro_rules! to_size {
    ($size:expr, $nunit:expr) => {
        ($size as f64 * 1024.0) / ($nunit as f64)
    };
}

// Binary system
const TO_B: f64 = 1.0;
const TO_KB: f64 = 1024.0;
const TO_MB: f64 = TO_KB * 1024.0;
const TO_GB: f64 = TO_MB * 1024.0;
const TO_TB: f64 = TO_GB * 1024.0;
const TO_PB: f64 = TO_TB * 1024.0;

// Decimal system
const TO_KiB: f64 = 1000.0;
const TO_MiB: f64 = TO_KiB * 1000.0;
const TO_GiB: f64 = TO_MiB * 1000.0;
const TO_TiB: f64 = TO_GiB * 1000.0;
const TO_PiB: f64 = TO_TiB * 1000.0;

// Lwm low memory
struct Lwm {
    /// Total installed memory (RAM)
    mem_total: u64,

    /// Free memory (that isn't actively allocated)
    mem_free: u64,

    /// Available memory
    mem_avail: u64,

    /// Memory that's actively allocated
    mem_used: u64,

    /// Temporary buffers used by the kernel
    buffers: u64,

    /// Memory used by page cache and slabs
    cached: u64,

    /// Swap cached memory (to the disk)
    swap_cached: u64,

    /// Total allocable swap memory
    swap_total: u64,

    /// Free swap (that isn't actively being used or allocated)
    swap_free: u64,

    /// Used swap (that is actively allocated or being used)
    swap_used: u64,

    /// Total zswap memory
    zswap: u64,

    /// Total zswapped memory
    zswapped: u64,

    /// Kernel shared memory
    shmem: u64,

    /// Reclaimable slab memory
    s_reclaimable: u64,
}

#[derive(Parser, Debug)]
struct LwmArgs {
    /// Print the default information (default)
    #[arg(short, long)]
    all: bool,

    /// Disable output colors
    #[arg(short, long)]
    no_color: bool,

    /// Calculate in binary
    #[arg(short, long)]
    binary: bool,

    /// Friendly (human-readable) output
    #[arg(short, long)]
    friendly: bool,

    /// Print memory information in bytes
    #[arg(long)]
    bytes: bool,

    /// Print memory information in kilobytes
    #[arg(long)]
    kilo: bool,

    /// Print memory information in kibibytes
    #[arg(long)]
    kibi: bool,

    /// Print memory information in megabytes
    #[arg(long)]
    mega: bool,

    /// Print memory information in mibibytes
    #[arg(long)]
    mibi: bool,

    /// Print memory information in gigabytes
    #[arg(long)]
    giga: bool,

    /// Print memory information in gibibytes
    #[arg(long)]
    gibi: bool,

    /// Print memory information in terabytes
    #[arg(long)]
    tera: bool,

    /// Print memory information in terabytes
    #[arg(long)]
    tibi: bool,

    /// Print memory information in petabytes
    #[arg(long)]
    peta: bool,

    /// Print memory information in petabytes
    #[arg(long)]
    pibi: bool,
}

impl Lwm {
    fn new() -> Self {
        Self {
            mem_total: 0,
            mem_free: 0,
            mem_avail: 0,
            mem_used: 0,
            buffers: 0,
            cached: 0,
            swap_cached: 0,
            swap_total: 0,
            swap_free: 0,
            swap_used: 0,
            zswap: 0,
            zswapped: 0,
            shmem: 0,
            s_reclaimable: 0,
        }
    }

    #[inline]
    fn lwm_read_file(&self) -> String {
        fs::read_to_string(MEMINFO_PATH).unwrap()
    }

    fn lwm_get_value(&self, src: &str, key: &str) -> u64 {
        let mut value = String::new();

        src.lines().for_each(|e| {
            // If we're able to find a match
            if e.starts_with(key) {
                let second = e.split(':').nth(1).unwrap();
                if second.contains("kB") {
                    value.push_str(second.trim_end_matches("kB").trim());
                } else {
                    value.push_str(second.trim());
                }
            }
        });

        value.parse::<u64>().unwrap()
    }

    fn lwm_attach_values(&mut self) {
        let src = self.lwm_read_file();

        self.mem_total = self.lwm_get_value(&src, "MemTotal:");
        self.mem_free = self.lwm_get_value(&src, "MemFree:");
        self.mem_avail = self.lwm_get_value(&src, "MemAvailable:");
        self.mem_used = self.mem_total - self.mem_avail;
        self.buffers = self.lwm_get_value(&src, "Buffers:");
        self.cached = self.lwm_get_value(&src, "Cached:");
        self.swap_cached = self.lwm_get_value(&src, "SwapCached:");
        self.swap_free = self.lwm_get_value(&src, "SwapFree:");
        self.swap_total = self.lwm_get_value(&src, "SwapTotal:");
        self.swap_used = self.swap_total - self.swap_free;
        self.zswap = self.lwm_get_value(&src, "Zswap:");
        self.zswapped = self.lwm_get_value(&src, "Zswapped:");
        self.shmem = self.lwm_get_value(&src, "Shmem:");
        self.s_reclaimable = self.lwm_get_value(&src, "SReclaimable:");
    }

    // Taken from: https://git.sr.ht/~nkeor/human_bytes/tree/main/item/src/lib.rs
    fn lwm_conv_to_hbytes(&self, size: f64, binary: bool) -> String {
        if size <= 0.0 {
            return "0B".to_string();
        }

        // If binary use 1024, and if not (decimal) use 1000 as the unit
        let unit = if binary { 1024.0 } else { 1000.0 } as f64;
        let base = size.log10() / unit.log10();
        let mut buffer = ryu::Buffer::new();
        let result = buffer
            // Source for this hack: https://stackoverflow.com/a/28656825
            .format((unit.powf(base - base.floor()) * 10.0).round() / 10.0);

        // Add suffix
        if binary {
            const SUFFIX: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
            [result, SUFFIX[base.floor() as usize]].join("")
        } else {
            const SUFFIX: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
            [result, SUFFIX[base.floor() as usize]].join("")
        }
    }

    fn lwm_print_all(&self, is_binary: bool, is_frndly: bool, is_color: bool) {
        let unit = if is_binary { 1024.0 } else { 1000.0 };

        if is_frndly {
            if is_color {
                let output = format!(
                    "======================\n\
                     | Memory Information |\n\
                     ======================\n\
                     * {WHITE_COLOR}Total Memory{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Free Memory{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Avail Memory{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Used Memory{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Buffered{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Total Swap{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Free Swap{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Cached Swap{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Used Swap{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Total ZSwap{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Commit ZSwap{END_COLOR}: {}\n\
                     * {WHITE_COLOR}Shared Memory{END_COLOR}: {}",
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_total, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_free, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_avail, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_used, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.buffers, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_total, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_free, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_cached, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_used, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.zswap, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.zswapped, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.shmem, unit) as f64, is_binary)
                );
                println!("{}", output);
            } else {
                let output = format!(
                    "======================\n\
                     | Memory Information |\n\
                     ======================\n\
                     * Total Memory: {}\n\
                     * Free Memory: {}\n\
                     * Avail Memory: {}\n\
                     * Used Memory: {}\n\
                     * Buffered: {}\n\
                     * Total Swap: {}\n\
                     * Free Swap: {}\n\
                     * Cached Swap: {}\n\
                     * Used Swap: {}\n\
                     * Total ZSwap: {}\n\
                     * Commit ZSwap: {}\n\
                     * Shared Memory: {}",
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_total, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_free, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_avail, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.mem_used, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.buffers, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_total, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_free, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_cached, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.swap_used, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.zswap, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.zswapped, unit) as f64, is_binary),
                    self.lwm_conv_to_hbytes(to_bytes!(self.shmem, unit) as f64, is_binary)
                );
                println!("{}", output);
            }
        } else {
            let output = format!(
                "======================\n\
                 | Memory Information |\n\
                 ======================\n\
                 * {WHITE_COLOR}Total Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Free Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Avail Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Used Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Buffered{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Total Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Free Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Cached Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Used Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Total ZSwap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Commit ZSwap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Shared Memory{END_COLOR}: {}",
                to_bytes!(self.mem_total, 1024.0) as u64,
                to_bytes!(self.mem_free, 1024.0) as u64,
                to_bytes!(self.mem_avail, 1024.0) as u64,
                to_bytes!(self.mem_used, 1024.0) as u64,
                to_bytes!(self.buffers, 1024.0) as u64,
                to_bytes!(self.swap_total, 1024.0) as u64,
                to_bytes!(self.swap_free, 1024.0) as u64,
                to_bytes!(self.swap_cached, 1024.0) as u64,
                to_bytes!(self.swap_used, 1024.0) as u64,
                to_bytes!(self.zswap, 1024.0) as u64,
                to_bytes!(self.zswapped, 1024.0) as u64,
                to_bytes!(self.shmem, 1024.0) as u64
            );
            println!("{}", output);
        }
    }

    fn lwm_print_to_size(&self, size: f64, is_color: bool) {
        if is_color {
            let output = format!(
                "======================\n\
                 | Memory Information |\n\
                 ======================\n\
                 * {WHITE_COLOR}Total Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Free Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Avail Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Used Memory{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Buffered{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Total Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Free Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Cached Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Used Swap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Total ZSwap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Commit ZSwap{END_COLOR}: {}\n\
                 * {WHITE_COLOR}Shared Memory{END_COLOR}: {}",
                to_size!(self.mem_total, size) as u64,
                to_size!(self.mem_free, size) as u64,
                to_size!(self.mem_avail, size) as u64,
                to_size!(self.mem_used, size) as u64,
                to_size!(self.buffers, size) as u64,
                to_size!(self.swap_total, size) as u64,
                to_size!(self.swap_free, size) as u64,
                to_size!(self.swap_cached, size) as u64,
                to_size!(self.swap_used, size) as u64,
                to_size!(self.zswap, size) as u64,
                to_size!(self.zswapped, size) as u64,
                to_size!(self.shmem, size) as u64
            );
            println!("{}", output);
        } else {
            let output = format!(
                "======================\n\
                 | Memory Information |\n\
                 ======================\n\
                 * Total Memory: {}\n\
                 * Free Memory: {}\n\
                 * Avail Memory: {}\n\
                 * Used Memory: {}\n\
                 * Buffered: {}\n\
                 * Total Swap: {}\n\
                 * Free Swap: {}\n\
                 * Cached Swap: {}\n\
                 * Used Swap: {}\n\
                 * Total ZSwap: {}\n\
                 * Commit ZSwap: {}\n\
                 * Shared Memory: {}",
                to_size!(self.mem_total, size) as u64,
                to_size!(self.mem_free, size) as u64,
                to_size!(self.mem_avail, size) as u64,
                to_size!(self.mem_used, size) as u64,
                to_size!(self.buffers, size) as u64,
                to_size!(self.swap_total, size) as u64,
                to_size!(self.swap_free, size) as u64,
                to_size!(self.swap_cached, size) as u64,
                to_size!(self.swap_used, size) as u64,
                to_size!(self.zswap, size) as u64,
                to_size!(self.zswapped, size) as u64,
                to_size!(self.shmem, size) as u64
            );
            println!("{}", output);
        }
    }
}

fn main() {
    let mut lwm = Lwm::new();
    let lwm_args = LwmArgs::parse();

    // Query for the requested fields
    lwm.lwm_attach_values();

    if lwm_args.all {
        lwm.lwm_print_all(lwm_args.binary, lwm_args.friendly, !lwm_args.no_color);
    } else if lwm_args.bytes {
        lwm.lwm_print_to_size(TO_B, !lwm_args.no_color);
    } else if lwm_args.kilo {
        lwm.lwm_print_to_size(TO_KB, !lwm_args.no_color);
    } else if lwm_args.kibi {
        lwm.lwm_print_to_size(TO_KiB, !lwm_args.no_color);
    } else if lwm_args.mega {
        lwm.lwm_print_to_size(TO_MB, !lwm_args.no_color);
    } else if lwm_args.mibi {
        lwm.lwm_print_to_size(TO_MiB, !lwm_args.no_color);
    } else if lwm_args.giga {
        lwm.lwm_print_to_size(TO_GB, !lwm_args.no_color);
    } else if lwm_args.gibi {
        lwm.lwm_print_to_size(TO_GiB, !lwm_args.no_color);
    } else if lwm_args.tera {
        lwm.lwm_print_to_size(TO_TB, !lwm_args.no_color);
    } else if lwm_args.tibi {
        lwm.lwm_print_to_size(TO_TiB, !lwm_args.no_color);
    } else if lwm_args.peta {
        lwm.lwm_print_to_size(TO_PB, !lwm_args.no_color);
    } else if lwm_args.pibi {
        lwm.lwm_print_to_size(TO_PiB, !lwm_args.no_color);
    } else {
        lwm.lwm_print_all(lwm_args.binary, lwm_args.friendly, !lwm_args.no_color);
    }
}
