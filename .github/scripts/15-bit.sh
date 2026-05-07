#!/usr/bin/env bash
set -euo pipefail

PYTHON_VERSION="${PYTHON_VERSION:-3.14.4}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD_ROOT="${BUILD_ROOT:-${RUNNER_TEMP:-${HOME}/.cache}/python3-14-4_15bit}"
PY_PREFIX="${PY_PREFIX:-${BUILD_ROOT}/python314_15bit}"
VENV="${VENV:-${BUILD_ROOT}/.venv-15-bit}"
WHEEL_DIR="${WHEEL_DIR:-${BUILD_ROOT}/wheel}"
SRC="${BUILD_ROOT}/Python-${PYTHON_VERSION}"

export LD_LIBRARY_PATH="${PY_PREFIX}/lib:${LD_LIBRARY_PATH:-}"
export PATH="${PY_PREFIX}/bin:${HOME}/.local/bin:${PATH}"

mkdir -p "${BUILD_ROOT}"

if [[ ! -x "${PY_PREFIX}/bin/python3.14" ]]; then
    if command -v apt-get >/dev/null; then
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends \
            build-essential \
            ca-certificates \
            wget
    fi

    rm -rf "${SRC}" "${PY_PREFIX}"
    wget -qO- "https://www.python.org/ftp/python/${PYTHON_VERSION}/Python-${PYTHON_VERSION}.tgz" | tar xz -C "${BUILD_ROOT}"
    (
        cd "${SRC}"
        ./configure \
            --prefix="${PY_PREFIX}" \
            --enable-shared \
            --enable-big-digits=15 \
            --with-pydebug \
            --without-ensurepip \
            LDFLAGS="-Wl,-rpath,${PY_PREFIX}/lib"
        make -j"$(nproc)"
        make install
    )
    rm -rf "${SRC}"
fi

python3.14 -c "import sys; print('Python:', sys.version); print(sys.int_info); assert sys.int_info.bits_per_digit == 15"

cd "${REPO_ROOT}"
rm -rf "${VENV}" "${WHEEL_DIR}"
uv venv "${VENV}" --python "${PY_PREFIX}/bin/python3.14"
PYTHON="${VENV}/bin/python"
export PATH="${VENV}/bin:${PATH}"
export PYO3_PYTHON="${PYTHON}"

uv pip install --python "${PYTHON}" --group pytest --group maturin
maturin build --out "${WHEEL_DIR}" --release
uv pip install --python "${PYTHON}" --force-reinstall "${WHEEL_DIR}"/*.whl
pytest tests/
rm -rf "${VENV}" "${WHEEL_DIR}"
