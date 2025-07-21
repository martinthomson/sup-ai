use std::{
    cmp::Ordering::{Equal, Greater, Less},
    io::{BufRead, Result},
};

use crate::UsagePreferences;

#[derive(Debug, Clone)]
struct ContentUsageLine {
    #[allow(dead_code, reason = "Tracking this for debugging purposes")]
    line: usize,
    path: String,
    usage: UsagePreferences,
}

impl ContentUsageLine {
    fn new(line: usize, path: String, usage: UsagePreferences) -> Self {
        Self { line, path, usage }
    }
}

#[derive(Debug, Clone)]
struct AdmissionLine {
    #[allow(dead_code, reason = "Tracking this for debugging purposes")]
    line: usize,
    allow: bool,
    path: String,
}

impl AdmissionLine {
    fn new(line: usize, allow: bool, path: String) -> Self {
        Self { line, allow, path }
    }

    fn is_more_specific(&self, other: &Self) -> bool {
        match self.path.len().cmp(&other.path.len()) {
            Greater => true,
            Equal => self.allow,
            Less => false,
        }
    }
}

#[derive(Default)]
struct Group {
    line: usize,
    user_agents: Vec<String>,
    usage_preferences: Vec<ContentUsageLine>,
    admissions: Vec<AdmissionLine>,
}

impl Group {
    /// Take a loosely-parsed line and integrate it into this group.
    fn parse_line(&mut self, line: usize, name: &str, value: &str) {
        if name.eq_ignore_ascii_case("content-usage") {
            {
                let (path, expr) = if value.starts_with('/') {
                    let Some((path, expr)) = value.split_once(&[' ', '\t']) else {
                        return;
                    };
                    (path, expr)
                } else {
                    ("", value)
                };
                let mut usage = UsagePreferences::default();
                usage.parse(expr);
                self.usage_preferences
                    .push(ContentUsageLine::new(line, path.to_string(), usage));
            };
        } else if name.eq_ignore_ascii_case("allow") {
            self.admissions
                .push(AdmissionLine::new(line, true, value.to_string()));
        } else if name.eq_ignore_ascii_case("disallow") {
            self.admissions
                .push(AdmissionLine::new(line, false, value.to_string()));
        }
    }

    /// Performs path matching according to the special character rules
    /// from Section 2.2.3 of RFC 9309.
    /// This assumes that the comment character ('#') has been handled;
    /// it therefore only handles the end-of-pattern ('$') and
    /// wildcard ('*').
    fn path_match(pattern: &str, path: &str) -> bool {
        let (pattern, complete) = if let Some(p) = pattern.strip_suffix('$') {
            if p.ends_with('*') {
                // A path of "/whatever*$" is pointless.
                (p.trim_end_matches('*'), false)
            } else {
                (p, true)
            }
        } else {
            (pattern, false)
        };
        let mut chunks = pattern.split('*');
        let Some(first) = chunks.next() else {
            return false;
        };
        let Some(mut remainder) = path.strip_prefix(first) else {
            return false;
        };
        for c in chunks {
            let Some(offset) = remainder.find(c) else {
                return false;
            };
            remainder = &remainder[offset + c.len()..];
        }
        !complete || remainder.is_empty()
    }

    /// Determine whether Allow/Disallow rules allow crawling of the given path.
    /// This operates across multiple groups, so that the lines that apply are all effectively
    /// merged into a single group.
    fn is_admitted<'a>(groups: impl Iterator<Item = &'a Self>, path: &str) -> bool {
        let mut current = AdmissionLine::new(0, false, String::new());
        for a in groups.flat_map(|g| &g.admissions) {
            if Self::path_match(&a.path, path) && a.is_more_specific(&current) {
                current = a.clone();
            }
        }
        current.allow
    }

    /// Obtains preferences for the given path across the provided groups.
    fn preferences<'a>(groups: impl Iterator<Item = &'a Self>, path: &str) -> UsagePreferences {
        let mut prefs = UsagePreferences::default();
        let mut len = 0;
        let mut matching = Vec::new();
        for p in groups.flat_map(|g| &g.usage_preferences) {
            if Self::path_match(&p.path, path) {
                match p.path.len().cmp(&len) {
                    Greater => {
                        matching.truncate(0);
                        len = p.path.len();
                        matching.push(p.clone());
                    }
                    Equal => matching.push(p.clone()),
                    Less => {}
                }
            }
        }
        for m in &matching {
            prefs.merge(&m.usage);
        }
        prefs
    }
}

