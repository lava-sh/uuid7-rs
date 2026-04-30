import copy
import ctypes
import gc
import itertools
import os
import sys
import uuid
from collections.abc import Callable
from re import escape as e
from typing import Any, Literal, TypeAlias

import pytest
import uuid7_rs

Mode: TypeAlias = Literal["fast", "secure"]
V7_MAX_TIMESTAMP_MS = 0xFFFFFFFFFFFF


class UUIDObject(ctypes.Structure):
    _fields_ = [
        ("ob_refcnt", ctypes.c_ssize_t),
        ("ob_type", ctypes.c_void_p),
        ("hi", ctypes.c_uint64),
        ("lo", ctypes.c_uint64),
    ]


def _uuid_ints(values: list[uuid7_rs.UUID]) -> list[int]:
    return [value.int for value in values]


def _assert_v7(value: uuid7_rs.UUID) -> None:
    # RFC 9562 §4.2
    # https://datatracker.ietf.org/doc/html/rfc9562#section-4.2
    #
    # Version 7 = 0111 in binary (bits 76-79 of the 128-bit UUID)
    assert (value.int >> 76) & 0xF == 0x7
    # RFC 9562 §4.1
    # https://datatracker.ietf.org/doc/html/rfc9562#section-4.1
    #
    # Variant RFC 4122 = 10 in binary (bits 62-63)
    assert (value.int >> 62) & 0x3 == 0x2


def _assert_inc(values: list[uuid7_rs.UUID]) -> None:
    ints = _uuid_ints(values)
    assert len(set(ints)) == len(ints)
    assert all(
        left < right
        for left, right in itertools.pairwise(values)
    )  # fmt: skip
    assert all(
        left < right
        for left, right in itertools.pairwise(ints)
    )  # fmt: skip


def _assert_timestamp_non_decreasing(values: list[uuid7_rs.UUID]) -> None:
    timestamps = [value.time for value in values]
    assert all(
        left <= right
        for left, right in itertools.pairwise(timestamps)
    )  # fmt: skip


@pytest.mark.parametrize("mode", [None, "fast", "secure"])
def test_uuid7_returns_uuid(mode: Mode | None) -> None:
    uuid7 = uuid7_rs.uuid7() if mode is None else uuid7_rs.uuid7(mode=mode)
    assert isinstance(uuid7, uuid7_rs.UUID)
    _assert_v7(uuid7)


@pytest.mark.parametrize(
    ("text", "expected"),
    [
        ("00000000-0000-0000-0000-000000000000", 0),
        ("ffffffff-ffff-ffff-ffff-ffffffffffff", (1 << 128) - 1),
    ],
)
def test_uuid_constructor_accepts_canonical_text(text: str, expected: int) -> None:
    uuid7 = uuid7_rs.UUID(text)
    assert isinstance(uuid7, uuid7_rs.UUID)
    assert str(uuid7) == text
    assert uuid7.int == expected


def test_uuid_nil_max() -> None:
    assert uuid7_rs.UUID("00000000-0000-0000-0000-000000000000") == uuid7_rs.NIL
    assert uuid7_rs.UUID("ffffffff-ffff-ffff-ffff-ffffffffffff") == uuid7_rs.MAX


@pytest.mark.parametrize(
    "c",
    [
        lambda: uuid7_rs.UUID(
            bytes=b"\x00\x11\x22\x33\x44\x55\x66\x77\x88\x99\xaa\xbb\xcc\xdd\xee\xff",
        ),
        lambda: uuid7_rs.UUID(
            bytes_le=b"\x33\x22\x11\x00\x55\x44\x77\x66\x88\x99\xaa\xbb\xcc\xdd\xee\xff",
        ),
        lambda: uuid7_rs.UUID(
            fields=(0x00112233, 0x4455, 0x6677, 0x88, 0x99, 0xAABBCCDDEEFF),
        ),
        lambda: uuid7_rs.UUID(int=0x00112233445566778899AABBCCDDEEFF),
    ],
)
def test_uuid_constructor(c: Callable[[], uuid7_rs.UUID]) -> None:
    assert str(c()) == "00112233-4455-6677-8899-aabbccddeeff"


