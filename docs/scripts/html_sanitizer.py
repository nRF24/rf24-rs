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

DEFINED_IN_PATTERN = re.compile(r"#### Defined in\n\nindex\.d\.ts\:\d+")
SECTIONS_PATTERN = re.compile(r"#### (Parameters|Returns|Throws)")
LIST_MARKER_PATTERN = re.compile("^• ", re.MULTILINE)


def on_page_markdown(
    markdown: str, page: Page, config: MkDocsConfig, files: Files
) -> str:
    if "node-api" not in page.file.src_path:
        return markdown
    # change edit_uri metadata since these files are generated.
    page.edit_url = f"{config.repo_url}/{config.edit_uri}typedoc.json"
    # remove all "Defined in" sections
    markdown = DEFINED_IN_PATTERN.sub("", markdown)
    # replace repeated section headers with <strong> elements
    markdown = SECTIONS_PATTERN.sub("**\\1**", markdown)
    # remove copy button from all signatures
    markdown = markdown.replace("```ts", "``` { .ts .no-copy }")
    markdown = LIST_MARKER_PATTERN.sub("- ", markdown)
    return markdown


def on_page_content(html: str, page: Page, config: MkDocsConfig, files: Files) -> str:
    if "python-api" not in page.file.src_path:
        return html
    return html.replace("doc-signature highlight", "doc-signature highlight no-copy")
