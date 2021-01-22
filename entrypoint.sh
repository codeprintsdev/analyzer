#!/usr/bin/env bash

git log --date=short --pretty=format:%ad | sort | uniq -c
