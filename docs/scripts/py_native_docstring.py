"""
Merge docstring from native module (`rf24_py.rf24_py`)
into objects discovered in type stubs (`rf24_py.pyi`).

Without this, mkdocstrings only uses the type stubs.
The type stubs don't include any doc strings because
they are embedded in the native binary for convenience
with Python REPL's `help()`.

NOTE: This must be run from a linux env.
The native binary does not include the RF24 class implementation
which requires a Linux to compile.
"""

import ast
import platform
from typing import cast
import griffe
import importlib
import logging
from mkdocs.utils import log

LOGGER = logging.getLogger("pyo3_merge_native_docs")
LOGGER.setLevel(log.getEffectiveLevel())


def elide_signature_from_docstring(docstring: str) -> str:
    lines = docstring.splitlines()
    start = 0
    for index, line in enumerate(lines):
        # pyo3 delimits the signature with a `\n--\n`
        # pybind11 delimits the signature(s) with a blank line
        if line.startswith("--"):
            start = index + 1
            break
    return "\n".join(lines[start:])


def inject_docstring(node: ast.FunctionDef | ast.ClassDef, native_doc: str):
    """Inject a given `native_docstring` into the AST node's body.

    Tested only with ClassDef and FunctionDef AST nodes.
    """
    if native_doc.startswith(f"{node.name}("):
        native_doc = elide_signature_from_docstring(native_doc)
    docstring = ast.get_docstring(node)
    if docstring is None:
        new_node = ast.Constant(native_doc)
        ast.copy_location(new_node, node)
        wrapper_node = ast.Expr(new_node)
        ast.copy_location(wrapper_node, node)
        node.body.insert(0, wrapper_node)
    elif cast(ast.Constant, cast(ast.Expr, node.body[0]).value).value != native_doc:
        cast(ast.Constant, cast(ast.Expr, node.body[0]).value).value = (
            native_doc + docstring
        )


class NativeDocstring(griffe.Extension):
    def __init__(self):
        self.native = importlib.import_module("rf24_py")

    def on_class_node(  # type: ignore[override]
        self,
        node: ast.ClassDef | griffe.ObjectNode,
        agent: griffe.Visitor | griffe.Inspector,
    ) -> None:
        """Prepend a docstring from the native module"""
        if isinstance(node, griffe.ObjectNode):
            return  # any docstring fetched from pure python should be adequate
        try:
            native_doc: str | None = getattr(self.native, node.name).__doc__
        except AttributeError:
            print(
                "The", node.name, "class was not found! Are you running this in Linux?"
            )
            return
        if not native_doc:
            return
        # print(f"Amending docstring for rf24_py.{node.name}")
        inject_docstring(node, native_doc)

    def on_function_node(  # type: ignore[override]
        self,
        node: ast.FunctionDef | griffe.ObjectNode,
        agent: griffe.Visitor | griffe.Inspector,
    ) -> None:
        """Prepend a docstring from the native module"""
        if isinstance(node, griffe.ObjectNode):
            return  # any docstring fetched from pure python should be adequate
        func_parent = node.parent  # type: ignore[attr-defined]
        if not (
            isinstance(func_parent, ast.ClassDef) or isinstance(func_parent, ast.Module)
        ):
            return  # we're only concerned with class methods or module-scoped functions
        native_obj = None
        if isinstance(func_parent, ast.ClassDef):
            native_cls = getattr(self.native, func_parent.name, None)
            if native_cls is not None:
                native_obj = getattr(native_cls, node.name, None)
        elif isinstance(func_parent, ast.Module):
            native_obj = getattr(self.native, node.name, None)
        if native_obj is None:
            if platform.system() == "Linux":
                raise AttributeError(
                    f"'{self.native}' has no attribute '{node.parent}'"
                )
            return
        native_doc: str = native_obj.__doc__ or ""
        if node.decorator_list:
            for dec in node.decorator_list:
                if isinstance(dec, ast.Name) and dec.id == "property":
                    break
            else:
                return  # class property setters are not used for the docstring
        if node.name == "__init__":
            # remove the default docstring from pyo3 (intended for `help()`)
            native_doc = native_doc.replace(
                "Initialize self.  See help(type(self)) for accurate signature.",
                "",
            )
        if not native_doc:
            return
        # print(f"Amending docstring for rf24_py.{func_parent.name}.{node.name}")
        inject_docstring(node, native_doc)
