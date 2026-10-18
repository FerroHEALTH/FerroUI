// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery served in-process on a port the system picks, from the
//! release site bundle `scripts/release/gallery-site.sh` builds.

use std::error::Error;
use std::path::PathBuf;

/// A running gallery.
#[derive(Debug)]
pub(crate) struct Gallery {
    /// Where it listens, `http://127.0.0.1:<port>`.
    base: String,
    /// The task that serves it, stopped when the gallery is dropped.
    task: tokio::task::JoinHandle<()>,
}

impl Gallery {
    /// Serves the gallery from the workspace's `target/site`.
    pub(crate) async fn start() -> Result<Self, Box<dyn Error>> {
        let site = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/site");
        let bundle = site.join("pkg").join("ferrohealth-ui-gallery.wasm");
        if !bundle.is_file() {
            return Err(format!(
                "{} is missing: build it with scripts/release/gallery-site.sh --release",
                bundle.display()
            )
            .into());
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let site = site
            .to_str()
            .ok_or("the site root is no UTF-8 path")?
            .to_owned();
        let router = ferrohealth_ui_gallery::server::router(
            ferrohealth_ui_gallery::server::leptos_options(&site, addr),
        );
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
