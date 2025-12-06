use anyhow::{Ok, Result};
use cryptography_prover::get_verifying_key;

fn main() -> Result<()> {
    let vkey = get_verifying_key()?;
    std::fs::write("vkey.bin", &vkey)?;
    Ok(())
}
