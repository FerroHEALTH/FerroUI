// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The About page: two cards of facts and, where the product has one, its
//! raw self-description.
//!
//! The Versions card opens with the console's own version in the one form
//! the footer writes ([`crate::view::shell::Footer::release`]); the
//! Manufacturer card names the maker, the licence, where to ask for help
//! and the documentation. Under them a product may show its
//! self-description (an OPTIONS document, a `CapabilityStatement`) with Copy
//! and Download, capped at 24 rem with its own scroll. The page has no
//! tabs, no sub-navigation and no side panel.

use leptos::prelude::*;

use crate::highlight::Language;
use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::code::self_description;
use crate::view::shell::MAKER;
use crate::view::surface::{Header, card, definitions, page_heading};
use crate::widget::copy::copy_button;

/// A fact's link: its words and where it leads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The link's words.
    pub label: String,
    /// Where it leads.
    pub href: String,
}

/// What the Manufacturer card says besides the maker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manufacturer {
    /// The product's licence, such as "Business Source License 1.1".
    pub licence: String,
    /// Where to ask for help, such as the maker's address.
    pub support: Link,
    /// The product's documentation, when it has one.
    pub documentation: Option<Link>,
}

/// A product's raw self-description, such as its OPTIONS document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfDescription {
    /// The card's title, also the code box's accessible name.
    pub title: String,
    /// One quiet line on what the document is.
    pub line: String,
    /// The document's language, for its token colours.
    pub language: Language,
    /// The document as the product answers it.
    pub text: String,
    /// The id of its Copy button.
    pub copy_id: String,
    /// The address Download fetches the document from.
    pub download: String,
}

/// What an About page shows.
#[derive(Debug)]
pub struct AboutPage {
    /// The title and its one-line subtitle.
    pub header: Header,
    /// The console and its version, in the footer's form.
    pub release: String,
    /// The Versions card's further rows: the server, the build, the
    /// specifications, the library.
    pub versions: Vec<(String, AnyView)>,
    /// The Manufacturer card.
    pub manufacturer: Manufacturer,
    /// Full-width cards a product adds under the two, such as the
    /// console's own settings.
    pub cards: Vec<AnyView>,
    /// The product's self-description, when it has one.
    pub self_description: Option<SelfDescription>,
}

/// Returns `link` as a fact's value.
fn linked(link: Link) -> AnyView {
    view! { <a href=link.href>{link.label}</a> }.into_any()
}

/// Returns the self-description card: its title with Copy and Download in
/// the card's head, its line, and the document in its capped box.
fn self_description_card(document: SelfDescription) -> AnyView {
    let SelfDescription {
        title,
        line,
        language,
        text,
        copy_id,
        download,
    } = document;
    let panel = self_description(&title, language, &text).into_any();
    // TODO(#220): the copy button is the library's copy island.
    let copy = copy_button(copy_id, tr(Text::DocumentCopy), Signal::stored(text)).into_any();
    view! {
        <section class="fui-card fui-panel">
            <div class="fui-panel-head">
                <h2 class="fui-card-title">{title}</h2>
                <div class="fui-actions">
                    {copy} <a href=download download="" class=Variant::Secondary.small()>
                        {icon(Glyph::DOWNLOAD, "14")}
                        {tr(Text::Download)}
                    </a>
                </div>
            </div>
            <div class="fui-panel-body fui-stack">
                <p class="fui-small fui-muted">{line}</p>
                {panel}
            </div>
        </section>
    }
    .into_any()
}

/// Returns the About page: its header, the Versions and Manufacturer cards
/// side by side, the product's further cards, and its self-description.
#[must_use]
pub fn about_page(page: AboutPage) -> AnyView {
    let AboutPage {
        header,
        release,
        versions,
        manufacturer,
        cards,
        self_description,
    } = page;
    let heading = page_heading(header, ()).into_any();
    let mut rows = vec![(tr(Text::AboutConsole), release.into_any())];
    rows.extend(versions);
    let versions = card(tr(Text::AboutVersions), definitions(rows)).into_any();
    let Manufacturer {
        licence,
        support,
        documentation,
    } = manufacturer;
    let mut facts = vec![
        (tr(Text::AboutMadeBy), MAKER.into_any()),
        (tr(Text::AboutLicence), licence.into_any()),
        (tr(Text::AboutSupport), linked(support)),
    ];
    if let Some(documentation) = documentation {
        facts.push((tr(Text::AboutDocumentation), linked(documentation)));
    }
    let maker = card(tr(Text::AboutManufacturer), definitions(facts)).into_any();
    let document = self_description.map(self_description_card);
    view! {
        {heading}
        <div class="fui-stack">
            <div class="fui-grid fui-grid-start">{versions} {maker}</div>
            {cards}
            {document}
        </div>
    }
    .into_any()
}
