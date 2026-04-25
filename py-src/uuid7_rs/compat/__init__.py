__all__ = (
    "MAX",
    "NIL",
    "uuid7",
)

from typing import Final, Literal
from uuid import UUID, SafeUUID

from uuid7_rs._core import _uuid7

NIL: Final[UUID] = UUID("00000000-0000-0000-0000-000000000000")
MAX: Final[UUID] = UUID("ffffffff-ffff-ffff-ffff-ffffffffffff")


def from_int(int_: int) -> UUID:
    obj = object.__new__(UUID)
    object.__setattr__(obj, "int", int_)  # noqa: PLC2801
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
