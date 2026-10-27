// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Actions as a console writes them.
//!
//! The plain post of `acknowledge`, with no WebAssembly and no `Referer`,
//! returns to the page the form names (`ferrohealth_ui::widget::action`,
//! #449); no page draws it, and the HTTP tests post to it as a form with no
//! WebAssembly does. `count_words` answers on its page: the forms page draws
//! its form with `answer_field`, and a plain post draws that page again
//! with the count and the posted note (#485).
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
        Err(ServerFnError::new(crate::locale::tr(
            crate::locale::Msg::AcknowledgeEmpty,
        )))
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

/// Counts the words of `note`, refusing an empty one.
///
/// Its form draws `ferrohealth_ui::widget::action::answer_field`, so a plain
/// post draws the forms page with the count; the function itself redirects
/// nowhere.
///
/// # Errors
///
/// When `note` is empty, or the server function cannot be reached.
#[cfg_attr(
    not(target_arch = "wasm32"),
    allow(
        clippy::unused_async,
        reason = "the server function macro wants the function async, and this one awaits nothing; the macro's expansion carries the lint"
    )
)]
#[server(endpoint = "count_words")]
pub async fn count_words(
    /// The note whose words are counted.
    note: String,
) -> Result<u32, ServerFnError> {
    let words = note.split_whitespace().count();
    if words == 0 {
        return Err(ServerFnError::new(crate::locale::tr(
            crate::locale::Msg::AcknowledgeEmpty,
        )));
    }
    u32::try_from(words).map_err(|error| ServerFnError::new(error.to_string()))
}
