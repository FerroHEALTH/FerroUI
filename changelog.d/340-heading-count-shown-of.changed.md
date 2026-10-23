- A list card's and a panel section's count can say the items shown of a
  larger set: `ListCard::count` and `SectionHead::count` take an
  `Option<view::template::list::HeadingCount>`, drawn as "· 15"
  (`HeadingCount::All`) or "· 6 of 15" (`HeadingCount::Shown`), each number
  with the locale's marks, and `template::list::counted` takes a
  `HeadingCount`. Breaking: `count: Some(15)` becomes `count:
  Some(HeadingCount::All(15))` (or `Some(15.into())` from a `u64`), and a
  console that translates the library's catalog adds `count-shown-of`.
