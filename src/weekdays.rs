use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Index, Not},
};

use crate::Weekday;

/// A collection of `Weekday`s stored as a single byte.
///
/// This type is `Copy` and provides efficient set-like and slice-like operations.
/// Many operations are `const` as well.
///
/// Implemented as a bitmask where bits 1-7 correspond to Monday-Sunday.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeekdaySet(u8); // Invariant: the 8-th bit is always 0.

impl WeekdaySet {
    /// Create a `WeekdaySet` from a bitmask.
    ///
    /// If present, the 8-th bit is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::EMPTY, WeekdaySet::from_bits_truncate(0));
    /// assert_eq!(WeekdaySet::single(Mon), WeekdaySet::from_bits_truncate(0b1));
    /// assert_eq!(WeekdaySet::single(Tue), WeekdaySet::from_bits_truncate(0b10));
    /// assert_eq!(WeekdaySet::from_array([Mon, Wed]), WeekdaySet::from_bits_truncate(0b101));
    /// assert_eq!(WeekdaySet::ALL, WeekdaySet::from_bits_truncate(0b111_1111));
    /// assert_eq!(WeekdaySet::single(Mon), WeekdaySet::from_bits_truncate(0b1000_0001));
    /// ```
    #[must_use]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & 0b111_1111)
    }

    /// Returns `Some(day)` if this collection contains exactly one day.
    ///
    /// Returns `None` otherwise.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).single_day(), Some(Mon));
    /// assert_eq!(WeekdaySet::from_array([Mon, Tue]).single_day(), None);
    /// assert_eq!(WeekdaySet::EMPTY.single_day(), None);
    /// assert_eq!(WeekdaySet::ALL.single_day(), None);
    /// ```
    pub const fn single_day(self) -> Option<Weekday> {
        match self {
            Self::MON => Some(Weekday::Mon),
            Self::TUE => Some(Weekday::Tue),
            Self::WED => Some(Weekday::Wed),
            Self::THU => Some(Weekday::Thu),
            Self::FRI => Some(Weekday::Fri),
            Self::SAT => Some(Weekday::Sat),
            Self::SUN => Some(Weekday::Sun),
            _ => None,
        }
    }

    /// Returns `true` if `other` contains all days in `self`.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert!(WeekdaySet::single(Mon).is_subset(WeekdaySet::ALL));
    /// assert!(!WeekdaySet::single(Mon).is_subset(WeekdaySet::EMPTY));
    /// assert!(WeekdaySet::EMPTY.is_subset(WeekdaySet::single(Mon)));
    /// ```
    pub const fn is_subset(self, other: Self) -> bool {
        self.intersection(other).0 == self.0
    }

    /// Returns `true` if `self` contains all days in `other`.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert!(WeekdaySet::ALL.is_superset(WeekdaySet::single(Mon)));
    /// assert!(WeekdaySet::single(Mon).is_superset(WeekdaySet::EMPTY));
    /// assert!(!WeekdaySet::single(Mon).is_superset(WeekdaySet::single(Tue)));
    /// ```
    pub const fn is_superset(self, other: Self) -> bool {
        self.intersection(other).0 == other.0
    }

    /// Adds a day to the collection.
    ///
    /// Returns `true` if the day was new to the collection.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// let mut weekdays = WeekdaySet::single(Mon);
    /// assert!(weekdays.insert(Tue));
    /// assert!(!weekdays.insert(Tue));
    /// ```
    pub fn insert(&mut self, day: Weekday) -> bool {
        if self.contains(day) {
            return false;
        }

        self.0 |= Self::single(day).0;
        true
    }

    /// Removes a day from the collection.
    ///
    /// Returns `true` if the collection did contain the day.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// let mut weekdays = WeekdaySet::single(Mon);
    /// assert!(weekdays.remove(Mon));
    /// assert!(!weekdays.remove(Mon));
    /// ```
    pub fn remove(&mut self, day: Weekday) -> bool {
        if self.contains(day) {
            self.0 &= !Self::single(day).0;
            return true;
        }

        false
    }

    /// Convert the collection into a `Vec<Weekday>`.
    #[cfg(feature = "std")]
    pub fn to_vec(self) -> Vec<Weekday> {
        self.iter().collect()
    }

    /// Iterate over the `Weekday`s in the collection.
    ///
    /// Starting from Monday, in ascending order.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// let weekdays = WeekdaySet::from_iter([Mon, Wed, Fri]);
    /// let mut iter = weekdays.iter();
    /// assert_eq!(iter.next(), Some(Mon));
    /// assert_eq!(iter.next(), Some(Wed));
    /// assert_eq!(iter.next(), Some(Fri));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub const fn iter(self) -> WeekdaySetIter {
        WeekdaySetIter(self)
    }

    /// Get the first day in the collection, starting from Monday.
    ///
    /// Returns `None` if the collection is empty.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).first(), Some(Mon));
    /// assert_eq!(WeekdaySet::single(Tue).first(), Some(Tue));
    /// assert_eq!(WeekdaySet::ALL.first(), Some(Mon));
    /// assert_eq!(WeekdaySet::EMPTY.first(), None);
    /// ```
    pub const fn first(self) -> Option<Weekday> {
        if self.is_empty() {
            return None;
        }

        // Find the first non-zero bit.
        let bit = 1 << self.0.trailing_zeros();

        Self(bit).single_day()
    }

    /// Get the last day in the collection, starting from Sunday.
    ///
    /// Returns `None` if the collection is empty.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).last(), Some(Mon));
    /// assert_eq!(WeekdaySet::single(Sun).last(), Some(Sun));
    /// assert_eq!(WeekdaySet::from_array([Mon, Tue]).last(), Some(Tue));
    /// assert_eq!(WeekdaySet::EMPTY.last(), None);
    /// ```
    pub fn last(self) -> Option<Weekday> {
        if self.is_empty() {
            return None;
        }

        // Find the last non-zero bit.
        let bit = 1 << (7 - self.0.leading_zeros());

        Self(bit).single_day()
    }

    /// Split the collection in two at the given day.
    ///
    /// Returns a tuple `(before, after)`. `before` contains all days starting from Monday
    /// up to but __not__ including `weekday`. `after` contains all days starting from `weekday`
    /// up to and including Sunday.
    ///
    /// # Example
    /// ```ignore
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// let (before, after) = WeekdaySet::ALL.split_at(Fri);
    /// assert_eq!(before, WeekdaySet::from_array([Mon, Tue, Wed, Thu]));
    /// assert_eq!(after, WeekdaySet::from_array([Fri, Sat, Sun]));
    /// ```
    const fn split_at(self, weekday: Weekday) -> (Self, Self) {
        let days_after = 0b1000_0000 - Self::single(weekday).0;
        let days_before = days_after ^ 0b0111_1111;
        (Self(self.0 & days_before), Self(self.0 & days_after))
    }

    /// Iterate over the `Weekday`s in the collection, starting from a given day
    /// and wrapping around from Sunday to Monday.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// let weekdays = WeekdaySet::from_array([Mon, Wed, Fri]);
    /// let mut iter = weekdays.iter_from(Wed);
    /// assert_eq!(iter.next(), Some(Wed));
    /// assert_eq!(iter.next(), Some(Fri));
    /// assert_eq!(iter.next(), Some(Mon));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub const fn iter_from(self, start: Weekday) -> WeekdaySetIterFrom {
        WeekdaySetIterFrom { days: self, start }
    }

    /// Returns the collection with all days inverted.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).inverse(), WeekdaySet::from_array([Tue, Wed, Thu, Fri, Sat, Sun]));
    /// assert_eq!(WeekdaySet::ALL.inverse(), WeekdaySet::EMPTY);
    /// assert_eq!(WeekdaySet::EMPTY.inverse(), WeekdaySet::ALL);
    /// ```
    pub const fn inverse(self) -> Self {
        Self(self.0 ^ 0b0111_1111)
    }

    /// Returns days that are in both `self` and `other`.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).intersection(WeekdaySet::single(Mon)), WeekdaySet::single(Mon));
    /// assert_eq!(WeekdaySet::single(Mon).intersection(WeekdaySet::single(Tue)), WeekdaySet::EMPTY);
    /// assert_eq!(WeekdaySet::ALL.intersection(WeekdaySet::single(Mon)), WeekdaySet::single(Mon));
    /// assert_eq!(WeekdaySet::ALL.intersection(WeekdaySet::EMPTY), WeekdaySet::EMPTY);
    /// ```
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Returns days that are in either `self` or `other`.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).union(WeekdaySet::single(Mon)), WeekdaySet::single(Mon));
    /// assert_eq!(WeekdaySet::single(Mon).union(WeekdaySet::single(Tue)), WeekdaySet::from_array([Mon, Tue]));
    /// assert_eq!(WeekdaySet::ALL.union(WeekdaySet::single(Mon)), WeekdaySet::ALL);
    /// assert_eq!(WeekdaySet::ALL.union(WeekdaySet::EMPTY), WeekdaySet::ALL);
    /// ```
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns days that are in `self` or `other` but not in both.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).symmetric_difference(WeekdaySet::single(Mon)), WeekdaySet::EMPTY);
    /// assert_eq!(WeekdaySet::single(Mon).symmetric_difference(WeekdaySet::single(Tue)), WeekdaySet::from_array([Mon, Tue]));
    /// assert_eq!(
    ///     WeekdaySet::ALL.symmetric_difference(WeekdaySet::single(Mon)),
    ///     WeekdaySet::from_array([Tue, Wed, Thu, Fri, Sat, Sun]),
    /// );
    /// assert_eq!(WeekdaySet::ALL.symmetric_difference(WeekdaySet::EMPTY), WeekdaySet::ALL);
    /// ```
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    /// Returns days that are in `self` but not in `other`.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).difference(WeekdaySet::single(Mon)), WeekdaySet::EMPTY);
    /// assert_eq!(WeekdaySet::single(Mon).difference(WeekdaySet::single(Tue)), WeekdaySet::single(Mon));
    /// assert_eq!(WeekdaySet::EMPTY.difference(WeekdaySet::single(Mon)), WeekdaySet::EMPTY);
    /// ```
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Returns `true` if the collection contains the given day.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert!(WeekdaySet::single(Mon).contains(Mon));
    /// assert!(WeekdaySet::from_array([Mon, Tue]).contains(Tue));
    /// assert!(!WeekdaySet::single(Mon).contains(Tue));
    /// ```
    pub const fn contains(self, day: Weekday) -> bool {
        self.0 & Self::single(day).0 != 0
    }

    /// Returns `true` if the collection is empty.
    ///
    /// # Example
    /// ```
    /// # use chrono::{Weekday, WeekdaySet};
    /// assert!(WeekdaySet::EMPTY.is_empty());
    /// assert!(!WeekdaySet::single(Weekday::Mon).is_empty());
    /// ```
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }
    /// Returns the number of days in the collection.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::single(Mon).len(), 1);
    /// assert_eq!(WeekdaySet::from_array([Mon, Wed, Fri]).len(), 3);
    /// assert_eq!(WeekdaySet::ALL.len(), 7);
    /// ```
    pub const fn len(self) -> u8 {
        self.0.count_ones() as u8
    }

    /// Create a `WeekdaySet` from a single `Weekday`.
    ///
    /// # Example
    /// ```
    /// # use chrono::{Weekday, WeekdaySet};
    /// assert_eq!(WeekdaySet::MON, WeekdaySet::single(Weekday::Mon));
    /// ```
    pub const fn single(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Mon => Self::MON,
            Weekday::Tue => Self::TUE,
            Weekday::Wed => Self::WED,
            Weekday::Thu => Self::THU,
            Weekday::Fri => Self::FRI,
            Weekday::Sat => Self::SAT,
            Weekday::Sun => Self::SUN,
        }
    }

    /// Create a `WeekdaySet` from an array of `Weekday`s.
    ///
    /// # Example
    /// ```
    /// # use chrono::WeekdaySet;
    /// use chrono::Weekday::*;
    /// assert_eq!(WeekdaySet::EMPTY, WeekdaySet::from_array([]));
    /// assert_eq!(WeekdaySet::single(Mon), WeekdaySet::from_array([Mon]));
    /// assert_eq!(WeekdaySet::ALL, WeekdaySet::from_array([Mon, Tue, Wed, Thu, Fri, Sat, Sun]));
    /// ```
    pub const fn from_array<const C: usize>(days: [Weekday; C]) -> Self {
        let mut acc = Self::EMPTY;
        let mut idx = 0;
        while idx < days.len() {
            acc.0 |= Self::single(days[idx]).0;
            idx += 1;
        }
        acc
    }

    /// An empty `WeekdaySet`.
    pub const EMPTY: Self = Self(0b000_0000);
    /// A `WeekdaySet` containing all seven `Weekday`s.
    pub const ALL: Self = Self(0b111_1111);

    /// A `WeekdaySet` containing only Monday.
    pub const MON: Self = Self(0b000_0001);
    /// A `WeekdaySet` containing only Tuesday.
    pub const TUE: Self = Self(0b000_0010);
    /// A `WeekdaySet` containing only Wednesday.
    pub const WED: Self = Self(0b000_0100);
    /// A `WeekdaySet` containing only Thursday.
    pub const THU: Self = Self(0b000_1000);
    /// A `WeekdaySet` containing only Friday.
    pub const FRI: Self = Self(0b001_0000);
    /// A `WeekdaySet` containing only Saturday.
    pub const SAT: Self = Self(0b010_0000);
    /// A `WeekdaySet` containing only Sunday.
    pub const SUN: Self = Self(0b100_0000);
}

