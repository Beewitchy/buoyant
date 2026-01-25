mod role;

pub use role::{Role, RoleSet};

/// The direction to search when acquiring focus
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FocusDirection {
    /// Search forward (towards the end)
    #[default]
    Forward,
    /// Search backward (towards the beginning)
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusAction {
    /// Move focus to the next element
    Next,
    /// Move focus to the previous element
    Previous,
    /// Obtain the selected container
    ///
    /// If the current container does not match the requested role mask,
    /// the direction will be used to obtain the nearest matching element.
    Focus(FocusDirection),
    /// Exit the selected container.
    ///
    /// Typically associated with the user pressing a "back" or "menu" button.
    Blur,
    /// Perform the focused element's primary action. If the currently focused element
    /// does not match the requested role set, this does nothing.
    Select,
}

/// A trait for focus tree types that can be initialized to either the first or last element.
///
/// This is roughly equivalent to `Default` but supports bidirectional navigation
pub trait DefaultFocus {
    /// Returns a focus tree initialized to the first element.
    fn default_first() -> Self;

    /// Returns a focus tree initialized to the last element.
    fn default_last() -> Self;
}

impl DefaultFocus for () {
    fn default_first() -> Self {}
    fn default_last() -> Self {}
}

/// A group identifying a set of related elements.
#[derive(Clone, Copy, Debug, Eq)]
pub struct FocusGroup(u8);

impl FocusGroup {
    /// Creates a focus group with the specified group index (0-7)
    #[must_use]
    pub const fn new(i: u8) -> Option<Self> {
        if i < 8 { Some(Self(0b1 << i)) } else { None }
    }

    /// Creates a focus group with the specified group index (0-7)
    #[must_use]
    pub const fn new_unchecked(i: u8) -> Self {
        Self(0b1 << i)
    }

    /// The common group, matching all other groups
    #[must_use]
    pub const fn common_group() -> Self {
        Self(0b1111_1111)
    }

    /// Returns true if this is the common group.
    #[must_use]
    pub const fn is_common_group(self) -> bool {
        self.0 == Self::common_group().0
    }

    /// Returns the underlying group index (0-7) for this focus group, or None if
    /// this is the common group.
    #[must_use]
    pub const fn index(self) -> Option<u8> {
        if self.is_common_group() {
            None
        } else {
            Some(self.0.trailing_zeros() as u8)
        }
    }
}

impl Default for FocusGroup {
    fn default() -> Self {
        Self::common_group()
    }
}

impl PartialEq for FocusGroup {
    fn eq(&self, other: &Self) -> bool {
        (self.0 & other.0) != 0
    }
}

pub static GROUP_0: FocusGroup = FocusGroup(0b1 << 0);
pub static GROUP_1: FocusGroup = FocusGroup(0b1 << 1);
pub static GROUP_2: FocusGroup = FocusGroup(0b1 << 2);
pub static GROUP_3: FocusGroup = FocusGroup(0b1 << 3);
pub static GROUP_4: FocusGroup = FocusGroup(0b1 << 4);
pub static GROUP_5: FocusGroup = FocusGroup(0b1 << 5);
pub static GROUP_6: FocusGroup = FocusGroup(0b1 << 6);
pub static GROUP_7: FocusGroup = FocusGroup(0b1 << 7);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_indecies() {
        for i in 0..8 {
            let group = FocusGroup::new(i).unwrap();
            let unchecked_group = FocusGroup::new_unchecked(i);
            assert_eq!(group, unchecked_group);
            assert_eq!(group.index(), Some(i));
        }
    }

    #[test]
    fn common_group_equals_all_groups() {
        assert_eq!(FocusGroup::common_group(), FocusGroup::common_group());

        for i in 0..8 {
            let group = FocusGroup::new(i).unwrap();
            assert_eq!(group, FocusGroup::common_group());
        }
    }
}
