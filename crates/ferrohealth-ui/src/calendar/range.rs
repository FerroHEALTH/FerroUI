// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The bounds a picker allows, the quick choices and the range presets.

use super::{Bounds, Day, Month, Preset, Quick};

impl Bounds {
    /// Whether `day` is allowed.
    #[must_use]
    pub fn allows(self, day: Day) -> bool {
        self.earliest.is_none_or(|earliest| day >= earliest)
            && self.latest.is_none_or(|latest| day <= latest)
    }

    /// Whether some day of `month` is allowed.
    #[must_use]
    pub fn reaches(self, month: Month) -> bool {
        self.earliest
            .is_none_or(|earliest| month.last() >= earliest)
            && self.latest.is_none_or(|latest| month.first() <= latest)
    }
}

impl Quick {
    /// The day the choice means when today is `today`.
    #[must_use]
    pub fn day(self, today: Day) -> Option<Day> {
        match self {
            Self::Days(count) => today.step(i32::from(count)),
            Self::Next(weekday) => {
                let ahead = (7 + weekday.index() - today.weekday().index()) % 7;
                today.step(i32::from(if ahead == 0 { 7 } else { ahead }))
            }
        }
    }
}

impl Preset {
    /// The first and last day of the preset when today is `today`.
    #[must_use]
    pub fn range(self, today: Day) -> Option<(Day, Day)> {
        match self {
            Self::Today => Some((today, today)),
            Self::LastDays(count) => {
                let back = i32::from(count.max(1)) - 1;
                Some((today.step(-back)?, today))
            }
            Self::ThisMonth => Some((today.month_of().first(), today)),
        }
    }
}
