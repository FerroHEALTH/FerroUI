// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The script modules, as text a console serves: one plain ES module per
//! job under the crate's `scripts/`, and the loader that imports them.
//!
//! Every page works with no script: the HTML is complete and a link or a
//! form is behind every action. The modules add behaviour on top. The
//! shell writes one `<script type="module">` for the loader ([`LOADER`]),
//! which marks `<html data-fui-script>`, finds every element carrying
//! `data-fui="<job>"`, imports `./<job>.js` from the same directory and
//! calls its `enhance(element)`, and watches the document for elements
//! added later. A page loads only the modules its components asked for,
//! with no bookkeeping in Rust: the attribute is the request. No inline
//! script and no `eval`, so `script-src 'self'` holds
//! (`scripts/checks/script-budget.sh`). No specification governs this: our
//! own design (FerroUI #510).

/// One module: its file name under the served directory and its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Module {
    /// The file name, such as `menu.js`.
    pub file: &'static str,
    /// The module's text.
    pub text: &'static str,
}

/// The loader every page loads, `fui.js`.
pub const LOADER: Module = Module {
    file: "fui.js",
    text: include_str!("../scripts/fui.js"),
};

/// Every module, the loader first, in the order of their file names.
pub const MODULES: &[Module] = &[
    LOADER,
    Module {
        file: "calendar.js",
        text: include_str!("../scripts/calendar.js"),
    },
    Module {
        file: "chart.js",
        text: include_str!("../scripts/chart.js"),
    },
    Module {
        file: "combobox.js",
        text: include_str!("../scripts/combobox.js"),
    },
    Module {
        file: "copy.js",
        text: include_str!("../scripts/copy.js"),
    },
    Module {
        file: "countdown.js",
        text: include_str!("../scripts/countdown.js"),
    },
    Module {
        file: "date.js",
        text: include_str!("../scripts/date.js"),
    },
    Module {
        file: "date_range.js",
        text: include_str!("../scripts/date_range.js"),
    },
    Module {
        file: "dialog.js",
        text: include_str!("../scripts/dialog.js"),
    },
    Module {
        file: "document.js",
        text: include_str!("../scripts/document.js"),
    },
    Module {
        file: "drawer.js",
        text: include_str!("../scripts/drawer.js"),
    },
    Module {
        file: "editor.js",
        text: include_str!("../scripts/editor.js"),
    },
    Module {
        file: "filter.js",
        text: include_str!("../scripts/filter.js"),
    },
    Module {
        file: "highlight.js",
        text: include_str!("../scripts/highlight.js"),
    },
    Module {
        file: "identifier.js",
        text: include_str!("../scripts/identifier.js"),
    },
    Module {
        file: "keep.js",
        text: include_str!("../scripts/keep.js"),
    },
    Module {
        file: "masked.js",
        text: include_str!("../scripts/masked.js"),
    },
    Module {
        file: "menu.js",
        text: include_str!("../scripts/menu.js"),
    },
    Module {
        file: "picker.js",
        text: include_str!("../scripts/picker.js"),
    },
    Module {
        file: "preference.js",
        text: include_str!("../scripts/preference.js"),
    },
    Module {
        file: "shortcuts.js",
        text: include_str!("../scripts/shortcuts.js"),
    },
    Module {
        file: "side_panel.js",
        text: include_str!("../scripts/side_panel.js"),
    },
    Module {
        file: "tabs.js",
        text: include_str!("../scripts/tabs.js"),
    },
    Module {
        file: "toast.js",
        text: include_str!("../scripts/toast.js"),
    },
    Module {
        file: "user_menu.js",
        text: include_str!("../scripts/user_menu.js"),
    },
];

/// The module named `file`, if the library ships it.
#[must_use]
pub fn module(file: &str) -> Option<Module> {
    MODULES.iter().copied().find(|module| module.file == file)
}

/// The attribute a component marks its root with to ask for a module: the
/// value is the job, the module's file name without `.js`.
pub const HOOK: &str = "data-fui";

/// The attribute the loader writes on `<html>` once it runs, which the
/// stylesheet reads to show a control that needs script and a journey
/// waits for.
pub const MARK: &str = "data-fui-script";

#[cfg(test)]
mod tests {
    use super::{LOADER, MODULES, module};

    #[test]
    fn the_loader_comes_first_and_every_module_is_named_by_its_file() {
        assert_eq!(Some(LOADER), MODULES.first().copied());
        let mut files: Vec<&str> = MODULES.iter().skip(1).map(|m| m.file).collect();
        let sorted = {
            let mut sorted = files.clone();
            sorted.sort_unstable();
            sorted
        };
        assert_eq!(sorted, files, "the modules are listed in file order");
        files.dedup();
        assert_eq!(MODULES.len() - 1, files.len(), "no file twice");
        assert_eq!(Some(LOADER), module("fui.js"));
        assert_eq!(None, module("no-such.js"));
    }

    #[test]
    fn no_module_evaluates_text_or_reaches_another_origin() {
        for module in MODULES {
            assert!(!module.text.contains("eval("), "{}", module.file);
            assert!(!module.text.contains("new Function"), "{}", module.file);
            assert!(!module.text.contains("https://"), "{}", module.file);
            assert!(
                module.text.starts_with("// SPDX-FileCopyrightText"),
                "{}",
                module.file
            );
        }
    }
}
