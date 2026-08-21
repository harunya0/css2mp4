use clap::Parser;

/// `serve` サブコマンドの引数定義。
#[derive(Parser, Debug)]
pub struct ServeArgs {
    /// バインドするホストアドレス。
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// リッスンするポート番号。
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}
