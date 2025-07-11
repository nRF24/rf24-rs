"""A script to increment a package's version in this project (monorepo).
This alters the version in metadata files and updates the appropriate CHANGELOG."""

import argparse
from os import environ
from pathlib import Path
import subprocess
from typing import Tuple, NamedTuple, List
import sys


class _PkgPaths(NamedTuple):
    include: List[str]
    exclude: List[str]
    path: Path


IN_CI = environ.get("CI", "false") == "true"
COMPONENTS = ["major", "minor", "patch"]
REPO_ROOT = Path(__file__).parent.parent.parent.resolve()
GIT_CLIFF_CONFIG = REPO_ROOT / ".config" / "cliff.toml"
RELEASE_NOTES = GIT_CLIFF_CONFIG.with_name("ReleaseNotes.md")
COMMON_EXCLUDES = [
    ".github/**/*",
    "docs/**/*",
    "examples/**/*",
    ".config/*",
    "README.md",
    ".gitattributes",
    ".gitignore",
    ".pre-commit-config.yaml",
    "crates/README.md",
    "package.json",
    "codecov.yml",
    "Cargo.toml",
    "cspell.config.yml",
    "CHANGELOG.md",
]
PACKAGES = {
    "rf24-rs": _PkgPaths(
        include=["crates/rf24-rs/**/*"],
        exclude=[
            "crates/rf24ble-rs/**/*",
            "bindings/**/*",
            "yarn.lock",
            ".yarnrc.yml",
            *COMMON_EXCLUDES,
        ],
        path=REPO_ROOT / "crates" / "rf24-rs",
    ),
    "rf24ble-rs": _PkgPaths(
        include=["crates/rf24ble-rs/**"],
        exclude=[
            "crates/rf24-rs/**/*",
            "bindings/**/*",
            "yarn.lock",
            ".yarnrc.yml",
            *COMMON_EXCLUDES,
        ],
        path=REPO_ROOT / "crates" / "rf24ble-rs",
    ),
    "rf24-py": _PkgPaths(
        include=[],
        exclude=["bindings/node/**/*", "yarn.lock", ".yarnrc.yml", *COMMON_EXCLUDES],
        path=REPO_ROOT / "bindings" / "python",
    ),
    "rf24-node": _PkgPaths(
        include=[],
        exclude=["bindings/python/**", *COMMON_EXCLUDES],
        path=REPO_ROOT / "bindings" / "node",
    ),
}


def ensure_main_branch():
    # get current branch
    result = subprocess.run(["git", "branch"], capture_output=True, check=True)
    for line in result.stdout.decode(encoding="utf-8").splitlines():
        if line.startswith("*"):
            branch = line.lstrip("*").strip()
            break
    else:
        raise RuntimeError("Could not determine the currently checked out branch")

    if branch != "main":
        raise RuntimeError(f"The checked out branch {branch} is not the default")


def increment_version(pkg: str, bump: str = "patch") -> Tuple[str, str]:
    """Increment the given ``pkg`` version based on specified ``bump`` component."""
    args = ["cargo", "set-version", "-p", pkg, "--bump", bump]
    if not IN_CI:
        args.append("--dry-run")
    result = subprocess.run(
        args,
        check=True,
        capture_output=True,
    )
    stdout_prefix = f"Upgrading {pkg} from "
    for line in result.stderr.splitlines():
        out = line.decode(encoding="utf-8").strip()
        if out.startswith(stdout_prefix):
            out = out.lstrip(stdout_prefix)
            old_ver, new_ver = out.split(" to ", maxsplit=1)
            break
    else:
        raise RuntimeError(f"Failed to get version change of {pkg} package")
    if pkg == "rf24-node" and IN_CI:
        subprocess.run(
            ["yarn", "version", new_ver], check=True, cwd=str(PACKAGES[pkg].path)
        )
        subprocess.run(
            ["yarn", "run", "napi", "version"], check=True, cwd=str(PACKAGES[pkg].path)
        )
        print("Updated version in bindings/node/**/package.json")
    return old_ver, new_ver


def get_changelog(tag: str, pkg: str, full: bool = False):
    """Gets the changelog for the given ``pkg``'s ``tag``.

    If ``full`` is true, then this stores the current release's changes in a temp file
    (to be used with `gh release create`).

    If ``full`` is true, then this stores the complete changelog in the package's
    CHANGELOG.md.
    """
    paths = PACKAGES[pkg]
    changelog = paths.path / "CHANGELOG.md"
    if not changelog.exists():
        changelog.write_bytes(b"")
    output = changelog
    args = [
        "git-cliff",
        "--tag-pattern",
        f"{pkg}/*",
        "--config",
        str(GIT_CLIFF_CONFIG),
    ]
    if IN_CI:
        args.extend(["--tag", tag])
    if not full:
        args.extend(["--strip", "header", "--unreleased"])
        output = RELEASE_NOTES
    args.extend(["--output", str(output)])
    if paths.include:
        args.extend(["--include-path", *paths.include])
    if paths.exclude:
        args.extend(["--exclude-path", *paths.exclude])
    subprocess.run(args, check=True, env=environ)
    print("Updated" if full else "Generated", str(output))


class Args(argparse.Namespace):
    bump: str = "patch"
    pkg: str = ""


def main() -> int:
    if IN_CI:
        ensure_main_branch()

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "pkg",
        choices=list(PACKAGES.keys()),
        help="The package name (as described in the relevant Cargo.toml) to bump",
    )
    parser.add_argument(
        "-b",
        "--bump",
        default="patch",
        choices=COMPONENTS,
        help="The version component to increment",
    )
    args = parser.parse_args(namespace=Args())

    old_ver, new_ver = increment_version(args.pkg, bump=args.bump)
    print("Current version:", old_ver)
    if IN_CI:
        print("New version:", new_ver)
    tag = f"{args.pkg}/{new_ver}"
    # generate release notes and save them to a file
    get_changelog(tag=tag, pkg=args.pkg, full=False)
    # generate complete changelog
    get_changelog(tag=tag, pkg=args.pkg, full=True)

    if "GITHUB_OUTPUT" in environ:  # create an output variables for use in CI workflow
        with open(environ["GITHUB_OUTPUT"], mode="a") as gh_out:
            gh_out.write(f"release-notes={RELEASE_NOTES}\n")
            gh_out.write(f"new-version={new_ver}\n")

    return 0


if __name__ == "__main__":
    sys.exit(main())
