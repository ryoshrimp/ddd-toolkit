#!/usr/bin/env python3
"""Verify packages by default; --publish publishes missing versions in order."""

import argparse
import json
from pathlib import Path
import subprocess
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
ORDER = ("ddd-toolkit-core", "ddd-toolkit-macro", "ddd-toolkit")


def published(name, version):
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{name}/{version}",
        headers={"User-Agent": "ddd-toolkit-release (https://github.com/ryoshrimp/ddd-toolkit)"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            data = json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise
    if data["version"]["yanked"]:
        raise RuntimeError(f"{name} {version} is yanked; choose a new version")
    return True


def release(publish=False):
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
        cwd=ROOT, text=True,
    ))
    packages = {p["name"]: p for p in metadata["packages"]
                if p["id"] in metadata["workspace_members"]}
    # Validate the entire plan before starting any irreversible operation.
    plan = [(name, packages[name]["version"]) for name in ORDER]
    for name, version in plan:
        print(f"{name} {version}", flush=True)
    for name, version in plan:
        if publish and published(name, version):
            print(f"Skipping already published {name} {version}", flush=True)
            continue
        command = ["cargo", "publish", "--locked", "-p", name]
        if not publish:
            command.append("--dry-run")
        # Cargo waits for registry availability before the next dependent crate.
        subprocess.run(command, cwd=ROOT, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--publish", action="store_true",
                        help="actually publish to crates.io (requires a token)")
    args = parser.parse_args()
    release(args.publish)
