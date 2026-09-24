use anyhow::Result;
use um::Process;

fn main() -> Result<()> {
    let process = Process::from_name("test_process.exe")?;

    dbg!(&process);

    Ok(())
}
