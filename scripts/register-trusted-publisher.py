#!/usr/bin/env python3
"""Attach the release workflow to the published crate using an owner API token."""

import json
import os
import sys
from urllib.error import HTTPError
from urllib.request import Request, urlopen

ENDPOINT = "https://crates.io/api/v1/trusted_publishing/github_configs"
PUBLISHER = {
    "crate": "meta-ontology",
    "repository_owner": "link-foundation",
    "repository_name": "meta-ontology",
    "workflow_filename": "release.yml",
    "environment": None,
}


def register(token):
    if not token:
        raise ValueError("Publisher registration requires a crate owner API token with trusted-publishing scope.")
    headers = {"Authorization": token, "User-Agent": "meta-ontology-release-bootstrap"}
    # Follow pagination so a retry never adds an already registered publisher.
    url = ENDPOINT + "?crate=meta-ontology"
    while url:
        with urlopen(Request(url, headers=headers), timeout=30) as response:
            page = json.load(response)
        for config in page["github_configs"]:
            if all(config.get(key) == value for key, value in PUBLISHER.items()):
                print("release.yml trusted publisher is already attached.")
                return
        next_page = page.get("meta", {}).get("next_page")
        url = ENDPOINT + next_page if next_page else None
    data = json.dumps({"github_config": PUBLISHER}).encode()
    with urlopen(Request(ENDPOINT, data=data, headers={
        **headers, "Content-Type": "application/json",
    }, method="POST"), timeout=30) as response:
        config = json.load(response)["github_config"]
    if not all(config.get(key) == value for key, value in PUBLISHER.items()):
        raise ValueError("Registry returned an unexpected trusted publisher configuration.")
    print("Attached trusted publisher: link-foundation/meta-ontology, release.yml, no environment.")


if __name__ == "__main__":
    try:
        register(os.environ.get("CARGO_REGISTRY_TOKEN", ""))
    except HTTPError as error:
        # Never include authentication headers or request data in diagnostics.
        print(f"::error::Publisher registration failed: HTTP {error.code}. "
              "The token must belong to a verified crate owner and allow trusted publishing. "
              "A crate owner can also attach release.yml in the crates.io crate settings.", file=sys.stderr)
        sys.exit(1)
    except (ValueError, OSError) as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
