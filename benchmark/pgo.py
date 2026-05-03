import uuid7_rs


def main() -> None:
    for _ in range(4000):
        uuid7_rs.uuid7()

    for _ in range(3800):
        uuid7_rs._core._uuid7_int()  # noqa: SLF001

    for _ in range(3500):
        uuid7_rs.uuid7(mode="secure")

    for _ in range(3000):
        uuid7_rs.uuid7(timestamp=1, nanos=1)


if __name__ == "__main__":
    main()
