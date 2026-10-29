// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's binary: serves the gallery on the address `GALLERY_ADDR`
//! names, `127.0.0.1:3000` by default.

#[tokio::main]
async fn main() -> std::process::ExitCode {
    ferrohealth_ui_gallery::server::main().await
}
