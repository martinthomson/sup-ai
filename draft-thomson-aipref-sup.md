---
title: "Short Usage Preference Strings for AI"
category: std

docname: draft-thomson-aipref-sup-latest
submissiontype: IETF
number:
updates: 9309
date:
consensus: true
v: 3
area: "Web and Internet Transport"
workgroup: "AI Preferences"
keyword:
 - nope
venue:
  group: "AI Preferences"
  type: "Working Group"
  mail: "ai-control@ietf.org"
  arch: "https://mailarchive.ietf.org/arch/browse/ai-control/"
  github: "martinthomson/sup-ai"
  latest: "https://martinthomson.github.io/sup-ai/draft-thomson-aipref-sup.html"

author:
 -
    fullname: "Martin Thomson"
    organization: Mozilla
    email: "mt@lowentropy.net"

normative:

informative:


--- abstract

Content creators and other stakeholders might wish to signal
their preferences about how their content
might be consumed by automated systems.
This document defines a very simple format for expressions.

This document updates RFC 9309 to define one means of conveyance.
An HTTP header field is also defined.


--- middle

# Introduction

The automated consumption of content by crawlers and other machines
has increased significantly in recent years.
This is partly due to the training of machine-learning models.

Content creators and other stakeholders,
such as distributors,
might wish to have a say in what types of usage
by automatons is acceptable.
At the same time,
the operator of an automated system might
be happy to follow any guidance given.

In the absence of a clear means of indicating preferences,
there is no way for preferences to conveyed.
This document seeks to address this shortcoming.

The format of preferences is a simple string
that looks something like this:

~~~
ai=n,search=y
~~~

This format seeks to be:

* Simple to understand and create
* Straightforward to consume
* Easily conveyed in multiple ways


## Applicability

These expressions do not consider the process
of acquiring content.
They only express preferences regarding
how the content is ultimately used.
In particular,
this includes both the training of machine learning models
and the use of those models.

This is not intended to replace copyright licensing
or other means of conveying similar information.
Rather, it is intended to provide clear information
that can be used by automatons to quickly identify
whether content is available or not for processing.

Preference expressions do not apply
where usage is explicitly allowed or disallowed in law
or when licensing has been pre-arranged.

This document assumes that
automatons have a default policy
that applies when no preferences have been expressed.
How those defaults are determined is not specified.



## Conventions and Definitions

{::boilerplate bcp14-tagged}


# Preference Expression Strings

A preference expression string is a Unicode string.

A preference expression string comprises zero or more directives,
where each directive is separated by a COMMA ("," or U+2C).


## Preference

Each preference consists of a label,
an EQUALS character ("=" or U+3D),
and value of either "y" (U+79) or "n" (U+6E).

Preferences that do not conform to this syntax are ignored
and have no effect.

Some whitespace --
specifically spaces (U+20) and horizontal tabs (U+9) --
are ignored to make authoring easier.
Whitespace is ignored before and after both labels and values.


## Labels

Each label describes how an automaton might use content.
A set of core labels is defined in this document; see {{usage}}.

A label that is not understood by an automaton are ignored.
Similarly, labels that do not apply to a usage are ignored.


## Values

Each preference includes one of two values: "y" or "n".

A value of "y" indicates a preference to allow the associated usage,
unless there is a label for a more specific usage
that has a value of "n".

A value of "n" indicates a preference to deny the associated usage,
unless there is a label for a more specific usage
that has a value of "y".

Any other value causes a preference to be ignored.


## Duplicated Labels {#duplicates}

Preferences with duplicated labels are permitted.
This can happen when multiple preference expressions are combined;
see {{multiple}}.

If multiple preferences include the same label,
any preference with a value of "n" applies.
Failing that, any preference with a value of "y" is used.
Preferences that cannot be parsed successfully
as either "y" or "n"
are ignored.

For example, the following indicates a preference
to deny artificial intelligence uses.

~~~
ai=y,ai=n,ai=y,unknown=y
~~~

The order of duplicate labels has no effect on the outcome,
until the preference expression string exceeds the length
permitted by the automaton processing it.


## Specificity

A label might be defined to be more specific than another label.

The value for the most specific applicable label applies,
even if the more general label has a contradictory value.

For example, the following indicates a preference
to allow a use in generative AI applications,
but not for other AI applications:

~~~
garbage!!!,genai=y,ai=n
~~~

The order of preferences and the presence of invalid values
has no effect on the outcome.


# Usage Labels {#usage}

This section defines a limited taxonomy of usages,
with just the following broad usage labels:

tdm:

: The "tdm" label relates to all forms of automated processing of content.
  The acronym TDM stands for "text and data mining".
  This is a broad category
  that refers to the practice of the automated extraction of any sort
  of information from content.

