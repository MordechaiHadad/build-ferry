pub mod cli;
pub mod modules;
use eyre::Result;

fn main() -> Result<()> {
    cli::start()
}