/// Print the underlying bitmask, padded to 7 bits.
///
/// # Example
/// ```
/// # use chrono::WeekdaySet;
/// use chrono::Weekday::*;
/// assert_eq!(format!("{:?}", WeekdaySet::single(Mon)), "WeekdaySet(0000001)");
/// assert_eq!(format!("{:?}", WeekdaySet::single(Tue)), "WeekdaySet(0000010)");
/// assert_eq!(format!("{:?}", WeekdaySet::ALL), "WeekdaySet(1111111)");
/// ```
impl Debug for WeekdaySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WeekdaySet({:0>7b})", self.0)
    }
}

/// An iterator over a collection of weekdays.
///
/// See `WeekdaySet::iter`.
#[derive(Debug, Clone)]
pub struct WeekdaySetIter(pub WeekdaySet);

impl Iterator for WeekdaySetIter {
    type Item = Weekday;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        let next = self.0.first().expect("the collection is not empty");
        self.0.remove(next);
        Some(next)
    }
}

impl DoubleEndedIterator for WeekdaySetIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        let next_back = self.0.last().expect("the collection is not empty");
        self.0.remove(next_back);
        Some(next_back)
    }
}

impl ExactSizeIterator for WeekdaySetIter {
    fn len(&self) -> usize {
        self.0.len().into()
    }
}

