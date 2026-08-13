<!-- rumdl-disable MD036 MD041 -->
<div align="center">

# uuid7-rs

_Fast UUID v7 generator written in Rust🦅_
<!-- rumdl-enable MD036 MD041 -->

[![PyPI version][pypi-version-badge]][pypi]
[![PyPI downloads][pypi-downloads-badge]][pypistats]
[![PyPI requires python][pypi-requires-python-badge]][pypi]

<a href="https://github.com/lava-sh/uuid7-rs/actions?query=branch%3Amain"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/ci/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&animate=pulse&mode=dark"><img alt="CI" src="https://shieldcn.dev/github/ci/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&animate=pulse&mode=light"></picture></a>
<a href="https://github.com/lava-sh/uuid7-rs/commits/main"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/last-commit/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&mode=dark"><img alt="Last Commit" src="https://shieldcn.dev/github/last-commit/lava-sh/uuid7-rs.svg?variant=outline&font=geist-mono&size=xs&mode=light"></picture></a>
<a href="https://github.com/lava-sh/uuid7-rs/blob/main/UNLICENSE"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/lava-sh/uuid7-rs/license.svg?variant=outline&font=geist-mono&size=xs&mode=dark"><img alt="License" src="https://shieldcn.dev/github/lava-sh/uuid7-rs/license.svg?variant=outline&font=geist-mono&size=xs&mode=light"></picture></a>

</div>

## Features

* Very fast UUID v7 generator (see [benchmarks](https://github.com/lava-sh/uuid7-rs/tree/main/benchmark))

## Installation

<p>
  <img
    src="https://thesvg.org/icons/python/default.svg"
    alt="Python"
    height="14"
  />
  Using <a href="https://github.com/pypa/pip">pip</a>:
</p>

```bash
pip install uuid7-rs
```

<p>
  <img
    src="https://thesvg.org/icons/uv/default.svg"
    alt="uv"
    height="14"
  />
  Using <a href="https://github.com/astral-sh/uv">uv</a>:
</p>

```bash
uv pip install uuid7-rs
```

<p>
  <img
    src="https://thesvg.org/icons/poetry/default.svg"
    alt="Poetry"
    height="14"
  />
  Using <a href="https://github.com/python-poetry/poetry">poetry</a>:
</p>

```bash
poetry add uuid7-rs
```

## Example

```python
import uuid7_rs

print(uuid7_rs.uuid7())         # 019ea835-f42a-7dd5-8fc3-9f91aab5a95e 
# or
print(uuid7_rs.compat.uuid7())  # 019ea835-f42b-7dbd-a198-1b6d65ef5eb4
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

<div align="center">

## Contributors

[![lava-sh/uuid7-rs contributors][contributors-badge]][github-contributors]

</div>

[github-contributors]: https://github.com/lava-sh/uuid7-rs/graphs/contributors

[pypi]: https://pypi.org/project/uuid7-rs
[pypistats]: https://pypistats.org/packages/uuid7-rs

[pypi-version-badge]: https://shieldcn.dev/badge/dynamic/json.svg?url=https%3A%2F%2Fpypi.org%2Fpypi%2Fuuid7-rs%2Fjson&query=%24.info.version&variant=branded&size=xs&mode=light&logo=python&label=pypi+version
[pypi-downloads-badge]: https://shieldcn.dev/badge/dynamic/json.svg?url=https%3A%2F%2Fpypistats.org%2Fapi%2Fpackages%2Fuuid7-rs%2Frecent&query=%24.data.last_month&suffix=%2Fmonth&size=xs&mode=light&logo=python&logoColor=ffffff&label=downloads&color=3775A9
[pypi-requires-python-badge]: https://shieldcn.dev/badge/dynamic/json.svg?url=https%3A%2F%2Fpypi.org%2Fpypi%2Fuuid7-rs%2Fjson&query=%24.info.requires_python&size=xs&mode=light&logo=python&logoColor=ffffff&label=requires+python&color=3775A9
[contributors-badge]: https://shieldcn.dev/contributors/lava-sh/uuid7-rs.svg?title=false&theme=slate&size=80&bots=true&titleAlign=center&mode=light&font=geist&border=false&image=https%3A%2F%2Fimages.wallpaperscraft.ru%2Fimage%2Fsingle%2Foblaka_nebo_ogni_1647475_3840x2400.jpg&overlay=0.3
