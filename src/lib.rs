use std::cmp::max;

#[cfg(feature = "robots")]
pub mod robots;

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
enum State {
    #[default]
    Unknown,
    Yes,
    No,
}

impl State {
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

impl TryFrom<State> for UsagePreference {
    type Error = ();
    fn try_from(value: State) -> Result<Self, Self::Error> {
        match value {
            State::Unknown => Err(()),
            State::Yes => Ok(Self::Allowed),
            State::No => Ok(Self::Denied),
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    name: Vec<u8>,
    parent: Option<usize>,
    value: State,
}

#[derive(Debug, Clone)]
pub struct UsagePreferences {
    items: Vec<Item>,
    max_len: usize,
}

impl UsagePreferences {
    /// Automated Processing.
    ///
    /// The act of using one or more assets in the context of automated processing
    /// aimed at analyzing text and data in order to generate information
    /// which includes but is not limited to patterns, trends and correlations.
    pub const ALL: &str = "all";
    /// Artificial Intelligence (or Machine Learning) Training.
    ///
    /// The act of training machine learning models or artificial intelligence (AI).
    pub const TRAIN_AI: &str = "train-ai";
    /// Generative AI Training.
    ///
    /// The act of training general purpose AI models that have the capacity
    /// to generate text, images or other forms of synthetic content, or
    /// the act of training more specialized AI models that have the purpose
    /// of generating text, images or other forms of synthetic content.
    pub const TRAIN_GENAI: &str = "train-genai";
    /// AI Use.
    ///
    /// The act of using one or more assets as input to a trained AI/ML model
    /// as part of the operation of that model (as opposed to the training of the model).
    pub const AI_USE: &str = "ai-use";
    /// Search.
    ///
    /// Using one or more assets in a search application
    /// that directs users to the location from which the assets were retrieved.
    pub const SEARCH: &str = "search";

    /// Instantiate a blank usage preferences set with no usages registered.
    ///
    /// Note: Use the `Default` implementation to get the standard set of usages.
    #[must_use]
    pub fn blank() -> Self {
        Self {
            items: Vec::new(),
            max_len: 0,
        }
    }

    /// Common logic for the addition of a usage.
    fn add_inner(&mut self, usage: impl AsRef<[u8]>, parent: Option<usize>) {
        let name = usage.as_ref();
        assert!(!name.contains(&b','), "usage name cannot contain a comma");
        assert!(!name.contains(&b'='), "usage name cannot contain equals");
        assert!(
            !self.items.iter().any(|it| it.name == name),
            "duplicate usage added"
        );
        self.max_len = max(self.max_len, name.len());
        self.items.push(Item {
            name: name.to_vec(),
            parent,
            value: State::Unknown,
        });
    }

    /// Add a usage that this object will track.
    pub fn add(&mut self, usage: impl AsRef<[u8]>) {
        self.add_inner(usage, None);
    }

    /// Add a usage that this object will track.
    /// Include the identified parent type, which will be used if there is no preference
    /// expressed for this usage.
    ///
    /// # Panics
    /// This panics if the identified parent cannot be found.
    pub fn add_child(&mut self, usage: impl AsRef<[u8]>, parent: impl AsRef<[u8]>) {
        let parent = parent.as_ref();
        let p = self
            .items
            .iter()
            .position(|it| it.name == parent)
            .expect("parent not found");
        self.add_inner(usage, Some(p));
    }

    /// Determine the state of a given thing,
    /// including the cascade from higher-level items in the hierarchy.
    fn get_state(&self, mut i: usize) -> State {
        loop {
            if self.items[i].value != State::Unknown {
                return self.items[i].value;
            }
            i = if let Some(p) = self.items[i].parent {
                debug_assert!(p < i, "avoid any potential infinite loop");
                p
            } else {
                return State::Unknown;
            };
        }
    }

    /// Find the index of the given item.
    fn index_of(&self, usage: &[u8]) -> Option<usize> {
        self.items.iter().position(|it| it.name == usage)
    }

    /// Evaluate the usage preference against the given usage.
    pub fn eval(&self, usage: impl AsRef<[u8]>, dflt: UsagePreference) -> UsagePreference {
        let Some(i) = self.index_of(usage.as_ref()) else {
            return dflt;
        };
        UsagePreference::try_from(self.get_state(i)).unwrap_or(dflt)
    }

    /// Combine two sets of preferences.
    pub fn merge(&mut self, other: &Self) {
        for item in &mut self.items {
            if let Some(idx) = other.index_of(&item.name) {
                item.value.merge(other.get_state(idx))
            }
        }
    }

    /// Parse the provided input.
    ///
    /// This adds the rules in the provided string to those that this object already holds.
    #[cfg(feature = "sfv")]
    pub fn parse(&mut self, expr: impl AsRef<[u8]>) {
        let parser = ::sfv::Parser::new(&expr);
        let mut visitor = crate::sfv::PreferenceVisitor { dict: self };
        let _ignore_err = parser.parse_dictionary_with_visitor(&mut visitor);
    }

    /// Parse the provided input.
    ///
    /// This adds the rules in the provided string to those that this object already holds.
    #[cfg(not(feature = "sfv"))]
    pub fn parse(&mut self, expr: impl AsRef<[u8]>) {
        crate::manual::parse(self, expr);
    }
}

#[cfg(feature = "sfv")]
mod sfv {
    use sfv::{
        BareItemFromInput, Error as SfvError, KeyRef,
        visitor::{
            DictionaryVisitor, EntryVisitor, Ignored, InnerListVisitor, ItemVisitor,
            ParameterVisitor,
        },
    };

    use super::{State, UsagePreferences};

    pub struct PreferenceVisitor<'a> {
        pub dict: &'a mut UsagePreferences,
    }

    impl<'a> DictionaryVisitor<'a> for PreferenceVisitor<'_> {
        type Error = SfvError;

        fn entry<'dv, 'ev>(
            &'dv mut self,
            key: &'a KeyRef,
        ) -> Result<impl EntryVisitor<'ev>, Self::Error>
        where
            'dv: 'ev,
        {
            // A linear search is good enough for a small vocabulary.
            let item = self.dict.items.iter_mut().find_map(|p| {
                if p.name == key.as_str().as_bytes() {
                    Some(&mut p.value)
                } else {
                    None
                }
            });
            Ok(item.map(|item| UsageVisitor { item }))
        }
    }