@pytest.mark.parametrize(
    ("c", "err_type", "err_msg"),
    [
        (
            lambda: uuid7_rs.UUID(int=1 << 200),
            ValueError,
            "int is out of range",
        ),
        (
            lambda: uuid7_rs.UUID(uuid7_rs.uuid7()),  # ty: ignore[invalid-argument-type]
            TypeError,
            e("UUID() argument must be a str"),
        ),
        (
            lambda: uuid7_rs.UUID(),
            TypeError,
            "one of the hex, bytes, bytes_le, fields, or int",
        ),
        (
            lambda: uuid7_rs.UUID(
                "00000000-0000-0000-0000-000000000000",
                int=0,
            ),
            TypeError,
            "one of the hex, bytes, bytes_le, fields, or int",
        ),
        (
            lambda: uuid7_rs.UUID(int=-1),
            ValueError,
            e("int is out of range (need a 128-bit value)"),
        ),
        (
            lambda: uuid7_rs.UUID(
                "00000000-0000-0000-0000-000000000000",
                b"\x00" * 16,
            ),
            TypeError,
            e("UUID() takes at most 1 positional argument"),
        ),
        (
            lambda: uuid7_rs.UUID(bytes="bad"),  # ty: ignore[invalid-argument-type]
            ValueError,
            "bytes is not a 16-char string",
        ),
        (
            lambda: uuid7_rs.UUID(bytes_le="bad"),  # ty: ignore[invalid-argument-type]
            ValueError,
            "bytes_le is not a 16-char string",
        ),
        (
            lambda: uuid7_rs.UUID(bytes=bytearray(b"0" * 16)),  # ty: ignore[invalid-argument-type]
            TypeError,
            "bytes is not a 16-char bytes object",
        ),
        (
            lambda: uuid7_rs.UUID(bytes_le=bytearray(b"0" * 16)),  # ty: ignore[invalid-argument-type]
            TypeError,
            "bytes_le is not a 16-char bytes object",
        ),
    ],
)
def test_uuid_constructor_errors(
    c: Callable[[], Any],
    err_type: type[Exception],
    err_msg: str,
) -> None:
    with pytest.raises(err_type, match=err_msg):
        c()


def test_uuid7_string_and_repr_shape() -> None:
    # Verify canonical UUID string format: 8-4-4-4-12 hex digits
    # with exactly 4 hyphens (RFC 9562 §4 text representation).
    #
    # https://datatracker.ietf.org/doc/html/rfc9562#section-4
    uuid7 = uuid7_rs.uuid7()
    text = str(uuid7)
    assert len(text) == 36
    assert text.count("-") == 4
    assert repr(uuid7) == f"UUID('{text}')"


def test_uuid7_hex() -> None:
    uuid7 = uuid7_rs.uuid7()
    assert len(uuid7.hex) == 32
    assert uuid7.int == int(uuid7.hex, 16)
    assert int(uuid7) == uuid7.int


def test_uuid7_with_stdlib_uuid() -> None:
    uuid7 = uuid7_rs.uuid7()
    stdlib_uuid = uuid.UUID(int=uuid7.int)

    assert uuid7.bytes == stdlib_uuid.bytes
    assert uuid7.bytes_le == stdlib_uuid.bytes_le
    assert uuid7.fields == stdlib_uuid.fields
    assert uuid7.time_low == stdlib_uuid.time_low
    assert uuid7.time_mid == stdlib_uuid.time_mid
    assert uuid7.time_hi_version == stdlib_uuid.time_hi_version
    assert uuid7.clock_seq_hi_variant == stdlib_uuid.clock_seq_hi_variant
    assert uuid7.clock_seq_low == stdlib_uuid.clock_seq_low
    assert uuid7.clock_seq == stdlib_uuid.clock_seq
    assert uuid7.node == stdlib_uuid.node

    if sys.version_info >= (3, 14):
        assert uuid7.time == stdlib_uuid.time
    else:
        assert uuid7.time == (stdlib_uuid.int >> 80)

    assert uuid7.urn == stdlib_uuid.urn


