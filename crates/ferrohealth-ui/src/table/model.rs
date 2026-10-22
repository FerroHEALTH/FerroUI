// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The paging and sorting arithmetic. Every number a request carries is user
//! input, so each is parsed defensively and clamped, and every sum
//! saturates. No specification governs paging: our own design.

use leptos_router::params::ParamsMap;

use crate::i18n::{Message, Text, tr, tr_with};

/// The rows a page shows when the request names no size.
pub const DEFAULT_SIZE: u32 = 25;

/// The sizes the pager offers as one-click choices.
pub const SIZES: [u32; 3] = [25, 50, 100];

/// The largest size a request may name: a hand-typed `?size=100000` never
/// renders a hundred thousand rows into one page.
pub const MAX_SIZE: u32 = 200;

/// The query key of the 1-based page.
pub const PAGE_KEY: &str = "page";

/// The query key of the page size.
pub const SIZE_KEY: &str = "size";

/// The query key of the sorted column.
pub const SORT_KEY: &str = "sort";

/// The query key of the sort's direction.
pub const ORDER_KEY: &str = "order";

/// What a listing names its rows by: "value set", "value sets".
///
/// A console names its rows in its own words: a message of its own catalog
/// that selects the form by `$count` (a Fluent selector over the CLDR plural
/// categories, so the library holds no plural rule of any language), with
/// two attributes, each a whole sentence in the console's words: `.empty`
/// for a listing with no row and `.opening` for the plural as a sentence
/// opens with it.
///
/// ```ftl
/// value-sets =
///     { $count ->
///         [one] value set
///        *[other] value sets
///     }
///     .empty = No value sets
///     .opening = Value sets
/// ```
///
/// The library's own noun, [`Noun::ROWS`], is drawn from the library's
/// catalog. A noun is `Copy`; it is made where the console's catalog is in
/// reach rather than as a `const`, since a catalog is a lazily built static.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Noun {
    message: Option<Message>,
}

impl Noun {
    /// Rows, the noun of a listing that names nothing else.
    pub const ROWS: Self = Self { message: None };

    /// The noun the console's message `message` names.
    #[must_use]
    pub const fn new(message: Message) -> Self {
        Self {
            message: Some(message),
        }
    }

    /// The word for `count` rows in the page's locale.
    #[must_use]
    pub fn word(self, count: u32) -> String {
        let args = [("count", count.into())];
        match self.message {
            Some(message) => message.text_with(&args),
            None => tr_with(Text::Rows, &args),
        }
    }

    /// The sentence for a listing with no row: "No value sets".
    #[must_use]
    pub fn empty(self) -> String {
        match self.message {
            Some(message) => message.attribute("empty", &[("count", 0.into())]),
            None => tr(Text::RangeEmpty),
        }
    }

    /// The word for several rows as a sentence opens with it: "Value sets".
    #[must_use]
    pub fn opening(self) -> String {
        match self.message {
            Some(message) => message.attribute("opening", &[("count", 2.into())]),
            None => tr(Text::RowsOpening),
        }
    }
}

/// The page sizes a listing offers, the one it starts at, and the largest a
/// request may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sizes {
    /// The one-click choices; none when the size is a setting of the
    /// console's.
    pub choices: &'static [u32],
    /// The size when the request names none.
    pub default: u32,
    /// The bound a request's size is clamped to.
    pub max: u32,
}

impl Sizes {
    /// 25, 50 and 100, starting at 25, clamped to 200.
    pub const STANDARD: Self = Self {
        choices: &SIZES,
        default: DEFAULT_SIZE,
        max: MAX_SIZE,
    };
}

impl Default for Sizes {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// Where a listing keeps its page, size and sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StateSource {
    /// In the URL's query: shareable, walked by back and forward, working
    /// before the page hydrates.
    #[default]
    Url,
    /// Out of the URL and the history: the listing's forms post it and the
    /// console keeps it in its session, for a screen that can show patient
    /// data.
    Session,
}