pub struct Robots {
    groups: Vec<Group>,
}

impl Robots {
    pub fn parse(mut input: impl BufRead) -> Result<Self> {
        let mut r = Self { groups: Vec::new() };
        let mut group = Group::default();
        let mut line = 0;
        let mut ua = false;

        let mut buf = String::new();
        while input.read_line(&mut buf)? > 0 {
            line += 1;
            if let Some((name, value)) = buf
                .split_once('#')
                .map(|(a, _b)| a)
                .unwrap_or(&buf)
                .split_once(':')
                .map(|(a, b)| (a.trim_ascii(), b.trim_ascii()))
            {
                if name.eq_ignore_ascii_case("user-agent") {
                    if !ua {
                        if group.line != 0 {
                            r.groups.push(group);
                        }
                        group = Group {
                            line,
                            ..Group::default()
                        };
                        ua = true;
                    }
                    group.user_agents.push(value.to_ascii_lowercase());
                } else {
                    ua = false;
                    group.parse_line(line, name, value);
                }
            }
            buf.truncate(0);
        }
        r.groups.push(group);
        Ok(r)
    }

    fn groups(&self, user_agent: &str) -> impl Iterator<Item = &Group> {
        self.groups.iter().filter(move |g| {
            g.user_agents
                .iter()
                .any(|ua| ua.eq_ignore_ascii_case(user_agent))
        })
    }

    /// Determine the preferences that apply to a given user agent for a specific path.
    ///
    /// # Returns
    /// An option, which is `Some` when crawling is permitted,
    /// including a value that can be interrogated regarding preferences.
    pub fn preferences(
        &self,
        user_agent: impl AsRef<str>,
        path: impl AsRef<str>,
    ) -> Option<UsagePreferences> {
        let user_agent = user_agent.as_ref().to_ascii_lowercase();
        let path = path.as_ref();

        if Group::is_admitted(self.groups(&user_agent), path) {
            Some(Group::preferences(self.groups(&user_agent), path))
        } else if Group::is_admitted(self.groups("*"), path) {
            Some(Group::preferences(self.groups("*"), path))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{UsagePreferences, UsagePreferencesAssertions, robots::Robots};

    #[test]
    fn parse_basic() {
        const FILE: &[u8] = br#"
User-Agent: *
# Comments
disAllow: /
allow: /*e**mple**$ # A very bad, but still valid, use of the '*' rule
allow:/allow # Whatever
content-usage :train-ai=y, search=y
 content-usage: /*.jpg$ train-ai=n
user-agenT:WhateVer#
allow: /
allow: /**
disallow  : /*$
allow: /
content-usage: all=y
User-Agent: *
user-agent:otherrrrrr
Content-Usage: search=n
content-USAGE: /NO-PREFS
allow: # no path
"#;
        let r = Robots::parse(FILE).unwrap();
        assert!(r.preferences("ExampleBot", "/foo").is_none());
        assert!(r.preferences("whateverbot", "/example").is_some());
        let p = r.preferences("eXaMpLeBot", "/allowed").unwrap();
        p.assert_allowed(UsagePreferences::TRAIN_AI);
        let p = r.preferences("otherBot", "/allow/nope.jpg").unwrap();
        p.assert_denied(UsagePreferences::TRAIN_AI);
        let p = r.preferences("ExampleBot", "/allow/nope.jpg/blah").unwrap();
        p.assert_allowed(UsagePreferences::TRAIN_AI);
        let p = r.preferences("whatever", "/allow/nope.jpg").unwrap();
        p.assert_allowed(UsagePreferences::TRAIN_AI);
        let p = r.preferences("ExampleBot", "/allow/nope.jpg/blah").unwrap();
        p.assert_denied(UsagePreferences::SEARCH);
    }
}