    struct UsageVisitor<'a> {
        item: &'a mut State,
    }

    impl<'a> ItemVisitor<'a> for UsageVisitor<'_> {
        type Error = SfvError;

        fn bare_item<'pv>(
            self,
            bare_item: BareItemFromInput<'a>,
        ) -> Result<impl ParameterVisitor<'pv>, Self::Error> {
            if let Some(v) = bare_item.as_token() {
                match v.as_str() {
                    "y" => self.item.merge(State::Yes),
                    "n" => self.item.merge(State::No),
                    _ => {}
                }
            }
            Ok(Ignored)
        }
    }

    impl EntryVisitor<'_> for UsageVisitor<'_> {
        fn inner_list<'ilv>(self) -> Result<impl InnerListVisitor<'ilv>, Self::Error> {
            Ok(Ignored) // do nothing
        }
    }
}

#[cfg(not(feature = "sfv"))]
mod manual {
    use std::iter::Peekable;

    use super::{Item, State, UsagePreferences};

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

    fn parse_name(items: &[Item], r: &mut impl Input, max_len: usize) -> Option<usize> {
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
            if let Some(pos) = items.iter().position(|it| it.name == usage) {
                return Some(pos);
            }
        }
        None
    }

    fn parse_value(r: &mut impl Input) -> State {
        r.skip_ws();
        let v = match r.next() {
            Some(b'y') => State::Yes,
            Some(b'n') => State::No,
            _ => State::Unknown,
        };
        r.skip_ws();
        if matches!(r.peek(), None | Some(b',')) {
            v
        } else {
            State::Unknown
        }
    }

    pub fn parse(prefs: &mut UsagePreferences, expr: impl AsRef<[u8]>) {
        let mut r = expr.as_ref().iter().peekable();
        while r.peek().is_some() {
            if let Some(i) = parse_name(&prefs.items, &mut r, prefs.max_len) {
                let v = parse_value(&mut r);
                prefs.items[i].value.merge(v);
            }
            r.skip_until(|c| c == b',');
            _ = Iterator::next(&mut r); // Discard any ','.
        }
    }
}

impl Default for UsagePreferences {
    fn default() -> Self {
        let mut v = Self {
            items: Vec::with_capacity(4),
            max_len: 0,
        };
        v.add(Self::ALL);
        v.add_child(Self::TRAIN_AI, Self::ALL);
        v.add_child(Self::TRAIN_GENAI, Self::TRAIN_AI);
        v.add_child(Self::AI_USE, Self::ALL);
        v.add_child(Self::SEARCH, Self::ALL);
        v
    }
}

#[cfg(test)]
trait UsagePreferencesAssertions {
    fn assert_unset(&self, usage: &str);
    fn assert_allowed(&self, usage: &str);
    fn assert_denied(&self, usage: &str);
}

#[cfg(test)]
impl UsagePreferencesAssertions for UsagePreferences {
    fn assert_unset(&self, usage: &str) {
        // There isn't an API for testing if something is set.
        // This checks that by testing that both defaults pass through.
        assert_eq!(
            self.eval(usage, UsagePreference::Denied),
            UsagePreference::Denied
        );
        assert_eq!(
            self.eval(usage, UsagePreference::Allowed),
            UsagePreference::Allowed
        );
    }
    fn assert_allowed(&self, usage: &str) {
        assert_eq!(
            self.eval(usage, UsagePreference::Denied),
            UsagePreference::Allowed
        );
    }
    fn assert_denied(&self, usage: &str) {
        assert_eq!(
            self.eval(usage, UsagePreference::Allowed),
            UsagePreference::Denied
        );
    }
}

