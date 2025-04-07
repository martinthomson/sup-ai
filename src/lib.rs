use std::{cmp::max, iter::Peekable};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
enum UsagePreferenceState {
    #[default]
    Unknown,
    Yes,
    No,
}

impl UsagePreferenceState {
    /// Produce a merged value from this and another value.
    ///
    /// Logic is: if either is "No", pick "No".
    /// Otherwise, if either is "Yes", pick "Yes".
    /// Finally, if both are "Unknown", pick "Unknown".
    fn merge(&mut self, other: Self) {
        *self = match (*self, other) {
            (Self::No, _) | (_, Self::No) => Self::No,
            (Self::Yes, _) | (_, Self::Yes) => Self::Yes,
            (Self::Unknown, Self::Unknown) => Self::Unknown,
        };
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum UsagePreference {
    Allowed,
    Denied,
}

impl TryFrom<UsagePreferenceState> for UsagePreference {
    type Error = ();
    fn try_from(value: UsagePreferenceState) -> Result<Self, Self::Error> {
        match value {
            UsagePreferenceState::Unknown => Err(()),
            UsagePreferenceState::Yes => Ok(Self::Allowed),
            UsagePreferenceState::No => Ok(Self::Denied),
        }
    }
}

#[derive(Debug)]
struct UsagePreferenceItem {
    name: Vec<u8>,
    parent: Option<usize>,
    value: UsagePreferenceState,
}

#[derive(Debug)]
pub struct UsagePreferences {
    items: Vec<UsagePreferenceItem>,
    max_len: usize,
}

/// A simple wrapper that makes handling input sequences easier.
trait Input {
    fn peek(&mut self) -> Option<u8>;
    fn next(&mut self) -> Option<u8>;
    fn next_if(&mut self, f: impl FnOnce(u8) -> bool) -> Option<u8>;
    fn skip_until(&mut self, f: impl Fn(u8) -> bool);
    fn skip_ws(&mut self) {
        self.skip_until(|c| !c.is_ascii_whitespace());
    }
}
impl<'a, T> Input for Peekable<T>
where
    T: Iterator<Item = &'a u8> + 'a,
{
    fn peek(&mut self) -> Option<u8> {
        Peekable::peek(self).copied().copied()
    }

    fn next(&mut self) -> Option<u8> {
        Iterator::next(self).copied()
    }

    fn next_if(&mut self, f: impl FnOnce(u8) -> bool) -> Option<u8> {
        Peekable::next_if(self, |&&c| f(c)).copied()
    }

    fn skip_until(&mut self, f: impl Fn(u8) -> bool) {
        while Input::next_if(self, |c| !f(c)).is_some() {}
    }
}

impl UsagePreferences {
    /// Text and Data Mining usage.
    ///
    /// This is a very broadly-defined category of usage that covers most automated processing of content.
    pub const TDM: &str = "tdm";
    /// Artificial Intelligence (or Machine Learning).
    ///
    /// This relates to any use of the content for training or operation of machine learning models.
    pub const AI: &str = "ai";
    /// Generative AI.
    ///
    /// Use of the content for training or operation of "foundational" models,
    /// or those that are capable of producing content.
    pub const GENAI: &str = "genai";
    /// Search.
    ///
    /// The use of content for the building of a search index
    /// and ultimately the usage of that index to produce a product that exists
    /// primarily to direct people to the location where the content was obtained.
    pub const SEARCH: &str = "search";

    /// Instantiate a blank usage preferences set with no usages registered.
    ///
    /// Note: Use the `Default` implementation to get the standard set of usages.
    pub fn blank() -> Self {
        Self {
            items: Vec::new(),
            max_len: 0,
        }
    }

    fn add_impl(&mut self, usage: impl AsRef<[u8]>, parent: Option<usize>) {
        let name = usage.as_ref();
        assert!(!name.contains(&b','), "usage name cannot contain a comma");
        assert!(!name.contains(&b'='), "usage name cannot contain equals");
        assert!(
            self.items.iter().find(|it| it.name == name).is_none(),
            "duplicate usage added"
        );
        self.max_len = max(self.max_len, name.len());
        self.items.push(UsagePreferenceItem {
            name: name.to_vec(),
            parent,
            value: UsagePreferenceState::Unknown,
        });
    }

    /// Add a usage that this object will track.
    pub fn add(&mut self, usage: impl AsRef<[u8]>) {
        self.add_impl(usage, None);
    }

    /// Add a usage that this object will track.
    /// Include the identified parent type, which will be used if there is no preference
    /// expressed for this usage.
    pub fn add_child(&mut self, usage: impl AsRef<[u8]>, parent: impl AsRef<[u8]>) {
        let parent = parent.as_ref();
        let p = self
            .items
            .iter()
            .position(|it| it.name == parent)
            .expect("parent not found");
        self.add_impl(usage, Some(p));
    }

    fn parse_name(&self, r: &mut impl Input, max_len: usize) -> Option<usize> {
        r.skip_ws();
        let mut n = Vec::with_capacity(max_len);
        for _ in 0..max_len {
            if let Some(c) = r.next_if(|c| c != b'=' && c != b',') {
                n.push(c);
            } else {
                break;
            }
        }
        r.skip_ws();
        if r.next_if(|c| c == b'=').is_some() {
            let usage = n.trim_ascii_end();
            if let Some(pos) = self.items.iter().position(|it| it.name == usage) {
                return Some(pos);
            }
        }
        None
    }

    fn parse_value(&self, r: &mut impl Input) -> UsagePreferenceState {
        r.skip_ws();
        let v = match r.next() {
            Some(b'y') => UsagePreferenceState::Yes,
            Some(b'n') => UsagePreferenceState::No,
            _ => UsagePreferenceState::Unknown,
        };
        r.skip_ws();
        if matches!(r.peek(), None | Some(b',')) {
            v
        } else {
            UsagePreferenceState::Unknown
        }
    }

    /// Parse the provided input.
    ///
    /// This adds the rules in the provided string to those that this object already holds.
    pub fn parse(&mut self, expr: impl AsRef<[u8]>) {
        let mut r = expr.as_ref().into_iter().peekable();
        while r.peek().is_some() {
            if let Some(i) = self.parse_name(&mut r, self.max_len) {
                let v = self.parse_value(&mut r);
                self.items[i].value.merge(v);
            }
            r.skip_until(|c| c == b',');
            _ = Iterator::next(&mut r); // Discard any ','.
        }
    }

    /// Evaluate the usage preference against the given usage.
    pub fn eval(&self, usage: impl AsRef<[u8]>, dflt: UsagePreference) -> UsagePreference {
        let usage = usage.as_ref();
        let Some(mut i) = self.items.iter().position(|it| it.name == usage) else {
            return dflt;
        };
        loop {
            if let Ok(res) = UsagePreference::try_from(self.items[i].value) {
                return res;
            }
            i = if let Some(p) = self.items[i].parent {
                p
            } else {
                return dflt;
            };
        }
    }
}

impl Default for UsagePreferences {
    fn default() -> Self {
        let mut v = Self {
            items: Vec::with_capacity(4),
            max_len: 0,
        };
        v.add(Self::TDM);
        v.add_child(Self::AI, Self::TDM);
        v.add_child(Self::GENAI, Self::AI);
        v.add_child(Self::SEARCH, Self::TDM);
        v
    }
}

#[cfg(test)]
mod test {
    use crate::{
        UsagePreference::{Allowed, Denied},
        UsagePreferences,
    };

    const TDM: &str = UsagePreferences::TDM;
    const GENAI: &str = UsagePreferences::GENAI;
    const AI: &str = UsagePreferences::AI;
    const SEARCH: &str = UsagePreferences::SEARCH;
    const ALL: &[&str] = &[TDM, AI, GENAI, SEARCH];

    trait UsagePreferencesExt {
        fn assert_unset(&self, usage: &str);
        fn assert_allowed(&self, usage: &str);
        fn assert_denied(&self, usage: &str);
    }

    impl UsagePreferencesExt for UsagePreferences {
        fn assert_unset(&self, usage: &str) {
            // There isn't an API for testing if something is set.
            // This checks that by testing that both defaults pass through.
            assert_eq!(self.eval(usage, Denied), Denied);
            assert_eq!(self.eval(usage, Allowed), Allowed);
        }
        fn assert_allowed(&self, usage: &str) {
            assert_eq!(self.eval(usage, Denied), Allowed);
        }
        fn assert_denied(&self, usage: &str) {
            assert_eq!(self.eval(usage, Allowed), Denied);
        }
    }

    #[test]
    fn make_blank() {
        let up = UsagePreferences::blank();
        assert_eq!(up.items.len(), 0);
        assert_eq!(up.max_len, 0);
    }

    #[test]
    fn make_default() {
        let up = UsagePreferences::default();
        assert_eq!(up.items.len(), 4);
        assert_eq!(up.max_len, 6);
    }

    #[test]
    #[should_panic(expected = "duplicate usage added")]
    fn add_duplicate() {
        let mut up = UsagePreferences::default();
        up.add("tdm");
    }

    #[test]
    #[should_panic(expected = "usage name cannot contain a comma")]
    fn add_comma() {
        let mut up = UsagePreferences::default();
        up.add("this,");
    }

    #[test]
    #[should_panic(expected = "usage name cannot contain equals")]
    fn add_equals() {
        let mut up = UsagePreferences::default();
        up.add("this=y");
    }

    #[test]
    #[should_panic(expected = "parent not found")]
    fn add_no_parent() {
        let mut up = UsagePreferences::default();
        up.add_child("this", "no");
    }

    #[test]
    fn allow_tdm() {
        let mut up = UsagePreferences::default();
        up.parse("tdm=y");
        for usage in ALL {
            up.assert_allowed(usage);
        }
    }

    #[test]
    fn allow_ai() {
        let mut up = UsagePreferences::default();
        up.parse("ai=y");
        up.assert_unset(TDM);
        up.assert_allowed(AI);
        up.assert_allowed(GENAI);
        up.assert_unset(SEARCH);
    }

    #[test]
    fn deny_search() {
        let mut up = UsagePreferences::default();
        up.parse("search=n");
        up.assert_unset(TDM);
        up.assert_unset(AI);
        up.assert_unset(GENAI);
        up.assert_denied(SEARCH);
    }

    #[test]
    fn full() {
        let mut up = UsagePreferences::default();
        up.parse("genai=y,search=n,tdm=y,ai=n");
        up.assert_allowed(TDM);
        up.assert_denied(AI);
        up.assert_allowed(GENAI);
        up.assert_denied(SEARCH);
    }

    #[test]
    fn full_split() {
        let mut up = UsagePreferences::default();
        up.parse("search=y,tdm=n");
        up.parse("genai=n,ai=y");
        up.assert_denied(TDM);
        up.assert_allowed(AI);
        up.assert_denied(GENAI);
        up.assert_allowed(SEARCH);
    }

    #[test]
    fn deny_overrides_allow() {
        let mut up = UsagePreferences::default();
        up.parse("ai=y,ai=n,ai=y");
        up.assert_unset(TDM);
        up.assert_denied(AI);
        up.assert_denied(GENAI);
        up.assert_unset(SEARCH);
    }

    #[test]
    fn whitespace() {
        let mut up = UsagePreferences::default();
        up.parse(", tdm\t=\ry\n,,   =");
        for usage in ALL {
            up.assert_allowed(usage);
        }
    }

    #[test]
    fn invalid_value() {
        let mut up = UsagePreferences::default();
        up.parse("tdm=junk,tdm=y,tdm=no");
        for usage in ALL {
            up.assert_allowed(usage);
        }
    }

    #[test]
    fn invalid_key() {
        let mut up = UsagePreferences::default();
        up.parse("a=y,bcdefghijklmnopqrstuvwxyz=y");
        up.assert_unset("a");
        up.assert_unset("bcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn invalid_unicode() {
        let mut up = UsagePreferences::default();
        up.parse(&[0xff, 0x00]);
        for usage in ALL {
            up.assert_unset(usage);
        }
    }

    #[test]
    fn custom_domain() {
        let mut up = UsagePreferences::blank();
        up.add("a");
        up.add("b");
        up.add_child("c", "a");

        up.parse("a=y,b=y");
        up.assert_allowed("a");
        up.assert_allowed("b");
        up.assert_allowed("c");
    }
}
