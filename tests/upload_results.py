#!/usr/bin/env python
"""
This script uploads the screenshots to a server so that they can be further
investigated. The key is obtained via secret travis environment variables to
prevent unauthorized uploads.

You should not run this script locally.
"""
import os
import requests
import sys

if (os.getenv("TRAVIS") == "true" and
      os.getenv("TRAVIS_SECURE_ENV_VARS") != "true"):
    sys.stderr.write("Disabld for pull requests.\n")
    sys.exit(2)

if (os.getenv("TRAVIS") != "true" or
      os.getenv("TRAVIS_SECURE_ENV_VARS") != "true"):
    sys.stderr.write(__doc__)
    sys.exit(1)

try:
    os.chdir(os.path.join(os.path.dirname(__file__), os.pardir, 'test-results'))
except OSError:
    sys.stderr.write("No results produced.")
    sys.exit(3)


TARGET_URL = "https://kingdread.de/eltrur/upload"


data = {
    "key": os.getenv("UPLOAD_KEY"),
    "branch": os.getenv("TRAVIS_BRANCH"),
    "commit": os.getenv("TRAVIS_COMMIT"),
    "build": os.getenv("TRAVIS_BUILD_NUMBER"),
    "job": os.getenv("TRAVIS_JOB_NUMBER"),
    "os": os.getenv("TRAVIS_OS_NAME"),
    "rust-version": os.getenv("TRAVIS_RUST_VERSION"),
    "url": "https://travis-ci.org/Kingdread/Rurtle/jobs/" +
           os.getenv("TRAVIS_JOB_ID"),
}

with open("test-result") as report:
    filenames = [line.rstrip("\r\n").split("/")[2] for line in report]

files = [
    ("report", open("test-result", "rb")),
]
for filename in filenames:
    files.append(("shots", open(filename, "rb")))


response = requests.post(TARGET_URL, data=data, files=files)
print "== Reponse: {} ==".format(response.status_code)
print response.text
