pub mod handlers;
pub mod routes;
pub mod state;

use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::TcpListener;

use crate::server::routes::create_router;
use crate::server::state::AppState;

/// ローカル API サーバーを起動する。
pub async fn run_server(host: &str, port: u16) -> Result<()> {
    let state = AppState::new();
    let app = create_router(state);

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("🚀 css2mp4 API サーバーが起動しました: http://{}", addr);
    println!("   - GET  /api/health            : ヘルスチェック");
    println!("   - POST /api/preview           : 1フレームPNGプレビュー生成");
    println!("   - POST /api/render            : 動画レンダリングタスク作成");
    println!("   - GET  /api/render/:id        : タスク状態取得");
    println!("   - GET  /api/render/:id/events : SSEリアルタイム進捗配信");
    println!("   - GET  /api/render/:id/download: 生成動画ダウンロード");

    axum::serve(listener, app).await?;
    Ok(())
}