ai:

: The "ai" label describes usage
  for all forms of artificial intelligence or machine learning.
  This includes both the training of models using the content
  and the use of a model that has been trained using the content.
  The "ai" label is more specific than "tdm".

genai:

: The "genai" label describes usage
  for artificial intelligence or machine learning models
  that are able to produce content.
  That content does not need to be similar in form
  to the content that a preference expression is applied.
  This includes both the training of models using the content
  and the use of a model that has been trained using the content.
  The "genai" label is more specific than "ai".

search:

: The "search" label describes a usage
  whereby content is indexed aid in its discovery
  and the use of those indexes to find content.
  The "search" label is more specific than "tdm".

{:aside}
> Note that although search applications often use machine learning,
  "search" is not more specific than the "ai" label.


## Defining New Labels {#new}

The set of labels can be expanded,
but the core set is intended to be small.
The most important goal is to ensure that each label is
clearly understood and widely recognized.
A proliferation of choices makes it harder
to choose the right label to use
and increases the chances that a label is not recognized by an automaton.

An entity processing a preference expression
only uses preferences that are known to it.
Preferences that contain labels unknown are ignored.
This potentially allows for the inclusion of preferences for new usages.

Because new labels might not be recognized,
it might be necessary for preference expressions
to include values for more general labels
in addition to the new and more specific label.

For example, if a new "example" usage
is defined to more specific
than the existing "tdm" label,
there are four potential expressions.
Depending on whether an automaton is updated to support the new label,
the outcome differs,
as shown in {{table-new-label}}.