impl FusedIterator for WeekdaySetIter {}

/// An iterator over a collection of weekdays, starting from a given day.
///
/// See `WeekdaySet::iter_from`.
#[derive(Debug, Clone)]
pub struct WeekdaySetIterFrom {
    pub days: WeekdaySet,
    pub start: Weekday,
}

impl Iterator for WeekdaySetIterFrom {
    type Item = Weekday;

    fn next(&mut self) -> Option<Self::Item> {
        if self.days.is_empty() {
            return None;
        }

        // Split the collection in two at `start`.
        // Look for the first day among the days after `start` first, including `start` itself.
        // If there are no days after `start`, look for the first day among the days before `start`.
        let (before, after) = self.days.split_at(self.start);
        let days = if after.is_empty() { before } else { after };

        let next = days.first().expect("the collection is not empty");
        self.days.remove(next);
        Some(next)
    }
}

impl DoubleEndedIterator for WeekdaySetIterFrom {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.days.is_empty() {
            return None;
        }

        // Split the collection in two at `start`.
        // Look for the last day among the days before `start` first, NOT including `start` itself.
        // If there are no days before `start`, look for the last day among the days after `start`.
        let (before, after) = self.days.split_at(self.start);
        let days = if before.is_empty() { after } else { before };

        let next_back = days.last().expect("the collection is not empty");
        self.days.remove(next_back);
        Some(next_back)
    }
}

