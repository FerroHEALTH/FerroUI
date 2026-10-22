// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The fan-out outcome (F5): how each node answered one request that went
//! to many.
//!
//! When any node failed, "Incomplete answer" comes first, in words, with
//! what is missing. Then a summary line: the verdict as a pill and the
//! console's count and timing; with no node asked the verdict says so in
//! the neutral tone, never Complete. Then one row per node: its name, its state
//! as a shape, a colour and a word (family rule 12), the time it took or
//! the rows it gave, and the reason when it failed. A node the request left
//! out is drawn dashed and stays listed, never hidden. No specification
//! governs the component: our own design.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::tone::Tone;
use crate::view::notice::{Announce, notice};

/// How one node answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// It answered: its rows are in the answer.
    Answered,
    /// It was asked and gave no answer, or an error: its rows are missing.
    Failed,
    /// It was not asked, such as a node excluded by an operator or one no
    /// locator named: drawn dashed, missing nothing.
    Skipped,
}

impl Answer {
    /// The tone a node's state is drawn in.
    const fn tone(self) -> Tone {
        match self {
            Self::Answered => Tone::Ok,
            Self::Failed => Tone::Bad,
            Self::Skipped => Tone::Neutral,
        }
    }
}

/// One node of a fan-out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanoutNode {
    /// The node's name, such as "node-a-pub", drawn in the code face.
    pub name: String,
    /// How it answered.
    pub answer: Answer,
    /// Its state in a word with a capital, such as "Answered", "Node
    /// error" or "Excluded".
    pub word: String,
    /// The protocol's own term for the state, such as `node-error`, as a
    /// quiet hint after the word, when the console shows it.
    pub term: Option<String>,
    /// What it gave, such as "3 rows · 41 ms", or "not asked".
    pub measure: String,
    /// Why it failed or was not asked, such as "It answered 503 Service
    /// Unavailable.", under the row.
    pub reason: Option<String>,
}

/// What a fan-out outcome shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fanout {
    /// The accessible name of the rows, such as "What each endpoint did".
    pub label: String,
    /// The summary after the verdict, such as "4 of 5 asked nodes answered
    /// · 412 ms · best effort".
    pub line: String,
    /// What is missing, after "Incomplete answer.", such as "node-b-pub
    /// answered with an error (503), so these rows are not the whole
    /// answer."; drawn only when a node failed.
    pub missing: String,
    /// The nodes, in the console's order.
    pub nodes: Vec<FanoutNode>,
}

/// The verdict of a fan-out: whole, partial, failed, or not asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// A node answered and none failed.
    Complete,
    /// Some nodes answered and some failed.
    Incomplete,
    /// Every node asked failed.
    Failed,
    /// No node was asked: every node was left out, or there is none, so
    /// nothing answered and the answer holds no rows.
    Unasked,
}

impl Verdict {
    /// The verdict of `nodes`.
    #[must_use]
    pub fn of(nodes: &[FanoutNode]) -> Self {
        let failed = nodes
            .iter()
            .filter(|node| node.answer == Answer::Failed)
            .count();
        let answered = nodes
            .iter()
            .filter(|node| node.answer == Answer::Answered)
            .count();
        match (failed, answered) {
            (0, 0) => Self::Unasked,
            (0, _) => Self::Complete,
            (_, 0) => Self::Failed,
            _ => Self::Incomplete,
        }
    }

    /// The tone the verdict is drawn in.
    const fn tone(self) -> Tone {
        match self {
            Self::Complete => Tone::Ok,
            Self::Incomplete => Tone::Warn,
            Self::Failed => Tone::Bad,
            Self::Unasked => Tone::Neutral,
        }
    }

    /// The verdict in a word.
    fn word(self) -> String {
        tr(match self {
            Self::Complete => Text::FanoutComplete,
            Self::Incomplete => Text::FanoutIncomplete,
            Self::Failed => Text::FanoutFailed,
            Self::Unasked => Text::FanoutUnasked,
        })
    }
}

