# codeprints client

A command-line client which analyzes local/private git repositories  
and generates a data file for [codeprints.dev](https://codeprints.dev/).

## Why?

A lot of code is not public on Github; especially commerical projects.
Nevertheless there is demand for creating prints from a repository
(e.g. to give them as a present to each team member after reaching a major milestone).

This is why we offer a client that can be used locally without having to
make any code public or install any dependencies.

## Usage

1. Navigate to any local git repository. 
2. Run the following command to generate a `codeprints.json` for the repo:

```
docker run -v `pwd`/repo codeprints/client 
```

(This will not parse any sensitive data. It is merely a wrapper around
`git log --date=short-local --pretty=format:%ad`.)

3. Upload the JSON file to codeprints.dev to render a print.

## How the Github Contribution Timeline works

The code is based on a great post about the contributions calendar.  
The "algorithm" behind it is [surprisingly sophisticated](https://bd808.com/blog/2013/04/17/hacking-github-contributions-calendar/).

## Support

In case you run into problems, don't hestitate to open an issue.  
We're always happy about code contributions as well of course.  
For business inquiries, please reach out to us at support@codeprints.dev.
