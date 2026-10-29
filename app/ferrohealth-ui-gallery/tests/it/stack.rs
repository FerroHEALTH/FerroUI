// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery served in-process on a port the system picks.

use std::error::Error;

/// A running gallery.
#[derive(Debug)]
pub(crate) struct Gallery {
    /// Where it listens, `http://127.0.0.1:<port>`.
    base: String,
    /// The task that serves it, stopped when the gallery is dropped.
    task: tokio::task::JoinHandle<()>,
}

impl Gallery {
    /// Serves the gallery.
    pub(crate) async fn start() -> Result<Self, Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let router = ferrohealth_ui_gallery::server::router();
        let task = tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, router).await {
                tracing::error!(%error, "the gallery stopped serving");
            }
        });
        Ok(Self {
            base: format!("http://{addr}"),
            task,
        })
    }

    /// The URL of `path` on the gallery.
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}

impl Drop for Gallery {
    fn drop(&mut self) {
        self.task.abort();
    }
}