/// Returns a node's name with a break opportunity after each dot, so a
/// long host name wraps between its labels.
fn breakable(name: &str) -> impl IntoView + use<> {
    let mut pieces = Vec::new();
    let mut rest = name;
    while let Some(at) = rest.find('.') {
        let (head, tail) = rest.split_at(at + 1);
        pieces.push(view! { {head.to_owned()} <wbr /> }.into_any());
        rest = tail;
    }
    pieces.push(rest.to_owned().into_any());
    pieces
}

/// Returns one node's row: its name and state on one line, what it gave
/// under them, and the reason last.
fn node_row(node: FanoutNode) -> impl IntoView {
    let FanoutNode {
        name,
        answer,
        word,
        term,
        measure,
        reason,
    } = node;
    let tone = answer.tone();
    let class = format!("fui-fanout-item {}", tone.class());
    let term = term.map(|term| view! { <span class="fui-fanout-term"><code>{term}</code></span> });
    let reason = reason.map(|reason| view! { <span class="fui-fanout-error">{reason}</span> });
    view! {
        <li class=class class:is-excluded=answer == Answer::Skipped>
            <span class="fui-fanout-line">
                <code>{breakable(&name)}</code>
                <span class=tone.pill()>{word}</span>
                {term}
            </span>
            <span class="fui-fanout-meta">{measure}</span>
            {reason}
        </li>
    }
}

/// Returns the fan-out outcome `fanout`: "Incomplete answer" in words when
/// a node failed, the verdict with the summary line, and one row per node.
///
/// Put it in the answer's panel section, above the rows the request
/// returned.
#[must_use]
pub fn fanout(fanout: Fanout) -> AnyView {
    let Fanout {
        label,
        line,
        missing,
        nodes,
    } = fanout;
    let verdict = Verdict::of(&nodes);
    let incomplete = matches!(verdict, Verdict::Incomplete | Verdict::Failed).then(|| {
        let tone = if verdict == Verdict::Failed {
            Tone::Bad
        } else {
            Tone::Warn
        };
        notice(
            tone,
            Announce::Status,
            None,
            view! {
                <strong>{tr(Text::FanoutIncompleteAnswer)}</strong>
                " "
                {missing}
            },
        )
    });
    let rows = nodes.into_iter().map(node_row).collect_view();
    view! {
        <div class="fui-fanout">
            {incomplete}
            <p class="fui-fanout-summary">
                <span class=verdict.tone().pill()>{verdict.word()}</span>
                <span class="fui-muted">{line}</span>
            </p>
            <ul class="fui-fanout-list" aria-label=label>
                {rows}
            </ul>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::{Answer, Fanout, FanoutNode, Verdict, fanout};

    fn node(name: &str, answer: Answer) -> FanoutNode {
        FanoutNode {
            name: name.to_owned(),
            answer,
            word: String::from("Excluded"),
            term: None,
            measure: String::from("not asked"),
            reason: None,
        }
    }

    fn drawn(nodes: Vec<FanoutNode>) -> String {
        Owner::new().with(|| {
            fanout(Fanout {
                label: String::from("What each node did"),
                line: String::from("0 of 2 nodes asked"),
                missing: String::new(),
                nodes,
            })
            .to_html()
        })
    }

    #[test]
    fn no_node_asked_is_never_complete() {
        let skipped = node("node-a", Answer::Skipped);
        assert_eq!(
            Verdict::Unasked,
            Verdict::of(&[skipped.clone(), skipped.clone()]),
            "every node left out"
        );
        assert_eq!(Verdict::Unasked, Verdict::of(&[]), "no node at all");
        let shown = drawn(vec![skipped]);
        assert!(!shown.contains(">Complete<"), "{shown}");
        assert!(shown.contains(">No node asked<"), "{shown}");
        assert!(shown.contains("fui-pill fui-tone-neutral"), "{shown}");
        assert!(
            !shown.contains("Incomplete answer."),
            "nothing is missing: {shown}"
        );
    }

    #[test]
    fn a_long_host_name_breaks_after_its_dots() {
        let shown = drawn(vec![node(
            "node-eu-west-1.ferrofed.internal.example.org",
            Answer::Answered,
        )]);
        assert!(
            shown.contains("node-eu-west-1.<wbr>ferrofed.<wbr>internal.<wbr>example.<wbr>org"),
            "{shown}"
        );
    }
}
