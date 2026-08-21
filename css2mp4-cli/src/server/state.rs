use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

/// タスクの状態。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Rendering,
    Completed,
    Failed,
}

/// SSE でクライアントへ送信する進捗イベント。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub frame: u32,
    pub total_frames: u32,
    pub percent: f64,
    pub status: TaskStatus,
    pub error: Option<String>,
}

/// 個別タスクの情報。
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub status: TaskStatus,
    pub output_path: PathBuf,
    pub total_frames: u32,
    pub current_frame: u32,
    pub error: Option<String>,
    pub tx: broadcast::Sender<ProgressEvent>,
}

/// サーバー全体の共有状態。
#[derive(Clone)]
pub struct AppState {
    pub tasks: Arc<RwLock<HashMap<String, TaskInfo>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 新規タスクを登録する。
    pub async fn create_task(&self, id: String, output_path: PathBuf, total_frames: u32) -> broadcast::Receiver<ProgressEvent> {
        let (tx, rx) = broadcast::channel(100);
        let task = TaskInfo {
            id: id.clone(),
            status: TaskStatus::Queued,
            output_path,
            total_frames,
            current_frame: 0,
            error: None,
            tx,
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(id, task);
        rx
    }

    /// タスクの進捗を更新・ブロードキャストする。
    pub async fn update_progress(&self, id: &str, frame: u32, total_frames: u32) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.current_frame = frame;
            task.status = TaskStatus::Rendering;
            let percent = if total_frames > 0 {
                (frame as f64 / total_frames as f64) * 100.0
            } else {
                0.0
            };
            let event = ProgressEvent {
                frame,
                total_frames,
                percent,
                status: TaskStatus::Rendering,
                error: None,
            };
            let _ = task.tx.send(event);
        }
    }

    /// タスクを完了状態にする。
    pub async fn complete_task(&self, id: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.status = TaskStatus::Completed;
            let event = ProgressEvent {
                frame: task.total_frames,
                total_frames: task.total_frames,
                percent: 100.0,
                status: TaskStatus::Completed,
                error: None,
            };
            let _ = task.tx.send(event);
        }
    }

    /// タスクをエラー終了状態にする。
    pub async fn fail_task(&self, id: &str, error: String) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.status = TaskStatus::Failed;
            task.error = Some(error.clone());
            let event = ProgressEvent {
                frame: task.current_frame,
                total_frames: task.total_frames,
                percent: if task.total_frames > 0 {
                    (task.current_frame as f64 / task.total_frames as f64) * 100.0
                } else {
                    0.0
                },
                status: TaskStatus::Failed,
                error: Some(error),
            };
            let _ = task.tx.send(event);
        }
    }
}
