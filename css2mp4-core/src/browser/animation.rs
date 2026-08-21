use std::collections::HashSet;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::animation::{
    EnableParams as AnimationEnableParams, EventAnimationStarted, SeekAnimationsParams,
    SetPlaybackRateParams,
};
use chromiumoxide::Page;
use futures::StreamExt;

use crate::error::Result;

/// CDP `Animation` ドメインを用いた仮想時間とアニメーション再生の制御モジュール。
pub struct AnimationController;

impl AnimationController {
    /// ページ内のアニメーションを検出し、自動再生を停止してアニメーション ID リストを返します。
    pub async fn initialize_and_collect_ids(page: &Page, target_url: &str) -> Result<Vec<String>> {
        // ページ読み込み前に Animation ドメインを有効化
        page.execute(AnimationEnableParams {}).await?;

        let mut animation_events = page.event_listener::<EventAnimationStarted>().await?;

        // 対象ページへ遷移
        page.goto(target_url).await?;

        // ページロード直後に開始されるアニメーションの ID を収集（300ms 待機）
        let mut animation_ids: HashSet<String> = HashSet::new();
        let collect_deadline = tokio::time::sleep(Duration::from_millis(300));
        tokio::pin!(collect_deadline);

        loop {
            tokio::select! {
                _ = &mut collect_deadline => break,
                maybe_event = animation_events.next() => {
                    match maybe_event {
                        Some(evt) => {
                            animation_ids.insert(evt.animation.id.clone());
                        }
                        None => break,
                    }
                }
            }
        }

        // タイムラインの自動再生を停止（以降は seekAnimations でシーク制御）
        page.execute(SetPlaybackRateParams {
            playback_rate: 0.0,
        })
        .await?;

        Ok(animation_ids.into_iter().collect())
    }

    /// 収集されたアニメーション ID 群に対して、指定時刻（ミリ秒）へシークします。
    pub async fn seek(page: &Page, animation_ids: &[String], time_ms: f64) -> Result<()> {
        if !animation_ids.is_empty() {
            let seek_params = SeekAnimationsParams {
                animations: animation_ids.to_vec(),
                current_time: time_ms,
            };
            page.execute(seek_params).await?;
        }
        Ok(())
    }
}
