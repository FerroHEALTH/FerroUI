// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! A form that answers on its page, answered in place (FerroUI #546).
//!
//! A check, a run or a load report posts to a route that draws the whole
//! page with the answer in it (`docs/consoles.md` § An action that answers
//! on its page). Its form carries [`Answer::attributes`]: the `answer`
//! module then posts it with `fetch`, takes the answer region by its id from
//! the page the route returns, and moves its content into the same region
//! of this page, which stays the element it was, a polite live region, so
//! the focus stays where it was and a screen reader hears the answer. With
//! [`Answer::with_settle`] the form also posts once a field rests from
//! typing, for a check on the server as you type; an answer to an older
//! request is dropped. With the modules refused, or on an answer that is
//! no `2xx` HTML page holding the region, the form posts as it does today.
//! A form whose post changes state keeps post-redirect-get and is never
//! marked. No specification governs this: our own design.

use crate::html::{Html, escape_into};

/// The job of the module that answers a form in place.
pub const ANSWER_JOB: &str = "answer";

/// The marker of a form answered in place: the ids of its answer regions,
/// the rest before it posts while typing, and any other job of the form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    regions: Vec<String>,
    settle: Option<u32>,
    jobs: Vec<String>,
}

impl Answer {
    /// The marker of a form whose answer is drawn in the element of id
    /// `region`, outside the form.
    #[must_use]
    pub fn new(region: impl Into<String>) -> Self {
        Self {
            regions: vec![region.into()],
            settle: None,
            jobs: Vec::new(),
        }
    }

    /// The same marker with a second region the answer also fills, such as
    /// a count in the page header beside the answer under the form.
    #[must_use]
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.regions.push(region.into());
        self
    }

    /// The same marker posting the form once a field has rested from typing
    /// for `millis` milliseconds; `0` posts only on submit.
    #[must_use]
    pub fn with_settle(mut self, millis: u32) -> Self {
        self.settle = (millis > 0).then_some(millis);
        self
    }

    /// The same marker asking for the module `job` too, such as
    /// `view::shortcuts::KEYS_JOB`, written into the one `data-fui`.
    #[must_use]
    pub fn with_job(mut self, job: impl Into<String>) -> Self {
        self.jobs.push(job.into());
        self
    }

    /// The attributes a console writes inside its `<form>` tag, each with a
    /// leading space: `data-fui`, `data-fui-answer` and, with a rest,
    /// `data-fui-settle`.
    #[must_use]
    pub fn attributes(&self) -> Html {
        let mut out = String::from(" data-fui=\"");
        escape_into(&mut out, ANSWER_JOB);
        for job in &self.jobs {
            out.push(' ');
            escape_into(&mut out, job);
        }
        out.push_str("\" data-fui-answer=\"");
        escape_into(&mut out, &self.regions.join(" "));
        out.push('"');
        if let Some(millis) = self.settle {
            out.push_str(" data-fui-settle=\"");
            out.push_str(&millis.to_string());
            out.push('"');
        }
        Html::raw(out)
    }
}