/// The page a listing shows and its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paging {
    /// The 1-based page.
    pub page: u32,
    /// The rows a page shows.
    pub size: u32,
}

impl Default for Paging {
    fn default() -> Self {
        Self {
            page: 1,
            size: DEFAULT_SIZE,
        }
    }
}

impl Paging {
    /// The paging the query values `page` and `size` name: a missing or
    /// unreadable page is the first, a missing or unreadable size the
    /// default, and a size is clamped to `1..=MAX_SIZE`.
    #[must_use]
    pub fn read(page: Option<&str>, size: Option<&str>) -> Self {
        Self::read_within(page, size, Sizes::STANDARD)
    }

    /// The paging the query values `page` and `size` name under the
    /// console's `sizes`: a missing or unreadable size is `sizes.default`,
    /// and a size is clamped to `1..=sizes.max`.
    #[must_use]
    pub fn read_within(page: Option<&str>, size: Option<&str>, sizes: Sizes) -> Self {
        // NOTE: no specification governs this: our own design; a page or size that
        // is no number is the value not given, never an error page.
        let page = page
            .and_then(|page| page.trim().parse::<u32>().ok())
            .unwrap_or(1)
            .max(1);
        let size = size
            .and_then(|size| size.trim().parse::<u32>().ok())
            .unwrap_or(sizes.default)
            .clamp(1, sizes.max.max(1));
        Self { page, size }
    }

    /// The paging the query `query` names.
    #[must_use]
    pub fn from_query(query: &ParamsMap) -> Self {
        Self::from_query_within(query, Sizes::STANDARD)
    }

    /// The paging the query `query` names under the console's `sizes`.
    #[must_use]
    pub fn from_query_within(query: &ParamsMap, sizes: Sizes) -> Self {
        Self::read_within(query.get_str(PAGE_KEY), query.get_str(SIZE_KEY), sizes)
    }

    /// This paging written into `query`: the first page and the default size
    /// are written as their absence, so the first page's URL is the plain
    /// path.
    pub fn write(self, query: &mut ParamsMap) {
        self.write_within(query, Sizes::STANDARD);
    }

    /// This paging written into `query`, the size `sizes.default` written
    /// as its absence.
    pub fn write_within(self, query: &mut ParamsMap, sizes: Sizes) {
        if self.page <= 1 {
            drop(query.remove(PAGE_KEY));
        } else {
            query.replace(PAGE_KEY, self.page.to_string());
        }
        if self.size == sizes.default {
            drop(query.remove(SIZE_KEY));
        } else {
            query.replace(SIZE_KEY, self.size.to_string());
        }
    }

    /// The same page size at page `page`.
    #[must_use]
    pub const fn at(self, page: u32) -> Self {
        Self {
            page,
            size: self.size,
        }
    }

    /// The first page at size `size`, since the rows a page holds move.
    #[must_use]
    pub const fn sized(size: u32) -> Self {
        Self { page: 1, size }
    }
}

/// The window of rows a page shows over a listing of `total` rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// The page shown: the one asked for, clamped to the last page.
    pub page: u32,
    /// The number of pages, at least one.
    pub pages: u32,
    /// The rows a page shows.
    pub size: u32,
    /// The rows of the whole listing.
    pub total: u32,
    /// The index of the page's first row.
    pub start: u32,
    /// The index one past the page's last row.
    pub end: u32,
}

impl Window {
    /// The window `paging` asks for over `total` rows; a page past the end
    /// shows the last page, never nothing.
    #[must_use]
    pub fn over(total: u32, paging: Paging) -> Self {
        Self::within(total, paging, MAX_SIZE)
    }

    /// The window `paging` asks for over `total` rows, the size clamped to
    /// the console's bound `max`.
    #[must_use]
    pub fn within(total: u32, paging: Paging, max: u32) -> Self {
        let size = paging.size.clamp(1, max.max(1));
        let pages = total.div_ceil(size).max(1);
        let page = paging.page.clamp(1, pages);
        let start = (page - 1).saturating_mul(size).min(total);
        let end = start.saturating_add(size).min(total);
        Self {
            page,
            pages,
            size,
            total,
            start,
            end,
        }
    }