def test_uuid7_copy_and_deepcopy() -> None:
    uuid7 = uuid7_rs.uuid7()

    assert copy.copy(uuid7) is uuid7
    assert copy.deepcopy(uuid7) is uuid7


def test_uuid7_sets_expected_version_and_variant_bits() -> None:
    for _ in range(128):
        _assert_v7(uuid7_rs.uuid7())


def test_uuid7_consecutive_values_change_more_than_the_last_bit() -> None:
    f = uuid7_rs.uuid7()
    s = uuid7_rs.uuid7()

    assert f != s
    assert (f.int & ((1 << 62) - 1)) != (s.int & ((1 << 62) - 1))
    assert (f.int ^ s.int) > 1


@pytest.mark.parametrize(
    ("c", "size"),
    [
        (uuid7_rs.uuid7, 1024),
        (uuid7_rs.uuid7, 10_000),
        (lambda: uuid7_rs.uuid7(mode="fast"), 2048),
        (lambda: uuid7_rs.uuid7(mode="secure"), 2048),
    ],
)
def test_uuid7_batches(c: Callable[[], uuid7_rs.UUID], size: int) -> None:
    values = [c() for _ in range(size)]

    _assert_inc(values)
    _assert_timestamp_non_decreasing(values)
    assert all(((value.int >> 76) & 0xF) == 0x7 for value in values)


def test_uuid7_explicit_timestamp_batch() -> None:
    values = [uuid7_rs.uuid7(1_704_164_645, 123_000_000) for _ in range(256)]

    assert all(value.time == 1_704_164_645_123 for value in values)
    assert all(value.hex[:12] == values[0].hex[:12] for value in values)
    assert len(set(_uuid_ints(values))) == len(values)
    assert all(((value.int >> 76) & 0xF) == 0x7 for value in values)


def test_uuid7_fixed_timestamp_batch() -> None:
    values = [uuid7_rs.uuid7(1_700_000_000) for _ in range(10_000)]
    assert len(set(_uuid_ints(values))) == len(values)
    assert all(((value.int >> 76) & 0xF) == 0x7 for value in values)


@pytest.mark.parametrize(
    ("args", "expected_timestamp"),
    [
        ((1_679_665_408,), 1_679_665_408_000),
        ((1_704_164_645, 123_000_000), 1_704_164_645_123),
    ],
)
def test_uuid7_timestamp_args(
    args: tuple[int, ...],
    expected_timestamp: int,
) -> None:
    # Verify that the provided Unix-second (+ optional nanos) is
    # correctly converted to a millisecond UUID timestamp.
    # RFC 9562 §5.7: 48-bit Unix timestamp in milliseconds.
    #
    # https://datatracker.ietf.org/doc/html/rfc9562#section-5.7
    uuid7 = uuid7_rs.uuid7(args[0], args[1] if len(args) > 1 else None)
    assert uuid7.time == expected_timestamp
    _assert_v7(uuid7)


@pytest.mark.parametrize("nanos", [0, 999_999_999])
def test_uuid7_accepts_valid_nanos_bounds(nanos: int) -> None:
    # Nanosecond sub-millisecond precision: accept 0 and 999_999_999
    # (the full valid range 0..999_999_999 per RFC 9562 §5.7).
    #
    # https://datatracker.ietf.org/doc/html/rfc9562#section-5.7
    _assert_v7(uuid7_rs.uuid7(nanos=nanos))


