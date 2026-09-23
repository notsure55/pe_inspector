use anyhow::Result;
use pe_inspector_um::Process;
use windows_core::w;

fn main() -> Result<()> {
    let process = Process::from_name("cs2.exe")?;

    dbg!(&process);

    Ok(())
}
