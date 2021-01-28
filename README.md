# codeprints client

This command-line client allows to analyze local/private git repositories,  
generate and upload the data for print on codeprints.


## Usage

1. Navigate to any Github repository. 
2. Run the following command to generate a `codeprints.json` for the repo:

```
docker run -v `pwd`/repo codeprints/client 
```

This will not parse any sensitive data. It is merely a wrapper around
`git log --date=short-local --pretty=format:%ad`.

3. Upload the JSON file on codeprints.dev to render a print.