impl ExactSizeIterator for WeekdaySetIterFrom {
    fn len(&self) -> usize {
        self.days.len().into()
    }
}

impl FusedIterator for WeekdaySetIterFrom {}

/// Print the collection as a slice-like list of weekdays.
///
/// # Example
/// ```
/// # use chrono::WeekdaySet;
/// use chrono::Weekday::*;
/// assert_eq!("[]", WeekdaySet::EMPTY.to_string());
/// assert_eq!("[Mon]", WeekdaySet::single(Mon).to_string());
/// assert_eq!("[Mon, Fri, Sun]", WeekdaySet::from_array([Mon, Fri, Sun]).to_string());
/// ```
impl fmt::Display for WeekdaySet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        let mut iter = self.iter();
        if let Some(first) = iter.next() {
            write!(f, "{first}")?;
        }
        for weekday in iter {
            write!(f, ", {weekday}")?;
        }
        write!(f, "]")
    }
}

// impl Bit* for WeekdaySet
impl BitOr for WeekdaySet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitAnd for WeekdaySet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl BitXor for WeekdaySet {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.symmetric_difference(rhs)
    }
}

// impl Bit*Assign for WeekdaySet
impl BitOrAssign for WeekdaySet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for WeekdaySet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXorAssign for WeekdaySet {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for WeekdaySet {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.inverse()
    }
}

