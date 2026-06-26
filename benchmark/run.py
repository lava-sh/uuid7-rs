import platform
import time
import uuid
from collections.abc import Callable
from importlib.metadata import (
    PackageNotFoundError,
    version as get_lib_version,
)
from pathlib import Path
from typing import TypeAlias

import altair as alt
import cpuinfo
import fastuuid
import lastuuid
import polars as pl
import uuid6
import uuid7 as uuid7gen
import uuid7_rs
import uuid_utils
import uuid_utils.compat as uuid_utils_compat
import uuidv7
from uuid_v7.base import uuid7 as uuid_v7_uuid7

N = 500_000

FILE_PATH = Path(__file__).resolve().parent
BenchmarkCase: TypeAlias = tuple[Callable, str]


def _benchmark(func: Callable, count: int) -> float:
    start = time.perf_counter()
    for _ in range(count):
        func()
    end = time.perf_counter()
    return end - start


def benchmark(
    cases: dict[str, BenchmarkCase],
    count: int,
) -> dict[str, float]:
    return {
        name: _benchmark(func, count)
        for name, (func, _) in cases.items()
    }  # fmt: skip


def get_label(name: str, package_name: str) -> str:
    try:
        version = get_lib_version(package_name)
    except PackageNotFoundError:
        version = "stdlib"
    return f"{name}\n{version}"


def plot_benchmark(
    cases: dict[str, BenchmarkCase],
    results: dict[str, float],
    save_path: Path,
    scenario: str,
) -> None:
    df = pl.DataFrame({
        "implementation": list(results.keys()),
        "exec_time": list(results.values()),
    }).sort("exec_time")

    df = df.with_columns(
        (pl.col("exec_time") / pl.col("exec_time").min()).alias("slowdown"),
    )

    df = df.with_columns(
        pl.Series(
            "implementation_label",
            [get_label(name, cases[name][1]) for name in df["implementation"]],
        ),
    )

    chart = (
        alt
        .Chart(df)
        .mark_bar(cornerRadiusTopLeft=6, cornerRadiusTopRight=6)
        .encode(
            x=alt.X(
                "implementation_label:N",
                sort=None,
                title="Implementation",
                axis=alt.Axis(
                    labelAngle=0,
                    labelExpr="split(datum.label, '\\n')",
                    labelLineHeight=14,
                ),
            ),
            y=alt.Y(
                "exec_time:Q",
                title="Execution Time (seconds, lower=better)",
                scale=alt.Scale(domain=(0, df["exec_time"].max() * 1.04)),
                axis=alt.Axis(grid=False),
            ),
            color=alt.Color(
                "implementation:N",
                legend=None,
                scale=alt.Scale(scheme="dark2"),
            ),
            tooltip=[
                alt.Tooltip("implementation:N", title=""),
                alt.Tooltip("exec_time:Q", title="Execution Time (s)", format=".4f"),
                alt.Tooltip("slowdown:Q", title="Slowdown", format=".2f"),
            ],
        )
    )

    text = (
        chart
        .mark_text(
            align="center",
            baseline="bottom",
            dy=-2,
            fontSize=9,
            fontWeight="bold",
        )
        .transform_calculate(
            label='format(datum.exec_time, ".4f") + '
            '"s (x" + format(datum.slowdown, ".2f") + ")"',
        )
        .encode(text="label:N")
    )

    cpu_info = cpuinfo.get_cpu_info()
    cpu_brand = cpu_info.get("brand_raw", "Unknown")
    py_version = f"{platform.python_version()} ({platform.system()} {platform.release()})"

    (chart + text).properties(
        width=800,
        height=600,
        title={
            "text": f"UUID v7 benchmark ({scenario})",
            "subtitle": f"Python: {py_version} | CPU: {cpu_brand}",
        },
    ).save(save_path)


def run(run_count: int) -> None:
    cases = {
        "uuid": (uuid.uuid7, "uuid"),
        "fastuuid7": (uuidv7.uuid7, "fastuuid7"),
        "uuid7_rs": (uuid7_rs.uuid7, "uuid7_rs"),
        "uuid_utils": (uuid_utils.uuid7, "uuid-utils"),
        "fastuuid": (fastuuid.uuid7, "fastuuid"),
        "uuid_v7": (uuid_v7_uuid7, "uuid-v7"),
        "uuid6": (uuid6.uuid7, "uuid6"),
        "lastuuid": (lastuuid.uuid7, "lastuuid"),
        "UUIDv7gen": (uuid7gen.UUIDv7().generate, "UUIDv7gen"),
    }

    compact_cases = {
        "uuid7_rs.compat": (uuid7_rs.compat.uuid7, "uuid7_rs"),
        "uuid_utils.compat": (uuid_utils_compat.uuid7, "uuid-utils"),
        "uuid_v7": (uuid_v7_uuid7, "uuid-v7"),
        "uuid6": (uuid6.uuid7, "uuid6"),
        "lastuuid": (lastuuid.uuid7, "lastuuid"),
        "uuid": (uuid.uuid7, "uuid"),
        "fastuuid7": (uuidv7._uuid7_python, "fastuuid7"),
    }

    plot_benchmark(
        cases,
        benchmark(cases, run_count),
        FILE_PATH / "uuid7.svg",
        "uuid7() default APIs",
    )
    plot_benchmark(
        compact_cases,
        benchmark(compact_cases, run_count),
        FILE_PATH / "uuid7-compact.svg",
        "uuid7() stdlib-compatible APIs",
    )


if __name__ == "__main__":
    run(N)
