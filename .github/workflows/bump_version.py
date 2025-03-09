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


COMPONENTS = ["major", "minor", "patch"]
GIT_CLIFF_CONFIG = (
    Path(__file__).parent.parent.parent / ".config" / "cliff.toml"
).resolve()
RELEASE_NOTES = GIT_CLIFF_CONFIG.with_name("ReleaseNotes.md")
PACKAGES = {
    "rf24-rs": _PkgPaths(
        include=["crates/rf24-rs/**"],
        exclude=[
            ".github/**",
            "docs/**",
            "examples/python/**",
            "examples/node/**",
        ],
    ),
    "rf24-py": _PkgPaths(
        include=[
            "crates/**/*.rs",
            "binding/python/**",
            "pyproject.toml",
            "rf24_py.pyi",
        ],
        exclude=[
            ".github/**",
            "docs/**",
            "examples/rust/**",
            "examples/node/**",
        ],
    ),
    "rf24-node": _PkgPaths(
        include=["crates/**/*.rs", "bindings/node/**"],
        exclude=[
            ".github/**",
            "docs/**",
            "examples/python/**",
            "examples/rust/**",
        ],
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
    result = subprocess.run(
        ["cargo", "set-version", "-p", pkg, "--bump", bump],
        check=True,
        capture_output=True,
    )
    print(result.stderr.decode(encoding="utf-8"), flush=True)
    stdout_prefix = f"Upgrading {pkg} from "
    for line in result.stderr.splitlines():
        out = line.decode(encoding="utf-8").strip()
        if out.startswith(stdout_prefix):
            out = out.lstrip(stdout_prefix)
            old_ver, new_ver = out.split(" to ", maxsplit=1)
            break
    else:
        raise RuntimeError(f"Failed to get version change of {pkg} package")
    if pkg == "rf24-node":
        subprocess.run(
            ["yarn", "version", "--no-git-tag-version", "--new-version", new_ver],
            check=True,
            shell=True,
            cwd="bindings/node",
        )
        print("Updated version in bindings/node/**package.json")
    return old_ver, new_ver


def get_pkg_path(pkg: str) -> Path:
    """Uses ``cargo pkgid`` to get the path of the specified ``pkg``."""
    result = subprocess.run(
        ["cargo", "pkgid", "-p", pkg],
        check=True,
        capture_output=True,
    )
    pkg_binding_prefix = f"#{pkg}@"
    for line in result.stdout.splitlines():
        out = line.decode(encoding="utf-8").strip()
        if pkg_binding_prefix in out:
            pkg_path = Path(out.split(pkg_binding_prefix)[0].lstrip("path+file://"))
            break
        if "#" in out:
            pkg_path = Path(out.split("#")[0].lstrip("path+file://"))
            break
    else:
        raise RuntimeError(f"Failed to find path to package {pkg}")
    return pkg_path


def get_changelog(tag: str, pkg: str, pkg_path: Path, full: bool = False):
    """Gets the changelog for the given ``pkg``'s ``tag``.

    If ``full`` is true, then this stores the current release's changes in a temp
    (to be used with `gh release create`).

    If ``full`` is true, then this stores the complete changelog in the package's
    CHANGELOG.md.
    """
    changelog = pkg_path.joinpath("CHANGELOG.md")
    if not changelog.exists():
        changelog.write_bytes(b"")
    output = changelog
    args = [
        "git-cliff",
        "--github-repo",
        "nRF24/rf24-rs",
        "--tag-pattern",
        f"{pkg}/[0-9]+.[0-9]+.[0-9]+",
        "--tag",
        tag,
        "--config",
        str(GIT_CLIFF_CONFIG),
    ]
    paths = PACKAGES[pkg]
    for included in paths.include:
        args.extend(["--include-path", included])
    for excluded in paths.exclude:
        args.extend(["--exclude-path", excluded])
    if not full:
        args.append("--unreleased")
        output = RELEASE_NOTES
    args.extend(["--output", str(output)])
    if not full:
        args.extend(["--strip", "header"])
    subprocess.run(args, check=True)
    if full:
        print("Updated", str(output))


class Args(argparse.Namespace):
    bump: str = "patch"
    pkg: str = ""


def main() -> int:
    if environ.get("CI", "false") == "true":
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
    print("New version:", new_ver)
    pkg_path = get_pkg_path(args.pkg)
    # generate release notes and save them to a file
    get_changelog(
        tag=f"{args.pkg}/{new_ver}", pkg=args.pkg, pkg_path=pkg_path, full=False
    )
    # generate complete changelog
    get_changelog(
        tag=f"{args.pkg}/{new_ver}", pkg=args.pkg, pkg_path=pkg_path, full=True
    )

    if "GITHUB_OUTPUT" in environ:  # create an output variables for use in CI workflow
        with open(environ["GITHUB_OUTPUT"], mode="a") as gh_out:
            gh_out.write(f"release-notes={RELEASE_NOTES}\n")
            gh_out.write(f"new-version={new_ver}\n")

    return 0


if __name__ == "__main__":
    sys.exit(main())