    /// The paging of this window.
    #[must_use]
    pub const fn paging(self) -> Paging {
        Paging {
            page: self.page,
            size: self.size,
        }
    }

    /// The rows of `rows` this window shows, for a listing paged in hand.
    #[must_use]
    pub fn slice<T>(self, rows: &[T]) -> &[T] {
        let start = usize::try_from(self.start).unwrap_or(usize::MAX);
        let end = usize::try_from(self.end).unwrap_or(usize::MAX);
        rows.get(start..end.min(rows.len())).unwrap_or_default()
    }

    /// The range label: "1-25 of 312", or "No rows" for an empty listing.
    #[must_use]
    pub fn range_label(self) -> String {
        if self.total == 0 {
            return tr(Text::RangeEmpty);
        }
        tr_with(
            Text::Range,
            &[
                ("first", self.start.saturating_add(1).into()),
                ("last", self.end.into()),
                ("total", self.total.into()),
            ],
        )
    }

    /// The range label naming the rows by `noun`: "1-25 of 312 value
    /// sets", "1-1 of 1 value set", or "No value sets".
    #[must_use]
    pub fn range_of(self, noun: Noun) -> String {
        if self.total == 0 {
            return noun.empty();
        }
        tr_with(
            Text::RangeOf,
            &[
                ("first", self.start.saturating_add(1).into()),
                ("last", self.end.into()),
                ("total", self.total.into()),
                ("noun", noun.word(self.total).into()),
            ],
        )
    }

    /// Whether the page before this one exists.
    #[must_use]
    pub const fn has_previous(self) -> bool {
        self.page > 1
    }

    /// Whether the page after this one exists.
    #[must_use]
    pub const fn has_next(self) -> bool {
        self.page < self.pages
    }
}

/// The sentence a listing adds when the server counted more rows than it
/// sent: `held` rows in hand, `stated` counted (a FHIR search's
/// `Bundle.total`). `None` when the two agree or nothing was stated.
#[must_use]
pub fn partial_note(held: u32, stated: Option<u32>, noun: Noun) -> Option<String> {
    let stated = stated.filter(|stated| *stated != held)?;
    Some(tr_with(
        Text::PartialNote,
        &[
            ("stated", stated.into()),
            ("noun", noun.word(stated).into()),
            ("held", held.into()),
        ],
    ))
}

/// The window of a listing whose total the server does not report: the
/// page asked for, and the rows that page came back with.
///
/// A previous page exists past the first; a next one may exist when the
/// page came back full, which is all a server that answers no total tells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenWindow {
    /// The 1-based page shown.
    pub page: u32,
    /// The rows a page asks for.
    pub size: u32,
    /// The index of the page's first row: the offset the console asks the
    /// server for.
    pub start: u32,
    /// The rows the page came back with.
    pub held: u32,
}

impl OpenWindow {
    /// The window of `paging`, whose page came back with `held` rows.
    #[must_use]
    pub fn of(paging: Paging, held: u32) -> Self {
        let page = paging.page.max(1);
        let size = paging.size.max(1);
        Self {
            page,
            size,
            start: (page - 1).saturating_mul(size),
            held: held.min(size),
        }
    }

    /// The paging of this window.
    #[must_use]
    pub const fn paging(self) -> Paging {
        Paging {
            page: self.page,
            size: self.size,
        }
    }

    /// Whether the page before this one exists.
    #[must_use]
    pub const fn has_previous(self) -> bool {
        self.page > 1
    }

    /// Whether a page after this one may exist: this one came back full.
    #[must_use]
    pub const fn has_next(self) -> bool {
        self.held >= self.size
    }

