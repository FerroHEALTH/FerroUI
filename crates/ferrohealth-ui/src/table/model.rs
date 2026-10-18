// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The paging and sorting arithmetic. Every number a request carries is user
//! input, so each is parsed defensively and clamped, and every sum
//! saturates. No specification governs paging: our own design.

use leptos_router::params::ParamsMap;

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

/// Where a listing keeps its page, size and sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StateSource {
    /// In the URL's query: shareable, walked by back and forward, working
    /// before the page hydrates.
    #[default]
    Url,
    /// In the page's own state, never in a URL or the history: for a screen
    /// that can show patient data.
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
        // NOTE: no specification governs this: our own design; a page or size that
        // is no number is the value not given, never an error page.
        let page = page
            .and_then(|page| page.trim().parse::<u32>().ok())
            .unwrap_or(1)
            .max(1);
        let size = size
            .and_then(|size| size.trim().parse::<u32>().ok())
            .unwrap_or(DEFAULT_SIZE)
            .clamp(1, MAX_SIZE);
        Self { page, size }
    }

    /// The paging the query `query` names.
    #[must_use]
    pub fn from_query(query: &ParamsMap) -> Self {
        Self::read(query.get_str(PAGE_KEY), query.get_str(SIZE_KEY))
    }

    /// This paging written into `query`: the first page and the default size
    /// are written as their absence, so the first page's URL is the plain
    /// path.
    pub fn write(self, query: &mut ParamsMap) {
        if self.page <= 1 {
            drop(query.remove(PAGE_KEY));
        } else {
            query.replace(PAGE_KEY, self.page.to_string());
        }
        if self.size == DEFAULT_SIZE {
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
        let size = paging.size.clamp(1, MAX_SIZE);
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
            return String::from("No rows");
        }
        format!(
            "{}-{} of {}",
            self.start.saturating_add(1),
            self.end,
            self.total
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

#[cfg(test)]
mod tests {
    use leptos_router::params::ParamsMap;

    use super::{DEFAULT_SIZE, Direction, MAX_SIZE, Paging, Sort, Window, href};

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
