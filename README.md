# codeprints client

This command-line client allows to analyze local/private git repositories,  
generate and upload the data for print on codeprints.

## Upload a raw data to the codeprints API

```
docker run -v `pwd`/repo codeprints/client upload 
```

## Show raw data to be uploaded

```
docker run -v `pwd`/repo codeprints/client raw 
```