@pytest.mark.parametrize(
    ("kwargs", "err_type", "err_msg"),
    [
        ({"nanos": 1_000_000_000}, ValueError, e("nanos must be in range 0..999999999")),
        ({"timestamp": -1}, TypeError, "timestamp must be a non-negative int or None"),
        ({"nanos": -1}, TypeError, "nanos must be a non-negative int or None"),
        ({"timestamp": "bad"}, TypeError, "timestamp must be a non-negative int or None"),
        ({"nanos": "bad"}, TypeError, "nanos must be a non-negative int or None"),
        ({"mode": 1}, TypeError, "mode must be 'fast', 'secure', or None"),
        ({"mode": "bad"}, ValueError, "mode must be 'fast' or 'secure'"),
        ({"timestamp": 281_474_976_711}, ValueError, "timestamp is too large"),
    ],
)
def test_uuid7_invalid_args(
    kwargs: dict[str, Any],
    err_type: type[Exception],
    err_msg: str,
) -> None:
    with pytest.raises(err_type, match=err_msg):
        uuid7_rs.uuid7(**kwargs)


def test_uuid_objects_cmp_and_hash() -> None:
    lower = uuid7_rs.uuid7(1_700_000_000, 1)
    higher = uuid7_rs.uuid7(1_700_000_001, 1)
    hash_ = hash(lower)

    assert lower < higher
    assert lower <= higher
    assert lower != higher
    assert higher > lower
    assert higher >= lower
    assert hash(lower) == hash_


@pytest.mark.skipif(
    sys.implementation.name != "cpython",
    reason="Relies on CPython object layout via ctypes.from_address",
)
def test_uuid_hash_never_returns_error_sentinel() -> None:
    uuid7 = uuid7_rs.uuid7()
    raw_uuid = UUIDObject.from_address(id(uuid7))

    original_hi = raw_uuid.hi
    original_lo = raw_uuid.lo

    raw_uuid.hi = 0x00000000FFFFFFFF
    raw_uuid.lo = 0xFFFFFFFFFFFFFFFF

    try:
        assert hash(uuid7) == -2
        assert {uuid7: "stored"}[uuid7] == "stored"
    finally:
        raw_uuid.hi = original_hi
        raw_uuid.lo = original_lo


@pytest.mark.parametrize(
    ("timestamp", "expected"),
    [
        (0, 0),
        (V7_MAX_TIMESTAMP_MS // 1000, None),
    ],
)
def test_uuid7_timestamp_bounds(timestamp: int, expected: int | None) -> None:
    # RFC 9562 §5.7: the 48-bit timestamp field must encode epoch (t=0)
    # and accept its maximum value 2^48-1 ms.
    #
    # https://datatracker.ietf.org/doc/html/rfc9562#section-5.7
    uuid7 = uuid7_rs.uuid7(timestamp)
    if expected is not None:
        assert uuid7.time == expected
    _assert_v7(uuid7)


@pytest.mark.skipif(
    sys.platform == "win32",
    reason="Does not run on Windows",
)
def test_reseed_is_called_when_forking() -> None:
    # After `fork()`, the child process must have a reseeded RNG so that
    # UUIDs generated in parent and child do not collide.
    read_end, write_end = os.pipe()
    uuid7_rs.uuid7()

    pid = os.fork()  # ty: ignore[unresolved-attribute]
    if pid == 0:
        os.close(read_end)
        next_uuid_child = str(uuid7_rs.uuid7())
        with os.fdopen(write_end, "w") as write_pipe:
            write_pipe.write(next_uuid_child)
        os._exit(0)

    os.close(write_end)
    next_parent_uuid = uuid7_rs.uuid7()
    os.waitpid(pid, 0)
    with os.fdopen(read_end) as read_pipe:
        uuid_from_pipe = read_pipe.read().strip()

    assert str(next_parent_uuid) != uuid_from_pipe


@pytest.mark.skipif(
    sys.implementation.name == "pypy",
    reason="sys.getsizeof() always raises TypeError on PyPy",
)
def test_uuid7_mem_size() -> None:
    uuid7 = uuid7_rs.uuid7()
    assert sys.getsizeof(uuid7) < 200


@pytest.mark.skipif(
    sys.implementation.name != "cpython",
    reason="CPython object-identity specific",
)
def test_uuid7_reuses_cached_object() -> None:
    first = uuid7_rs.uuid7()
    first_id = id(first)

    del first
    gc.collect()

    second = uuid7_rs.uuid7()
    assert id(second) == first_id
