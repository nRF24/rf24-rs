# This script automates the release process for all of the packages in this repository.
# In order, this script does the following:
#
# 1. Bump version number in Cargo.toml manifest.
#
#    This step requires `cargo-edit` installed.
#
# 2. Updates the CHANGELOG.md
#
#    Requires `git-cliff` (see https://git-cliff.org) to be installed
#    to regenerate the change logs from git history.
#
#    NOTE: `git cliff` uses GITHUB_TOKEN env var to access GitHub's REST API for
#    fetching certain data (like PR labels and commit author's username).
#
# 3. Pushes the changes from (steps 1 and 2) to remote
#
# 4. Creates a GitHub Release and uses the section from the CHANGELOG about the new tag
#    as a release description.
#
#    Requires `gh-cli` (see https://cli.github.com) to be installed to create the release
#    and push the tag.
#
#    NOTE: This step also tags the commit from step 3.
#    When a tag is pushed to the remote, the CI builds are triggered and
#    a package are published to crates.io
#
#    NOTE: In a CI run, the GITHUB_TOKEN env var to authenticate access.
#    Locally, you can use `gh login` to interactively authenticate the user account.

const COMMON_EXCLUDES = [
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

const PkgPaths = {
    "rf24-rs": {
        include: ["crates/rf24-rs/**/*"],
        exclude: [
            "crates/rf24ble-rs/**/*",
            "bindings/**/*",
            "yarn.lock",
            ".yarnrc.yml",
            ...$COMMON_EXCLUDES,
        ],
        path: "crates/rf24-rs",
    },
    "rf24ble-rs": {
        include: ["crates/rf24ble-rs/**"],
        exclude: [
            "crates/rf24-rs/**/*",
            "bindings/**/*",
            "yarn.lock",
            ".yarnrc.yml",
            ...$COMMON_EXCLUDES,
        ],
        path: "crates/rf24ble-rs",
    },
    "rf24-py": {
        include: [],
        exclude: ["bindings/node/**/*", "yarn.lock", ".yarnrc.yml", ...$COMMON_EXCLUDES],
        path: "bindings/python",
    },
    "rf24-node": {
        include: [],
        exclude: ["bindings/python/**", ...$COMMON_EXCLUDES],
        path: "bindings/node",
    },
}

let IN_CI = $env | get --optional CI | default "false" | ($in == "true") or ($in == true)

# Bump the version per the given component name (major, minor, patch)
#
# This function also updates known occurrences of the old version spec to
# the new version spec in various places (like README.md and action.yml).
def bump-version [
    pkg: string, # The crate name to bump in respective Cargo.toml manifests
    component: string, # The version component to bump
] {
    mut args = [-p $pkg --bump $component]
    if (not $IN_CI) {
        $args = $args | append "--dry-run"
    }
    let result = (
        cargo set-version ...$args e>| lines
        | first
        | str trim
        | parse "Upgrading {pkg} from {old} to {new}"
        | first
    )
    print $"bumped ($result | get old) to ($result | get new)"
    # update the version in various places
    if (($pkg == "rf24-node") and $IN_CI)  {
        cd ($PkgPaths | get $pkg | get path)
        ^yarn version ($result | get new)
        print("Updated version in bindings/node/package.json")
    }
    $result | get new
}

# Use `git-cliff` tp generate changes.
#
# If `--unreleased` is asserted, then the `git-cliff` output will be saved to .config/ReleaseNotes.md.
# Otherwise, the generated changes will span the entire git history and be saved to CHANGELOG.md.
def gen-changes [
    pkg: string, # The crate name being bumped.
    tag: string, # The new version tag to use for unreleased changes.
    --unreleased, # only generate changes from unreleased version.
] {
    let paths = $PkgPaths | get $pkg

    mut args = [--tag $tag --config .config/cliff.toml --tag-pattern $"($pkg)/*"]
    let prompt = if $unreleased {
        let out_path = ".config/ReleaseNotes.md"
        $args = $args | append [--strip, header, --unreleased, --output, $out_path]
        {out_path: $out_path, log_prefix: "Generated"}
    } else {
        let out_path = "CHANGELOG.md"
        $args = $args | append [--output, $out_path]
        {out_path: $out_path, log_prefix: "Updated"}
    }
    if (($paths | get include | length) > 0) {
        $args = $args | append [--include-path ...($paths | get include)]
    }
    if (($paths | get exclude | length) > 0) {
        $args = $args | append [--exclude-path ...($paths | get exclude)]
    }
    ^git-cliff ...$args
    print ($prompt | format pattern "{log_prefix} {out_path}")
}

# Is the the default branch currently checked out?
def is-on-main [] {
    let branch = (
        ^git branch
        | lines
        | where {$in | str starts-with '*'}
        | first
        | str trim --left --char '*'
        | str trim
    ) == "main"
    $branch
}

# Publish a GitHub Release for the given tag.
#
# This requires a token in $env.GITHUB_TOKEN for authentication.
def gh-release [
    tag: string, # The tag corresponding to the published release.
    pkg: string, # The crate name to bump in respective Cargo.toml manifests
    ver: string, # The version number of the `$pkg`.
] {
    ^gh release create $tag --notes-file ".config/ReleaseNotes.md" --title $"($pkg) v($ver)"
}

# The main function of this script.
#
# The `component` parameter is a required CLI option:
#     nu .github/workflows/bump-n-release.nu patch
#
# The acceptable `component` values are what `cargo set-version` accepts:
#
# - manor
# - minor
# - patch
def main [
    pkg: string, # The crate name to bump in respective Cargo.toml manifests
    component: string, # The version component to bump
] {
    let ver = bump-version $pkg $component
    let tag = $"($pkg)/v($ver)"
    gen-changes $pkg $tag
    gen-changes $pkg $tag --unreleased
    let is_main = is-on-main
    if not $is_main {
        print $"(ansi yellow)Not checked out on default branch!(ansi reset)"
    }
    if $IN_CI and $is_main {
        print "Pushing metadata changes"
        git config --global user.name $"($env.GITHUB_ACTOR)"
        git config --global user.email $"($env.GITHUB_ACTOR_ID)+($env.GITHUB_ACTOR)@users.noreply.github.com"
        git add --all
        git commit -m $"build: bump version to ($tag)"
        git push
        print $"Deploying ($tag)"
        gh-release $tag $pkg $ver
    } else if $is_main {
        print $"(ansi yellow)Not deploying from local clone.(ansi reset)"
    }
}
