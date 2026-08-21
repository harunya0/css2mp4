mod args;
mod commands;
mod server;
mod ui;

use clap::Parser;

use crate::args::{Cli, Command};
use crate::commands::{handle_export_ymmp, handle_render, handle_serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Render(args) => handle_render(args).await?,
        Command::ExportYmmp(args) => handle_export_ymmp(args).await?,
        Command::Serve(args) => handle_serve(args).await?,
    }

    Ok(())
}
