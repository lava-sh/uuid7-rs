<div align="center">

# uuid7-rs

*Fast UUID v7 generator written in Rust🦅*

<a href="https://pypi.org/project/uuid7-rs"><img alt="PyPI Version" src="https://shieldcn.dev/pypi/uuid7-rs.svg?variant=branded&font=geist-mono&size=xs"/></a>
<a href="https://pypi.org/project/uuid7-rs"><img alt="Monthly Downloads" src="https://shieldcn.dev/pypi/dm/uuid7-rs.svg?variant=branded&font=geist-mono&size=xs"/></a>
<a href="https://pypi.org/project/uuid7-rs"><img alt="Python Version" src="https://shieldcn.dev/pypi/python/uuid7-rs.svg?variant=branded&font=geist-mono&size=xs"/></a>

<a href="https://github.com/lava-sh/uuid7-rs/actions?query=branch%3Amain"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/ci/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&animate=pulse&mode=dark"><img alt="CI" src="https://shieldcn.dev/github/ci/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&animate=pulse&mode=light"></picture></a>
<a href="https://github.com/lava-sh/uuid7-rs/commits/main"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/last-commit/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&mode=dark"><img alt="Last Commit" src="https://shieldcn.dev/github/last-commit/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&mode=light"></picture></a>
<a href="https://github.com/lava-sh/uuid7-rs/blob/main/UNLICENSE"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/lava-sh/uuid7-rs/license.svg?variant=outline&font=geist-mono&size=xs&mode=dark"><img alt="License" src="https://shieldcn.dev/github/lava-sh/uuid7-rs/license.svg?variant=outline&font=geist-mono&size=xs&mode=light"></picture></a>

</div>

## Features

* Very fast UUID v7 generator (see [benchmarks](https://github.com/lava-sh/uuid7-rs/tree/main/benchmark))

## Installation

Using pip:

```bash
pip install uuid7-rs
```

Using uv:

```bash
uv pip install uuid7-rs
```

## Example

```python
import uuid7_rs

print(uuid7_rs.uuid7())  # 019d1ab2-cfea-71f3-ab07-0bf844ff9149
print(uuid7_rs.compat.uuid7())  # 019d1ab2-cfea-71f3-ab07-0bf98a94016c
```

## Compatibility with Python [uuid.UUID](https://docs.python.org/3/library/uuid.html)

In some cases, for example if you are using `Django`, you might
need [uuid.UUID](https://docs.python.org/3/library/uuid.html) instances to be returned
from the standard-library `uuid`, not a custom `UUID` class.

In that case you can use the `uuid7-rs.compat` which comes with a performance penalty
in comparison with the `uuid7-rs` default behaviour, but is still faster than the standard-library.

```py
import uuid7_rs.compat as uuid

# make a random UUID
print(repr(uuid.uuid7()))
# UUID('019d1ab3-f95a-79df-b868-56fe41c33af3')
```