impl From<Weekday> for WeekdaySet {
    fn from(weekday: Weekday) -> Self {
        Self::single(weekday)
    }
}

impl Extend<Weekday> for WeekdaySet {
    fn extend<T: IntoIterator<Item = Weekday>>(&mut self, iter: T) {
        for weekday in iter {
            self.insert(weekday);
        }
    }
}

impl FromIterator<Weekday> for WeekdaySet {
    fn from_iter<T: IntoIterator<Item = Weekday>>(iter: T) -> Self {
        let mut weekdays = Self::EMPTY;
        weekdays.extend(iter);
        weekdays
    }
}

impl IntoIterator for WeekdaySet {
    type Item = Weekday;
    type IntoIter = WeekdaySetIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// impl Bit*<Weekday> for WeekdaySet
impl BitOr<Weekday> for WeekdaySet {
    type Output = Self;

    fn bitor(self, rhs: Weekday) -> Self::Output {
        self | Self::from(rhs)
    }
}

impl BitAnd<Weekday> for WeekdaySet {
    type Output = Self;

    fn bitand(self, rhs: Weekday) -> Self::Output {
        self & Self::from(rhs)
    }
}

impl BitXor<Weekday> for WeekdaySet {
    type Output = Self;

    fn bitxor(self, rhs: Weekday) -> Self::Output {
        self ^ Self::from(rhs)
    }
}

/// Can be used to check the presence of a day in the collection.
///
/// # Example
/// ```
/// # use chrono::WeekdaySet;
/// use chrono::Weekday::*;
/// assert!(WeekdaySet::single(Mon)[Mon]);
/// assert!(WeekdaySet::ALL[Mon]);
/// assert!(!WeekdaySet::EMPTY[Mon]);
/// assert!(!WeekdaySet::single(Tue)[Mon]);
/// ```
impl Index<Weekday> for WeekdaySet {
    type Output = bool;

    fn index(&self, weekday: Weekday) -> &Self::Output {
        if self.contains(weekday) { &true } else { &false }
    }
}

// impl Bit*<WeekdaySet> for Weekday
impl BitOr<WeekdaySet> for Weekday {
    type Output = WeekdaySet;

    fn bitor(self, rhs: WeekdaySet) -> Self::Output {
        WeekdaySet::from(self) | rhs
    }
}

impl BitAnd<WeekdaySet> for Weekday {
    type Output = WeekdaySet;

    fn bitand(self, rhs: WeekdaySet) -> Self::Output {
        WeekdaySet::from(self) & rhs
    }
}

impl BitXor<WeekdaySet> for Weekday {
    type Output = WeekdaySet;

