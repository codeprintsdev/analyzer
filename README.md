# codeprints analyzer

[![rust](https://github.com/codeprintsdev/analyzer/actions/workflows/rust.yml/badge.svg)](https://github.com/codeprintsdev/analyzer/actions/workflows/rust.yml)

## Table of contents

* [Why](#why)
* [Usage](#usage)
* [Limiting the time range](#limiting-the-time-range)
* [Limiting authors/committers](#limiting-authorscommitters)
* [More options](#more-options)

![](assets/framed.jpg)

A command-line tool which analyzes local/private git repositories  
and generates a data file for [codeprints.dev](https://codeprints.dev/).

## Why?

A lot of code is not public on Github; especially commercial projects.
Nevertheless there is demand for creating prints from a repository
(e.g. to give them as a present to each team member after reaching a major milestone).

This is why we offer a standalone tool that can be used locally without having to
make any code public or install any dependencies.

## Usage

1. Navigate to any local git repository.
2. Run the following command to generate a `codeprints.json` for the repo:

```
docker run -v `pwd`:/repo codeprints/analyzer
```

(This will not parse any sensitive data. It is merely a wrapper around
`git log --date=short-local --pretty=format:%ad`.)

Alternatively you can also run the Rust binary without Docker:

```
# Install the tool
cargo install --git https://github.com/codeprintsdev/analyzer

# Use it inside any git repository
codeprints-analyzer
```

3. Upload the JSON file to codeprints.dev to render a print.

## Limiting the time range

You can set the start- and end-date of the output.

```
docker run -v `pwd`:/repo codeprints/analyzer --after "2020-12-24" --before "2021-02-10"
```

The syntax is exactly the same that `git` also uses.
In fact we just pass the parameters to `git log`.

## Limiting authors/committers

If you work in a bigger team, you might want to filter the contributions by
author. Here is how:

```
docker run -v `pwd`:/repo codeprints/analyzer --author "Matthias" --author "Octocat"
```

To get a list of all author names, run `git shortlog --summary --numbered --email`.

(You can also filter by committers. The difference is subtle, but in contrast to authors, these are the
contributors who pushed/committed a patch to the repository.)

## More options

To get an exhaustive list of options, run

```
docker run codeprints/analyzer --help
```

## Background: How the Github Contribution Timeline works

The code is based on a great post about the Github contributions calendar.  
The logic around it is [surprisingly sophisticated](https://bd808.com/blog/2013/04/17/hacking-github-contributions-calendar/).

## Support

In case you run into problems, don't hestitate to open an issue.  
We're always happy about code contributions as well of course.  
For business inquiries, please reach out to us at support@codeprints.dev.

## License

lychee is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.
