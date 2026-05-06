import builtins
from typing import Any, Final, Literal, NoReturn, TypeAlias

# https://github.com/python/typeshed/blob/4849b072da0689dcf4cff1f9a9bc4404954fb2f5/stdlib/_typeshed/__init__.pyi#L52-L53
Unused: TypeAlias = object

_FieldsType: TypeAlias = tuple[int, int, int, int, int, int]

class _UUID:
    # https://github.com/python/typeshed/blob/4849b072da0689dcf4cff1f9a9bc4404954fb2f5/stdlib/uuid.pyi#L18
    int: Final[builtins.int]

    # https://github.com/python/typeshed/blob/4849b072da0689dcf4cff1f9a9bc4404954fb2f5/stdlib/uuid.pyi#L20-L56
    def __init__(
        self,
        hex: str | None = None,  # noqa: A002
        bytes: builtins.bytes | None = None,  # noqa: A002
        bytes_le: builtins.bytes | None = None,
        fields: _FieldsType | None = None,
        int: builtins.int | None = None,  # noqa: A002
    ) -> None: ...
    @property
    def bytes(self) -> builtins.bytes: ...
    @property
    def bytes_le(self) -> builtins.bytes: ...
    @property
    def clock_seq(self) -> builtins.int: ...
    @property
    def clock_seq_hi_variant(self) -> builtins.int: ...
    @property
    def clock_seq_low(self) -> builtins.int: ...
    @property
    def fields(self) -> _FieldsType: ...
    @property
    def hex(self) -> str: ...
    @property
    def node(self) -> builtins.int: ...
    @property
    def time(self) -> builtins.int: ...
    @property
    def time_hi_version(self) -> builtins.int: ...
    @property
    def time_low(self) -> builtins.int: ...
    @property
    def time_mid(self) -> builtins.int: ...
    @property
    def urn(self) -> str: ...
    # https://github.com/python/typeshed/blob/4849b072da0689dcf4cff1f9a9bc4404954fb2f5/stdlib/uuid.pyi#L61-L68
    def __int__(self) -> builtins.int: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: _UUID) -> bool: ...
    def __le__(self, other: _UUID) -> bool: ...
    def __gt__(self, other: _UUID) -> bool: ...
    def __ge__(self, other: _UUID) -> bool: ...
    def __hash__(self) -> builtins.int: ...
    def __setattr__(self, name: Unused, value: Unused) -> NoReturn: ...
    # # #
    def __copy__(self) -> _UUID: ...
    def __deepcopy__(self, memo: dict[Any, Any], /) -> _UUID: ...

# https://github.com/python/typeshed/blob/4849b072da0689dcf4cff1f9a9bc4404954fb2f5/stdlib/uuid.pyi#L92-L94
_NIL: Final[_UUID]
_MAX: Final[_UUID]

# https://www.rfc-editor.org/rfc/rfc9562#section-5.7
#
#  0                   1                   2                   3
#  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
# +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
# |                           unix_ts_ms                          |
# +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
# |          unix_ts_ms           |  ver  |       rand_a          |
# +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
# |var|                        rand_b                             |
# +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
# |                            rand_b                             |
# +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
def _uuid7(
    timestamp: int | None = None,
    nanos: int | None = None,
    mode: Literal["fast", "secure"] | None = "fast",
) -> _UUID:
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

            mode="fast" seeds the internal generator state from the OS
            randomness source once and then derives the random and counter
            bits from the internal PRNG state in the hot path.

            mode="secure" seeds a cryptographically secure PRNG (ChaCha12)
            from the OS randomness source once and then derives the random
            and counter bits from that PRNG state in the hot path.

    Returns:
        A UUIDv7 object.

    """

def _uuid7_int(
    timestamp: int | None = None,
    nanos: int | None = None,
    mode: Literal["fast", "secure"] | None = "fast",
) -> builtins.int:
    ...

def _reseed_rng() -> None: ...

_VERSION: Final[str]
