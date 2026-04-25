__all__ = (
    "MAX",
    "NIL",
    "uuid7",
)

import sys
from typing import Literal
from uuid import UUID

from uuid7_rs._core import _uuid7

if sys.version_info >= (3, 14):
    from uuid import MAX, NIL

    def from_int(value: int) -> UUID:
        return UUID._from_int(value)  # noqa: SLF001

else:
    from uuid import SafeUUID

    MAX = UUID("ffffffff-ffff-ffff-ffff-ffffffffffff")
    NIL = UUID("00000000-0000-0000-0000-000000000000")

    def from_int(value: int) -> UUID:
        obj = object.__new__(UUID)
        object.__setattr__(obj, "int", value)  # noqa: PLC2801
        object.__setattr__(obj, "is_safe", SafeUUID.unknown)  # noqa: PLC2801
        return obj


def uuid7(
    timestamp: int | None = None,
    nanos: int | None = None,
    mode: Literal["fast", "secure"] = "fast",
) -> UUID:
    """
    Generate a UUIDv7 object.

    Args:
        timestamp:
            Unix timestamp in whole seconds.
            If omitted, the current system time is used.
        nanos:
            Optional fractional part in nanoseconds in the range
            `0..999_999_999`.
            When `timestamp` is provided, `nanos` contributes the
            sub-millisecond part of `unix_ts_ms`.
        mode:
            Changes only how the random and counter bits are produced.

            `mode="fast"` seeds the internal generator state from the OS
            randomness source once and then derives the random and counter
            bits from the internal PRNG state in the hot path.

            `mode="secure"` gets the random and counter bits from the OS
            randomness source while UUIDs are being generated and does not
            rely on the internal PRNG state for per-UUID random data.

    Returns:
        A UUIDv7 object.

    """
    uuid7_int = _uuid7(timestamp, nanos, mode).int
    return from_int(uuid7_int)
