#![allow(clippy::unwrap_used)]
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]
use irox_bits::Error;
use irox_tools::buf::ZeroedBuffer;
use irox_tools::hex::to_hex_str_lower;
use std::io::Read;
use std::path::PathBuf;
fn main() -> Result<(), Error> {
    let mut hasher = irox_simd::blake2::BLAKE2s256::default();
    let path: PathBuf = "/proj/BLAKE2/b2sum/test10grand".into();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .truncate(false)
        .open(&path)?;
    let filelen = file.metadata()?.len();
    let (tx, rx) = std::sync::mpsc::sync_channel(30);
    let join = std::thread::spawn(move || loop {
        let mut buf = <Vec<u8> as ZeroedBuffer>::new_zeroed(2048 * 2048);
        let read = file.read(&mut buf).unwrap();
        if read == 0 {
            break;
        }
        buf.truncate(read);
        tx.send(buf).unwrap();
    });
    let mut iter = 0;
    let start = irox_time::epoch::UnixTimestamp::now();
    while let Ok(buf) = rx.recv() {
        hasher.write(buf.as_slice());
        iter += 1;
    }
    let h = hasher.finish();
    let elapsed = start.elapsed();
    join.join().unwrap();
    let h = to_hex_str_lower(h.as_ref());
    println!("{h} *{}", path.display());
    let mbs = filelen as f64 / elapsed.as_seconds_f64() / 1024.0 / 1024.0;
    eprintln!("did {filelen} bytes in {elapsed} seconds, in {iter} iterations, {mbs}MB/s");
    Ok(())
}
