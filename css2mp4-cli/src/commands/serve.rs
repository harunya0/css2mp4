use anyhow::Result;

use crate::args::ServeArgs;
use crate::server::run_server;

/// `serve` サブコマンドの実行処理。
pub async fn handle_serve(args: ServeArgs) -> Result<()> {
    run_server(&args.host, args.port).await?;
    Ok(())
}