    /// The range label with no total: "Value sets 26 to 50", or "No value
    /// sets" when the page came back empty.
    #[must_use]
    pub fn range_of(self, noun: Noun) -> String {
        if self.held == 0 {
            return noun.empty();
        }
        tr_with(
            Text::OpenRangeOf,
            &[
                ("nouns", noun.opening().into()),
                ("first", self.start.saturating_add(1).into()),
                ("last", self.start.saturating_add(self.held).into()),
            ],
        )
    }
}

/// The direction of a sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Smallest first.
    #[default]
    Ascending,
    /// Largest first.
    Descending,
}

impl Direction {
    /// The direction's query value.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }

    /// The value of `aria-sort` on the header of a column sorted this way
    /// (WAI-ARIA 1.2, `aria-sort`).
    #[must_use]
    pub const fn aria(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }
}

/// The column a listing is sorted by, and the direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort {
    /// The sorted column's key.
    pub column: String,
    /// The direction.
    pub direction: Direction,
}

impl Sort {
    /// The sort the query values `column` and `order` name, when the column
    /// is one of `columns`; `None` for no sort or a column the listing does
    /// not sort by.
    #[must_use]
    pub fn read(column: Option<&str>, order: Option<&str>, columns: &[&str]) -> Option<Self> {
        let column = column.filter(|column| columns.contains(column))?;
        let direction = if order == Some(Direction::Descending.key()) {
            Direction::Descending
        } else {
            Direction::Ascending
        };
        Some(Self {
            column: column.to_owned(),
            direction,
        })
    }

    /// The sort the query `query` names, over the sortable `columns`.
    #[must_use]
    pub fn from_query(query: &ParamsMap, columns: &[&str]) -> Option<Self> {
        Self::read(query.get_str(SORT_KEY), query.get_str(ORDER_KEY), columns)
    }

    /// The `aria-sort` of the column `column` under the sort `sort`.
    #[must_use]
    pub fn aria(sort: Option<&Self>, column: &str) -> &'static str {
        match sort {
            Some(sort) if sort.column == column => sort.direction.aria(),
            _ => "none",
        }
    }

    /// The sort a click on the header of `column` asks for: that column
    /// ascending, or the other direction when it is already sorted.
    #[must_use]
    pub fn toggled(sort: Option<&Self>, column: &str) -> Self {
        let direction = match sort {
            Some(sort) if sort.column == column && sort.direction == Direction::Ascending => {
                Direction::Descending
            }
            _ => Direction::Ascending,
        };
        Self {
            column: column.to_owned(),
            direction,
        }
    }

    /// This sort written into `query`, from the first page.
    pub fn write(&self, query: &mut ParamsMap) {
        query.replace(SORT_KEY, self.column.clone());
        query.replace(ORDER_KEY, self.direction.key().to_owned());
        drop(query.remove(PAGE_KEY));
    }
}

/// The href of `path` with the query `query`, encoded by the router.
#[must_use]
pub fn href(path: &str, query: &ParamsMap) -> String {
    format!("{path}{}", query.to_query_string())
}

/// The keys and values of `query` in order, for the hidden fields of a form
/// that keeps the rest of a listing's query when it submits (a sort that
/// keeps the filter and the size).
#[must_use]
pub fn fields(query: &ParamsMap) -> Vec<(String, String)> {
    query
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_owned()))
        .collect()
}

