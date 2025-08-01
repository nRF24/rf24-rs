# This script automates the release process for all of the packages in this repository.
# In order, this script does the following:
#
# 1. Bump version number in appropriate Cargo.toml manifest.
#
#    This step requires `cargo-edit` installed.
#
# 2. Updates the appropriate CHANGELOG.md
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
#
# The GITHUB_TOKEN permissions shall include:
# - read access to Pull Requests (for better CHANGELOG generation).
# - write (and inherently read) access to the repository "Contents"
#   for publishing a GitHub release and pushing metadata changes.

const COMMON_EXCLUDES = [
    '.github/**/*'
    'docs/**/*'
    'examples/**/*'
    '.config/*'
    'README.md'
    '.gitattributes'
    '.gitignore'
    '.pre-commit-config.yaml'
    'crates/README.md'
    'package.json'
    'codecov.yml'
    'Cargo.toml'
    'cspell.config.yml'
    'CHANGELOG.md'
]

const PkgPaths = {
    'rf24-rs': {
        include: ['crates/rf24-rs/**/*']
        exclude: [
            'crates/rf24ble-rs/**/*'
            'bindings/**/*'
            'yarn.lock'
            '.yarnrc.yml'
            ...$COMMON_EXCLUDES
        ]
        path: 'crates/rf24-rs'
    },
    'rf24ble-rs': {
        include: ['crates/rf24ble-rs/**']
        exclude: [
            'crates/rf24-rs/**/*'
            'bindings/**/*'
            'yarn.lock'
            '.yarnrc.yml'
            ...$COMMON_EXCLUDES
        ]
        path: 'crates/rf24ble-rs'
    },
    'rf24-py': {
        include: []
        exclude: ['bindings/node/**/*', 'yarn.lock', '.yarnrc.yml', ...$COMMON_EXCLUDES]
        path: 'bindings/python'
    },
    'rf24-node': {
        include: []
        exclude: ['bindings/python/**', ...$COMMON_EXCLUDES]
        path: 'bindings/node'
    }
}

# Is this executed in a CI run?
#
# Uses env var CI to determine the resulting boolean
export def is-in-ci [] {
    $env | get --optional CI | default 'false' | (($in == 'true') or ($in == true))
}

# Run an external command and output it's elapsed time.
#
# Not useful if you need to capture the command's output.
export def --wrapped run-cmd [...cmd: string] {
    let app = if (
        ($cmd | first) == 'cargo'
        or ($cmd | first) == 'yarn'
        or ($cmd | first) == 'git'
        or ($cmd | first) == 'gh'
    ) {
        ($cmd | first 2) | str join ' '
    } else {
        ($cmd | first)
    }
    print $"(ansi blue)\nRunning(ansi reset) ($cmd | str join ' ')"
    let elapsed = timeit {|| ^($cmd | first) ...($cmd | skip 1)}
    print $"(ansi magenta)($app) took ($elapsed)(ansi reset)"
}

# Bump the version per the given component name (major, minor, patch)
#
# This function also updates known occurrences of the old version spec to
# the new version spec in various places (like README.md and action.yml).
export def bump-version [
    pkg: string, # The crate name to bump in respective Cargo.toml manifests
    component: string, # The version component to bump
] {
    mut args = ['-p', $pkg, '--bump', $component]
    if (not (is-in-ci)) {
        $args = $args | append '--dry-run'
    }
    let result = (
        cargo 'set-version' ...$args e>| lines
        | first
        | str trim
        | parse 'Upgrading {pkg} from {old} to {new}'
        | first
    )
    print $"bumped ($result | get 'old') to ($result | get 'new')"
    # update the version in various places
    if (($pkg == 'rf24-node') and (is-in-ci))  {
        cd ($PkgPaths | get $pkg | get 'path')
        run-cmd 'yarn' 'version' ($result | get 'new')
        print 'Updated version in bindings/node/package.json'
        cd '../..'
    }
    $result | get new
}

# Use `git-cliff` tp generate changes.
#
# If `--unreleased` is asserted, then the `git-cliff` output will be saved to .config/ReleaseNotes.md.
# Otherwise, the generated changes will span the entire git history and be saved to CHANGELOG.md.
export def gen-changes [
    pkg: string, # The crate name being bumped.
    --tag (-t): string = '', # The new version tag to use for unreleased changes.
    --unreleased (-u), # only generate changes from unreleased version.
] {
    let paths = $PkgPaths | get $pkg
    let path = $paths | get path | path expand
    let config_path = '.config' | path expand

    mut args = [
        '--config' $"($config_path | path join 'cliff.toml')"
        '--tag-pattern' $"($pkg)/*"
        '--workdir' $path
        '--repository' (pwd)
    ]
    if (($tag | str length) > 0) {
        $args = $args | append ['--tag', $tag]
    }
    let prompt = if $unreleased {
        let out_path = $config_path | path join 'ReleaseNotes.md'
        $args = $args | append [
            '--strip', 'header', '--unreleased', '--output', $out_path
        ]
        {out_path: ($out_path | path relative-to (pwd)), log_prefix: 'Generated'}
    } else {
        let out_path = $path | path expand | path join 'CHANGELOG.md'
        $args = $args | append [--output $out_path]
        {out_path: ($out_path | path relative-to (pwd)), log_prefix: 'Updated'}
    }
    if (($paths | get 'include' | length) > 0) {
        $args = $args | append ['--include-path', ...($paths | get 'include')]
    }
    if (($paths | get 'exclude' | length) > 0) {
        $args = $args | append ['--exclude-path', ...($paths | get 'exclude')]
    }
    run-cmd 'git-cliff' ...$args
    print ($prompt | format pattern '{log_prefix} {out_path}')
}

# Is the the default branch currently checked out?
#
# Only accurate if the default branch is named "main".
export def is-on-main [] {
    let branch = (
        ^git branch
        | lines
        | where {$in | str starts-with '*'}
        | first
        | str trim --left --char '*'
        | str trim
    ) == 'main'
    $branch
}

# The main function of this script.
#
# The `pkg` and `component` parameters are required CLI options:
#     nu .github/workflows/bump-n-release.nu rf24-rs patch
#
# The acceptable `pkg` value are defined in the Cargo.toml manifests' `[package.name]` field.
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
    gen-changes $pkg --tag $tag
    gen-changes $pkg --tag $tag --unreleased
    let is_main = is-on-main
    if not $is_main {
        print $"(ansi yellow)\nNot checked out on default branch!(ansi reset)"
    }
    if (is-in-ci) and $is_main {
        print 'Pushing metadata changes'
        run-cmd 'git' 'config' '--global' 'user.name' $"($env.GITHUB_ACTOR)"
        run-cmd 'git' 'config' '--global' 'user.email' $"($env.GITHUB_ACTOR_ID)+($env.GITHUB_ACTOR)@users.noreply.github.com"
        run-cmd 'git' 'add' '--all'
        run-cmd 'git' 'commit' '-m' $"build: bump version to ($tag)"
        run-cmd 'git' 'push'
        print $"Deploying ($tag)"
        run-cmd 'gh' 'release' 'create' $tag '--notes-file' '.config/ReleaseNotes.md' '--title' $"($pkg) v($ver)"
    } else if $is_main {
        print $"(ansi yellow)Not deploying from local clone.(ansi reset)"
    }
}
