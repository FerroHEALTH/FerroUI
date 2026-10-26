// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! An action as a console writes one.
//!
//! The server function's plain post, with no WebAssembly and no `Referer`,
//! returns to the page the form names (`ferrohealth_ui::widget::action`,
//! #449). No page draws it; the HTTP tests post to it as a form with no
//! WebAssembly does.
#![cfg_attr(
    target_arch = "wasm32",
    allow(
        clippy::unused_async_trait_impl,
        reason = "the server function macro writes the browser half's call as an async trait method; only that target has it"
    )
)]

use leptos::prelude::*;

/// Acknowledges `note`, refusing an empty one, and sends a plain post back
/// to `ferroui_return`, the page its form was drawn on.
///
/// # Errors
///
/// When `note` is empty, or the server function cannot be reached.
#[server(endpoint = "acknowledge")]
pub async fn acknowledge(
    /// What is acknowledged.
    note: String,
    /// The page the form was drawn on (`ferrohealth_ui::widget::action::RETURN`).
    ferroui_return: Option<String>,
) -> Result<(), ServerFnError> {
    use ferrohealth_ui::widget::action::{plain_return, with_error};
    use leptos::server_fn::ServerFn as _;
    let outcome = if note.trim().is_empty() {
        Err(ServerFnError::new("the note is empty"))
    } else {
        Ok(())
    };
    let headers: http::HeaderMap = leptos_axum::extract().await?;
    let accept = headers
        .get(http::header::ACCEPT)
        .and_then(|value| value.to_str().ok());
    if let Some(back) = plain_return(ferroui_return.as_deref(), accept) {
        let to = match &outcome {
            Ok(()) => back,
            Err(error) => with_error(&back, Acknowledge::PATH, error.clone()),
        };
        leptos_axum::redirect(&to);
    }
    outcome
}
