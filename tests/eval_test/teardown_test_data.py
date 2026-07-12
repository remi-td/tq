"""Drops the two eval tables from EVALS_DATABASE."""

from __future__ import annotations

import os

from dotenv import load_dotenv

from setup_test_data import get_connection, drop_tables

load_dotenv()


def main() -> None:
    db = os.environ.get("EVALS_DATABASE", "").strip()
    if not db:
        raise SystemExit("EVALS_DATABASE env var is not set")

    print(f"Connecting to Teradata to tear down tables ...")
    with get_connection() as con:
        with con.cursor() as cur:
            drop_tables(cur, db)
    print("Teardown complete.")


if __name__ == "__main__":
    main()