/// The columns of `columns` that some row of `rows` fills, in their order.
///
/// A table then draws no column that shows nothing in every row. `filled`
/// says whether a row has a value for a column, and a column whose key is in
/// `keep` stays whatever the rows hold.
#[must_use]
pub fn filled_columns<R>(
    columns: &[crate::table::view::Column],
    rows: &[R],
    keep: &[&str],
    filled: impl Fn(&R, &str) -> bool,
) -> Vec<crate::table::view::Column> {
    columns
        .iter()
        .filter(|column| {
            keep.contains(&column.key.as_ref()) || rows.iter().any(|row| filled(row, &column.key))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_column_no_row_fills_is_left_out_unless_kept() {
        use crate::table::view::Column;
        let columns = [
            Column::text("code", "Code"),
            Column::text("flags", "Flags"),
            Column::text("use", "Use"),
        ];
        let rows = [("a", ""), ("b", "")];
        let shown = super::filled_columns(&columns, &rows, &["use"], |row, key| match key {
            "code" => !row.0.is_empty(),
            "flags" => !row.1.is_empty(),
            _ => false,
        });
        let keys: Vec<&str> = shown.iter().map(|column| column.key.as_ref()).collect();
        assert_eq!(keys, ["code", "use"]);
    }

    use leptos_router::params::ParamsMap;

    use super::{
        DEFAULT_SIZE, Direction, MAX_SIZE, Noun, OpenWindow, Paging, Sizes, Sort, Window, href,
        partial_note,
    };
    use crate::i18n::Message;

    fluent_templates::static_loader! {
        static CONSOLE = {
            locales: "./tests/it/console",
            fallback_language: "en",
            customise: |bundle| bundle.set_use_isolating(false),
        };
    }

    /// The noun `id` of the test's catalog.
    fn noun(id: &'static str) -> Noun {
        Noun::new(Message::new(&*CONSOLE, id))
    }

    #[test]
    fn the_range_names_its_noun_and_a_partial_answer_says_so() {
        let sets = noun("value-sets");
        let window = Window::over(312, Paging { page: 2, size: 25 });
        assert_eq!("26-50 of 312 value sets", window.range_of(sets));
        assert_eq!(
            "1-1 of 1 value set",
            Window::over(1, Paging::default()).range_of(sets)
        );
        assert_eq!(
            "No value sets",
            Window::over(0, Paging::default()).range_of(sets)
        );
        assert_eq!(None, partial_note(312, Some(312), sets));
        assert_eq!(None, partial_note(312, None, sets));
        assert_eq!(
            Some(String::from(
                "The server counted 340 value sets, and this listing holds 312."
            )),
            partial_note(312, Some(340), sets)
        );
    }

    #[test]
    fn a_console_chooses_its_sizes_and_its_bound() {
        let sizes = Sizes {
            choices: &[],
            default: 100,
            max: 1000,
        };
        assert_eq!(100, Paging::read_within(None, None, sizes).size);
        assert_eq!(1000, Paging::read_within(None, Some("5000"), sizes).size);
        let window = Window::within(
            4000,
            Paging {
                page: 1,
                size: 1000,
            },
            1000,
        );
        assert_eq!((0, 1000, 4), (window.start, window.end, window.pages));
        let mut query = ParamsMap::new();
        Paging { page: 1, size: 100 }.write_within(&mut query, sizes);
        assert_eq!("/l", href("/l", &query), "the default is its absence");
    }

    #[test]
    fn an_open_window_knows_a_previous_page_and_a_full_one() {
        let rows = Noun::ROWS;
        let first = OpenWindow::of(Paging { page: 1, size: 25 }, 25);
        assert!(!first.has_previous() && first.has_next());
        assert_eq!("Rows 1 to 25", first.range_of(rows));
        let second = OpenWindow::of(Paging { page: 2, size: 25 }, 12);
        assert_eq!(25, second.start, "the offset the console asks for");
        assert!(
            second.has_previous() && !second.has_next(),
            "a short page ends it"
        );
        assert_eq!("Rows 26 to 37", second.range_of(rows));
        let past = OpenWindow::of(Paging { page: 9, size: 25 }, 0);
        assert_eq!("No rows", past.range_of(rows));
        assert_eq!("EHRs", noun("ehrs").opening());
        assert_eq!("EHRs 1 to 25", first.range_of(noun("ehrs")));
        assert_eq!("No EHRs", past.range_of(noun("ehrs")));
    }

    #[test]
    fn a_request_is_read_defensively_and_clamped() {
        assert_eq!(Paging::default(), Paging::read(None, None));
        assert_eq!(
            Paging { page: 3, size: 50 },
            Paging::read(Some("3"), Some("50"))
        );
        assert_eq!(1, Paging::read(Some("0"), None).page, "the first page is 1");
        assert_eq!(1, Paging::read(Some("-2"), None).page, "no negative page");
        assert_eq!(1, Paging::read(Some("two"), None).page, "no word");
        assert_eq!(MAX_SIZE, Paging::read(None, Some("100000")).size);
        assert_eq!(1, Paging::read(None, Some("0")).size, "no empty page");
        assert_eq!(DEFAULT_SIZE, Paging::read(None, Some("lots")).size);
    }

    #[test]
    fn a_window_clamps_its_page_and_never_shows_nothing() {
        let window = Window::over(312, Paging { page: 2, size: 25 });
        assert_eq!((25, 50, 13), (window.start, window.end, window.pages));
        assert_eq!("26-50 of 312", window.range_label());
        let last = Window::over(312, Paging { page: 99, size: 25 });
        assert_eq!((13, 300, 312), (last.page, last.start, last.end));
        assert!(!last.has_next() && last.has_previous(), "the last page");
        let empty = Window::over(0, Paging::default());
        assert_eq!(
            (1, 1, 0, 0),
            (empty.page, empty.pages, empty.start, empty.end)
        );
        assert_eq!("No rows", empty.range_label());
        let rows: Vec<u32> = (0..60).collect();
        let window = Window::over(60, Paging { page: 3, size: 25 });
        assert_eq!(&rows[50..60], window.slice(&rows));
    }

    #[test]
    fn the_range_label_names_the_rows_the_page_shows() {
        let rows: Vec<u32> = (1..=137).collect();
        let full = Window::over(137, Paging { page: 1, size: 25 });
        assert_eq!(25, full.slice(&rows).len());
        assert_eq!("1-25 of 137", full.range_label());
        let last = Window::over(137, Paging { page: 6, size: 25 });
        assert_eq!(&rows[125..], last.slice(&rows));
        assert_eq!(12, last.slice(&rows).len());
        assert_eq!("126-137 of 137", last.range_label());
    }

    #[test]
    fn a_default_is_written_as_its_absence_and_other_keys_are_kept() {
        let mut query = ParamsMap::new();
        query.insert("filter", String::from("a b&c"));
        Paging::default().write(&mut query);
        assert_eq!("/list?filter=a%20b%26c", href("/list", &query));
        Paging { page: 2, size: 50 }.write(&mut query);
        let link = href("/list", &query);
        assert!(
            link.contains("page=2") && link.contains("size=50"),
            "{link}"
        );
        assert!(link.contains("filter=a%20b%26c"), "{link}");
    }

    #[test]
    fn the_fields_of_a_query_keep_its_keys_values_and_order() {
        let mut query = ParamsMap::new();
        query.insert("q", String::from("a b&c"));
        query.insert("size", String::from("50"));
        assert_eq!(
            vec![
                (String::from("q"), String::from("a b&c")),
                (String::from("size"), String::from("50")),
            ],
            super::fields(&query)
        );
    }

    #[test]
    fn a_sort_names_a_known_column_and_toggles() {
        let columns = ["name", "version"];
        assert_eq!(None, Sort::read(Some("secret"), None, &columns));
        let sort = Sort::read(Some("name"), Some("desc"), &columns).expect("a sort");
        assert_eq!(Direction::Descending, sort.direction);
        assert_eq!("descending", Sort::aria(Some(&sort), "name"));
        assert_eq!("none", Sort::aria(Some(&sort), "version"));
        assert_eq!(
            Direction::Ascending,
            Sort::toggled(Some(&sort), "name").direction
        );
        let ascending = Sort::toggled(None, "name");
        assert_eq!(
            Direction::Descending,
            Sort::toggled(Some(&ascending), "name").direction
        );
        let mut query = ParamsMap::new();
        query.insert("page", String::from("4"));
        sort.write(&mut query);
        assert_eq!("/l?order=desc&sort=name", href("/l", &query));
    }
}
