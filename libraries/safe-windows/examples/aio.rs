// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

#![allow(clippy::print_stdout)]
#![allow(clippy::unwrap_used)]

use irox_safe_windows::fs::{AsyncFile, AsyncFileExt};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const PAGE_SIZE: usize = 2_048_000;
// const PAGE_SIZE : usize = 1_024_000;
// const PAGE_SIZE: usize = 512_000;
// const PAGE_SIZE : usize = 262_144;
// const PAGE_SIZE : usize = 256_000;
// const PAGE_SIZE : usize = 128_000;
// const PAGE_SIZE : usize = 64_000;
// const PAGE_SIZE : usize = 32_000;
// const PAGE_SIZE : usize = 16_000;
// const PAGE_SIZE : usize = 8_000;
// const PAGE_SIZE : usize = 4_000;
pub const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
pub const FILE_FLAG_RANDOM_ACCESS: u32 = 0x10000000;
pub const FILE_FLAG_WRITE_THROUGH: u32 = 0x80000000;
pub const FILE_FLAG_OVERLAPPED: u32 = 0x40000000;

pub fn main() {
    // let mut rt = tokio::
    let mut rt = irox::threading::CurrentThreadExecutor::new();
    // let file = File::options()
    //     .write(true)
    //     .custom_flags(FILE_FLAG_WRITE_THROUGH | FILE_FLAG_NO_BUFFERING | FILE_FLAG_OVERLAPPED)
    //     .create(true)
    //     .read(true).open("E:/test_page.pagefile").unwrap();
    let file = AsyncFile::open("./test_page.pagefile").unwrap();
    // let mut file = Pagefile::open("test_page.pagefile")?;
    let start = std::time::Instant::now();
    let mut handles = VecDeque::with_capacity(1000);
    let total_size = 8_192_000_000u64;
    let total_pages = total_size / PAGE_SIZE as u64;
    let num_threads = 2;
    let queue = Arc::new(AtomicU64::new(0));
    for _ in 0..num_threads {
        // let file = file.try_clone().unwrap();
        let file = file.clone();
        let queue = queue.clone();
        handles.push_back(rt.submit(async move {
            let file = file;
            loop {
                let page = vec![0u8; PAGE_SIZE].into_boxed_slice();
                let page_index = queue.fetch_add(1, Ordering::Relaxed);
                if page_index > total_pages {
                    break;
                };

                let offset = page_index * PAGE_SIZE as u64;
                // (&file).seek_write(&page, offset).unwrap();
                file.seek_write(page, offset).unwrap().await.unwrap();
            }
        }));
    }
    println!("Run until complete.");
    rt.run_until_complete();
    println!("Complete.");
    while !handles.is_empty() {
        rt.run_some();
        handles.retain_mut(|h| h.get().is_none());
    }
    println!("dropping");
    drop(rt);
    let elapsed = start.elapsed();
    let len = total_pages * PAGE_SIZE as u64;
    let dur = elapsed.as_secs_f64();
    println!(
        "Wrote {} in {}s = {} MB/s",
        len,
        dur,
        len as f64 / dur / 1e6
    );
}
