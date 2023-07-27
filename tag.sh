#!/bin/bash

if [[ $# -ne 2 ]] ; then
  echo "Usage: tag.sh <tag> <message>"
  exit 1
fi

git tag -a "$1" -m "$2"
git push "$1"
