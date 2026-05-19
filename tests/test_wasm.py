import pytest

pytest.importorskip("pytest_pyodide")

from pathlib import Path

from pytest_pyodide import spawn_web_server
from pytest_pyodide.decorator import SeleniumType

ROOT = Path(__file__).resolve().parent.parent


def test_version(selenium: SeleniumType) -> None:
    dist = ROOT / "dist"
    with spawn_web_server(dist) as (host, port, _):
        url = f"http://{host}:{port}/"
        wheel = next(dist.glob("uuid7_rs-*.whl")).name
        selenium.run_async(f"""
        import micropip
        await micropip.install("{url}{wheel}")

        import uuid7_rs
        assert uuid7_rs.__version__
        """)
