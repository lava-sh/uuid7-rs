<div align="center">

# uuid7-rs

*Fast UUID v7 generator written in Rust🦅*

</div>

[![PyPI License](https://img.shields.io/pypi/l/uuid7-rs.svg?style=flat-square)](https://pypi.org/project/uuid7-rs)
[![Monthly downloads](https://img.shields.io/pypi/dm/uuid7-rs.svg?style=flat-square)](https://pypi.org/project/uuid7-rs)
[![Github Repository size](https://img.shields.io/github/repo-size/lava-sh/uuid7-rs?style=flat-square)](https://github.com/lava-sh/uuid7-rs)
[![Python version](https://img.shields.io/pypi/pyversions/uuid7-rs.svg?style=flat-square)](https://pypi.org/project/uuid7-rs)
[![Implementation](https://img.shields.io/pypi/implementation/uuid7-rs.svg?style=flat-square)](https://pypi.org/project/uuid7-rs)

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
