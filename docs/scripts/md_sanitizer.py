"""
This script is a locally maintained MkDocs plugin that

1. Transforms the MarkDown output from TypeDoc
   because the options for customization are rather opinionated.
2. Removes the copy button from mkdocstrings HTML output's code
   blocks (for signatures only).
"""

import re
from mkdocs.structure.pages import Page
from mkdocs.structure.files import Files
from mkdocs.config.defaults import MkDocsConfig

DEFINED_IN_PATTERN = re.compile(r"^Defined in: index\.d\.ts\:\d+\n\n", re.MULTILINE)
SECTIONS_PATTERN = re.compile(
    r"#+ (Parameters|Returns|Throws|Default Value|\wet Signature)"
)
LIST_MARKER_PATTERN = re.compile("^• ", re.MULTILINE)
GROUP_API_PATTERN = re.compile(r"^## \w+\s*$", re.MULTILINE)
DIVIDER = re.compile(r"\*\*\*\n\n", re.MULTILINE)


def on_page_markdown(
    markdown: str, page: Page, config: MkDocsConfig, files: Files
) -> str:
    if "node-api" not in page.file.src_path:
        return markdown
    # change edit_uri metadata since these files are generated.
    page.edit_url = f"{config.repo_url}/{config.edit_uri}typedoc.json"
    # remove all "***" lines (equivalent to a `<hr> element`)
    markdown = DIVIDER.sub("", markdown)
    # remove all "Defined in" sections
    markdown = DEFINED_IN_PATTERN.sub("", markdown)
    # replace repeated section headers with <strong> elements
    markdown = SECTIONS_PATTERN.sub("**\\1**", markdown)
    # remove copy button from all signatures
    markdown = markdown.replace("```ts", "``` { .ts .no-copy }")
    markdown = LIST_MARKER_PATTERN.sub("- ", markdown)
    if page.file.src_path != "node-api/classes/RF24.md":
        return markdown
    # Now rearrange the grouped API of the RF24 class.
    # By default, output is grouped in alphabetical order. We want
    # 1. Basic
    # 2. Advanced
    # 3. Configuration
    result = ""
    basic_api_span = []
    advanced_api_span = []
    configure_api_span = []
    for index, match in enumerate(GROUP_API_PATTERN.finditer(markdown)):
        if index == 0:
            result += markdown[: match.start()]
        if "Basic" in match.group(0):
            basic_api_span.append(match.start())
            advanced_api_span.append(match.start())
        elif "Advanced" in match.group(0):
            advanced_api_span.append(match.start())
        elif "Configuration" in match.group(0):
            basic_api_span.append(match.start())
            configure_api_span.extend([match.start(), len(markdown)])
    result += markdown[basic_api_span[0] : basic_api_span[1]].replace(
        "## Basic", "## Basic API", 1
    )
    result += markdown[advanced_api_span[0] : advanced_api_span[1]].replace(
        "## Advanced", "## Advanced API", 1
    )
    result += markdown[configure_api_span[0] : configure_api_span[1]].replace(
        "## Configuration", "## Configuration API", 1
    )
    return result


def on_page_content(html: str, page: Page, config: MkDocsConfig, files: Files) -> str:
    if "python-api" not in page.file.src_path:
        return html
    return html.replace("doc-signature highlight", "doc-signature highlight no-copy")