    fn bitxor(self, rhs: WeekdaySet) -> Self::Output {
        WeekdaySet::from(self) ^ rhs
    }
}

// impl Bit*Assign<Weekday> for WeekdaySet
impl BitOrAssign<Weekday> for WeekdaySet {
    fn bitor_assign(&mut self, rhs: Weekday) {
        *self |= Self::from(rhs);
    }
}

impl BitAndAssign<Weekday> for WeekdaySet {
    fn bitand_assign(&mut self, rhs: Weekday) {
        *self &= Self::from(rhs);
    }
}

impl BitXorAssign<Weekday> for WeekdaySet {
    fn bitxor_assign(&mut self, rhs: Weekday) {
        *self ^= Self::from(rhs);
    }
}

// impl Bit* for Weekday
impl BitOr for Weekday {
    type Output = WeekdaySet;

    fn bitor(self, rhs: Self) -> Self::Output {
        WeekdaySet::from(self) | WeekdaySet::from(rhs)
    }
}

impl BitAnd for Weekday {
    type Output = WeekdaySet;

    fn bitand(self, rhs: Self) -> Self::Output {
        WeekdaySet::from(self) & WeekdaySet::from(rhs)
    }
}

impl BitXor for Weekday {
    type Output = WeekdaySet;

    fn bitxor(self, rhs: Self) -> Self::Output {
        WeekdaySet::from(self) ^ WeekdaySet::from(rhs)
    }
}

impl Not for Weekday {
    type Output = WeekdaySet;

    fn not(self) -> Self::Output {
        !WeekdaySet::from(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::Weekday;

    use super::WeekdaySet;

    impl WeekdaySet {
        /// Iterate over all 128 possible sets, from `EMPTY` to `ALL`.
        fn iter_all() -> impl Iterator<Item = Self> {
            (0b0000_0000..0b1000_0000).map(Self)
        }
    }

    /// Panics if the 8-th bit of `self` is not 0.
    fn assert_8th_bit_invariant(days: WeekdaySet) {
        assert!(days.0 & 0b1000_0000 == 0, "the 8-th bit of {days:?} is not 0");
    }

    #[test]
    fn debug_prints_8th_bit_if_not_zero() {
        assert_eq!(format!("{:?}", WeekdaySet(0b1000_0000)), "WeekdaySet(10000000)");
    }

    #[test]
    fn bitwise_set_operations_preserve_8th_bit_invariant() {
        for set1 in WeekdaySet::iter_all() {
            for set2 in WeekdaySet::iter_all() {
                assert_8th_bit_invariant(set1.union(set2));
                assert_8th_bit_invariant(set1.intersection(set2));
                assert_8th_bit_invariant(set1.symmetric_difference(set2));
            }
        }
    }

    #[test]
    fn not_operation_preserves_8th_bit_invariant() {
        for days in WeekdaySet::iter_all() {
            assert_8th_bit_invariant(!days);
        }
    }

    /// Test `split_at` on all possible arguments.
    #[test]
    fn split_at_is_equivalent_to_iterating() {
        use Weekday::*;

        // `split_at` is used in `iter_from`, so we must not iterate
        // over all days with `WeekdaySet::ALL.iter_from(Mon)`.
        const WEEK: [Weekday; 7] = [Mon, Tue, Wed, Thu, Fri, Sat, Sun];

        for weekdays in WeekdaySet::iter_all() {
            for split_day in WEEK {
                let expected_before: WeekdaySet = WEEK
                    .into_iter()
                    .take_while(|&day| day != split_day)
                    .filter(|&day| weekdays.contains(day))
                    .collect();
                let expected_after: WeekdaySet = WEEK
                    .into_iter()
                    .skip_while(|&day| day != split_day)
                    .filter(|&day| weekdays.contains(day))
                    .collect();

                assert_eq!(
                    (expected_before, expected_after),
                    weekdays.split_at(split_day),
                    "split_at({split_day}) failed for {weekdays}",
                );
            }
        }
    }
}
