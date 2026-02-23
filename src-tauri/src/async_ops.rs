// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Background task manager with progress tracking

use anyhow::Result;
use serde::Serialize;
use std::future::Future;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

/// Background task with progress tracking
pub struct BackgroundTask {
    id: String,
    name: String,
    progress: Arc<Mutex<TaskProgress>>,
    handle: Option<JoinHandle<Result<()>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub is_complete: bool,
    pub error: Option<String>,
}

impl TaskProgress {
    pub fn new(total: usize) -> Self {
        Self {
            current: 0,
            total,
            message: String::new(),
            is_complete: false,
            error: None,
        }
    }

    pub fn get_percent(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.current as f64 / self.total as f64) * 100.0
    }
}

/// Task manager for background operations
pub struct TaskManager {
    tasks: Arc<Mutex<Vec<BackgroundTask>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn spawn_task<F, Fut>(&self, id: String, name: String, f: F) -> Result<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let progress = Arc::new(Mutex::new(TaskProgress::new(100)));
        let handle = tokio::spawn(async move {
            f().await
        });

        let task = BackgroundTask {
            id: id.clone(),
            name,
            progress,
            handle: Some(handle),
        };

        let mut tasks = self.tasks.lock()
            .map_err(|e| anyhow::anyhow!("Task lock poisoned: {}", e))?;
        tasks.push(task);
        Ok(())
    }

    pub fn get_task_progress(&self, id: &str) -> Option<TaskProgress> {
        let tasks = self.tasks.lock().ok()?;
        tasks.iter().find(|t| t.id == id).map(|t| {
            t.progress.lock().unwrap().clone()
        })
    }

    pub fn cancel_task(&self, id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock()
            .map_err(|e| anyhow::anyhow!("Task lock poisoned: {}", e))?;
        if let Some(pos) = tasks.iter().position(|t| t.id == id) {
            let task = tasks.remove(pos);
            if let Some(handle) = task.handle {
                handle.abort();
            }
        }
        Ok(())
    }
}