| example | tdm | updated |   old   |
|:-------:|:---:|:--------|:--------|
| n       | n   | DENIED  | DENIED  |
| n       | y   | DENIED  | ALLOWED |
| y       | n   | ALLOWED | DENIED  |
| y       | y   | ALLOWED | ALLOWED |
{: #table-new-label title="Backward Compatibility for New Labels"}

If automatons that might use content in the new way
can also be expected to understand the new label,
then the new label can be safely used.
Otherwise, there is a risk that automatons infer permission
based on more general preferences.

{{iana}} describes a process for registering new labels.


## Label Characters {#new-chars}

A preference expression
cannot include certain characters
when carried in specific protocols (see {{carriage}}):

* A "#" (U+23) character is interpreted as a line ending
  in the "robots.txt" format {{robots}}.

* The structured field dictionaries used for the
  auto-usage field in HTTP {{http}}
  limit character repertoire to
  lowercase alphabetic characters ("a" through "z"),
  digits ("0" through "9"),
  "_", "-", ".", and "*".

The format does not prohibit the use of these characters.
However, any new labels intended for wide compatibility
SHOULD use only lowercase alphabetical characters.


# Processing Preference Expression Strings {#algorithm}

This section details an example algorithm
for processing of preference expressions.

An automaton that receives a preference expression uses the following algorithm,
or any process with an equivalent outcome,
to determine whether a particular usage is ALLOWED or DENIED according to that expression.

1. The automaton parses the preference expression
   and produces a record of values for each label known to it
   following the process in {{parse}}.

2. The automaton selects the labels that apply to its current usage,
   including labels at all levels of generality.
   These are passed to the determination process in {{deter}}
   to determine whether the usage is ALLOWED or DENIED.


## Parsing {#parse}

The process of parsing a preference expression
and updating the record of values, is as follows:

1. The automaton initializes a record one variable
   for each of the labels that is known to it
   with a value of UNKNOWN.
   Include labels in this record
   that are more general,
   even if the specific usage is properly described by that subset
   (for example, include "tdm" in addition to "search" for a simple search indexing function).

2. The expression is split into a sequence of preferences
   by splitting the string at each COMMA (",", U+2C).

3. Each preference is then processed in turn:

   a. The preference is split into a label and a value
      at the first EQUALS ("=", U+3D).
      Preferences that do not contain an EQUALS ("=", U+3D) are skipped
      and processing continues with the next preference.

   b. Spaces (U+20) and tabs (U+9) are trimmed from the start and end
      of each label and value.
      (ISSUE: move this up a step for SF compatibility and easier parsing?)

   c. If the label is not understood by the automaton
      or the label does not apply to the current usage,
      it is ignored and the processing continues with the next preference.

   d. If the value is the string "n" (a single U+6E),
      the record for that the corresponding label is set to NO.

   e. If the value is the string "y" (a single U+79),
      and the current record for the corresponding label is not set,
      that record is set to YES.


## Determination {#deter}

When values have been recorded
for known labels,
the automaton selects the labels that applies to a specific usage.

This includes more general labels.
For example, a "genai" application
would include both "ai" and "tdm" labels.

The following process can be followed to determine
whether that usage is ALLOWED or DENIED
according to a preference expression.

1. Starting with the most general labels,
   iterate over the recorded value for each label.

   a. For each of the labels
      that are more specific than this label,
      replace any value of UNKNOWN for that more specific label
      with the value from this (more general) label.

   b. If all aspect of the usage
      is covered by more specific labels
      discard the more general label from the record.

2. If the value for any of the remaining labels is UNKNOWN,
   the automaton substitutes a YES or NO value
   based on its configured policy for missing preferences
   for the corresponding label.

3. If the value for any of the remaining labels is NO,
   resolve that the indicated preference
   regarding the usage
   is DENIED.

4. Otherwise, resolve that the indicated preference
   regarding the usage
   is ALLOWED.

In addition to understanding which labels apply to a given usage,
automatons need to have a default policy
for each of those labels.


## Optimization and Length Limits

The parsing of preference expressions
can be implemented in a streaming fashion,
requiring minimal state beyond that needed to track the value of applicable labels.

A preference expression of arbitrary length can be processed efficiently.
However, implementations MAY choose to stop processing expressions
at any point once at least 1000 characters have been processed.


## Multiple Preference Expressions {#multiple}

An automaton that is consuming content
might encounter multiple preference expressions
that relate to the same piece of content.
For example, preference expressions might be carried by
both "robots.txt" ({{robots}}) and content metadata ({{metadata}}).

Multiple preference expressions can be combined
by simple concatenation in any order.
A single comma (",") is inserted between each expression.

For an identical outcome, the parsing process ({{parse}})
can be run separately for each expression,
updating the same record of values
rather than starting anew for each.


## Multiple Usages {#multi}

An automaton that might use content for multiple purposes
can parse preference expressions once.

The parsing process ({{parse}}) can be run,
recording the value of all labels
that apply to any of the potential usages.

The state that is produced from parsing
can be retained and applied ({{deter}}) to each different usage.


# Carrying Preference Expressions {#carriage}

A preference expression might be propagated by multiple different mechanisms.
This provides content creators and distributors choice
in how they manage the signaling of their preferences.


## The robots.txt Format {#robots}

The robots.txt file {{!ROBOTS=RFC9309}},
also known as the Robots Exclusion Protocol,
provides automated crawlers with information
about what resources can be gathered.

This is a file that is served in a well-known location by HTTP servers.

An extension is defined for this format
to allow for carrying simple usage preference strings.
A rule name of "usage-pref" is added to each group,
appearing between the `startgroupline` and `rule` lines as follows:

~~~abnf
group = startgroupline
        *(startgroupline / emptyline)
        *(usage / emptyline)
        *(rule / emptyline)
usage = *WS "usage-pref" *WS ":" *UTF8-char-noctl EOL
~~~

Notes:

* The label "usage-pref" is case insentive,
  but the usage preference string is case sensitive.

* Preference expressions will be truncated
  at the first "#" character (U+23);
  see {{new-chars}}.


## HTTP Header Field {#http}

An HTTP Response Header field called Usage-Pref is defined.

This field is defined as a structured field dictionary,
as defined in {{Section 3.2 of !RFC9651}}.

Dictionary members MUST all include a single token value
({{Section 3.3.4 of !RFC9651}}) that is either "y" or "n".

The value of this field MAY be first parsed
using a structured field parser.
This implies the following restrictions:

* Structured field parsers do not permit whitespace
  between dictionary key, the "=" separator, and value.
  Therefore, this additional space MUST be removed
  before creating the field.

* Dictionary members are tokens,
  not Booleans ({{Section 3.3.6 of ?RFC9651}}).
  This means that members with no value or a value of "?1"
  MUST be ignored.

* Dictionary members with parameters MUST be ignored.


## Content Metadata {#metadata}

Specific formats can define how preference expressions
are carried in content metadata.

This document does not define any such format.

{:aside}
> Question:
> Should we define how to use HTML meta, or rely on http-equiv?


# Security Considerations

This document defines a preference mechanism,
not a security feature.

The primary consideration for security
is the correct and efficient implementation of parsing.
Errors in parsers have been known to be exploited by actors
that produce malicious input
to effect one of two common types of problem:

* buffer overruns,
  where malicious input is formed to enable remote code execution

* denial of service,
  where malicious input is constructed to be difficult to process
  in order to exhaust resources

The algorithms in {{algorithm}} are not immune to these issues
if incautiously implemented by an automaton.
They are exemplary only
and are biased toward being comprehensible
rather than being secure.


# IANA Considerations {#iana}

TODO establish a registry for labels


--- back

# Acknowledgments
{:numbered="false"}

TODO acknowledge.
