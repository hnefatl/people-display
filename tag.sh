#!/bin/bash

if [[ $# -lt 1 ]] ; then
  echo "Usage: tag.sh (display|exporter) <version>"
  exit 1
fi

latest_tag=$(git describe --match "$1*" | sed "s/$1-\(.*\)-.*-.*/\1/")
echo "Latest tag is $latest_tag"

if [[ $# -ne 2 ]] ; then
  exit 2
fi

git tag -a "$1" -m "$2"
git push origin "$1"
