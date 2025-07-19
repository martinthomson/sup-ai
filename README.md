<!-- regenerate: n -->

# Short Usage Preference Strings for Automated Processing

This is the working area for the individual Internet-Draft, "Short Usage Preference Strings for Automated Processing".

* [Editor's Copy](https://martinthomson.github.io/sup-ai/#go.draft-thomson-aipref-sup.html)
* [Datatracker Page](https://datatracker.ietf.org/doc/draft-thomson-aipref-sup)
* [Individual Draft](https://datatracker.ietf.org/doc/html/draft-thomson-aipref-sup)
* [Compare Editor's Copy to Individual Draft](https://martinthomson.github.io/sup-ai/#go.draft-thomson-aipref-sup.diff)


## Rust Implementation

This includes a simple Rust implementation, which can be used as follows:

```rust
use sup_ai::{UsagePreferences, UsagePreference::Allowed};

// Construct usage preferences with the default usages.
let mut up = UsagePreferences::default();

// An expression is a string or bytes, as dictated by the source.
// This might be sourced from robots.txt, an HTTP header, metadata, or anywhere.
let expression = "tdm=y,ai=n";
up.parse(expression);

// Evaluation determines whether the preference is to allow or deny the usage.
let result = up.eval("search", Allowed);
assert_eq!(result, Allowed);
```

## Contributing

See the
[guidelines for contributions](https://github.com/martinthomson/sup-ai/blob/main/CONTRIBUTING.md).

Contributions can be made by creating pull requests.
The GitHub interface supports creating pull requests using the Edit (✏) button.


## Command Line Usage

Formatted text and HTML versions of the draft can be built using `make`.

```sh
$ make
```

Command line usage requires that you have the necessary software installed.  See
[the instructions](https://github.com/martinthomson/i-d-template/blob/main/doc/SETUP.md).

