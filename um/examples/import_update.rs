use anyhow::Result;
use shared::Va;
use um::Process;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().into_iter().collect();

    if args.len() < 2 {
        eprintln!("USAGE: {} [address]", args[0]);
    }

    let va = Va(usize::from_str_radix(&args[1], 16)?);

    let process = Process::from_name("test_process.exe")?;
    let health: u32 = process.read_virtual_memory(va)?;

    println!("{health}");

    dbg!(&process);

    Ok(())
}