#[cfg(test)]
mod test {
    use crate::{UsagePreferences, UsagePreferencesAssertions};

    const ALL: &str = UsagePreferences::ALL;
    const TRAIN_GENAI: &str = UsagePreferences::TRAIN_GENAI;
    const TRAIN_AI: &str = UsagePreferences::TRAIN_AI;
    const SEARCH: &str = UsagePreferences::SEARCH;
    const AI_USE: &str = UsagePreferences::AI_USE;
    const EVERYTHING: &[&str] = &[ALL, TRAIN_AI, TRAIN_GENAI, SEARCH];

    #[test]
    fn make_blank() {
        let up = UsagePreferences::blank();
        assert_eq!(up.items.len(), 0);
        assert_eq!(up.max_len, 0);
    }

    #[test]
    fn make_default() {
        let up = UsagePreferences::default();
        assert_eq!(up.items.len(), 5);
        assert_eq!(up.max_len, 11);
    }

    #[test]
    #[should_panic(expected = "duplicate usage added")]
    fn add_duplicate() {
        let mut up = UsagePreferences::default();
        up.add("all");
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
        up.parse("all=y");
        for usage in EVERYTHING {
            up.assert_allowed(usage);
        }
    }

    #[test]
    fn allow_ai() {
        let mut up = UsagePreferences::default();
        up.parse("train-ai=y");
        up.assert_unset(ALL);
        up.assert_allowed(TRAIN_AI);
        up.assert_allowed(TRAIN_GENAI);
        up.assert_unset(SEARCH);
        up.assert_unset(AI_USE);
    }

    #[test]
    fn deny_search() {
        let mut up = UsagePreferences::default();
        up.parse("search=n");
        up.assert_unset(ALL);
        up.assert_unset(TRAIN_AI);
        up.assert_unset(TRAIN_GENAI);
        up.assert_denied(SEARCH);
        up.assert_unset(AI_USE);
    }

    #[test]
    fn full() {
        let mut up = UsagePreferences::default();
        up.parse("train-genai=y,search=n,all=y,train-ai=n,ai-use=n");
        up.assert_allowed(ALL);
        up.assert_denied(TRAIN_AI);
        up.assert_allowed(TRAIN_GENAI);
        up.assert_denied(SEARCH);
        up.assert_denied(AI_USE);
    }

    #[test]
    fn full_split() {
        let mut up = UsagePreferences::default();
        up.parse("search=y,all=n");
        up.parse("train-genai=n,train-ai=y");
        up.parse("ai-use=y");
        up.assert_denied(ALL);
        up.assert_allowed(TRAIN_AI);
        up.assert_denied(TRAIN_GENAI);
        up.assert_allowed(SEARCH);
        up.assert_allowed(AI_USE);
    }

    #[test]
    fn deny_overrides_allow() {
        let mut up = UsagePreferences::default();
        up.parse("train-ai=y,train-ai=n,train-ai=y");
        up.assert_unset(ALL);
        up.assert_denied(TRAIN_AI);
        up.assert_denied(TRAIN_GENAI);
        up.assert_unset(SEARCH);
        up.assert_unset(AI_USE);
    }

    #[test]
    fn whitespace() {
        let mut up = UsagePreferences::default();

        // Using SFV means that the grammar is slightly less permissive about whitespace.
        up.parse(if cfg!(feature = "sfv") {
            "x, all=y\n,,"
        } else {
            ", all\t=\ry\n,,   ="
        });
        for usage in EVERYTHING {
            up.assert_allowed(usage);
        }
    }

    #[test]
    fn invalid_value() {
        let mut up = UsagePreferences::default();
        up.parse("all=junk,all=y,all=no");
        for usage in EVERYTHING {
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
        up.parse([0xff, 0x00]);
        for usage in EVERYTHING {
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

    #[test]
    fn merge() {
        let mut up1 = UsagePreferences::default();
        up1.parse("train-gen=y");
        let mut up2 = UsagePreferences::default();
        up2.parse("train-ai=n");
        up1.merge(&up2);
        up1.assert_unset(ALL);
        up1.assert_denied(TRAIN_AI);
        up1.assert_denied(TRAIN_GENAI);
        up1.assert_unset(AI_USE);
        up1.assert_unset(SEARCH);
    }

    #[test]
    fn merge_unrelated() {
        let mut up1 = UsagePreferences::default();
        up1.parse("train-gen=y");
        let mut up2 = UsagePreferences::blank();
        up2.add(TRAIN_AI);
        up2.add("a");
        up2.parse("train-ai=n, a=y");
        up1.merge(&up2);
        up1.assert_unset(ALL);
        up1.assert_denied(TRAIN_AI);
        up1.assert_denied(TRAIN_GENAI);
        up1.assert_unset(AI_USE);
        up1.assert_unset(SEARCH);
        up1.assert_unset("a");
    }
}
