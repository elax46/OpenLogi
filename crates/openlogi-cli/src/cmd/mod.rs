use anyhow::Result;
use clap::Subcommand;

pub mod assets;
pub mod list;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List connected Logitech HID++ devices.
    List(list::ListArgs),
    /// Manage assets fetched from assets.openlogi.org.
    #[command(subcommand)]
    Assets(assets::AssetsCmd),
}

impl Command {
    pub async fn run(self) -> Result<()> {
        match self {
            Self::List(args) => list::run(args).await,
            // `assets sync` is blocking HTTP — no need for the async runtime.
            Self::Assets(cmd) => cmd.run(),
        }
    }
}
