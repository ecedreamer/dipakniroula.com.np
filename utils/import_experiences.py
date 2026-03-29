#!/usr/bin/env python3
"""
Python equivalent of src/utils/import_experiences.rs

Reads experiences.json and imports them into the PostgreSQL `experiences` table.
Existing records are wiped and replaced (same behaviour as the Rust version).

Requirements:
    pip install psycopg2-binary python-dotenv

Usage:
    python utils/import_experiences.py
    python utils/import_experiences.py --json path/to/other.json
    python utils/import_experiences.py --db-url postgresql://user:pass@localhost:5432/dipakdb
"""

import argparse
import json
import os
import sys
from pathlib import Path

try:
    import psycopg2
    from psycopg2.extras import execute_values
except ImportError:
    sys.exit("psycopg2 not installed. Run: pip install psycopg2-binary")

try:
    from dotenv import load_dotenv
except ImportError:
    sys.exit("python-dotenv not installed. Run: pip install python-dotenv")


# ─── Helpers ───────────────────────────────────────────────────────────────

def resolve_db_url(override: str | None) -> str:
    """Load DATABASE_URL from .env (same directory as this script) or accept override."""
    # Walk up to find the .env file (mirrors Rust dotenvy behaviour)
    here = Path(__file__).resolve()
    for parent in [here.parent, here.parent.parent, here.parent.parent.parent]:
        env_file = parent / ".env"
        if env_file.exists():
            load_dotenv(env_file)
            break

    if override:
        return override

    url = os.getenv("DATABASE_URL")
    if not url:
        sys.exit("DATABASE_URL not found. Provide it via .env or --db-url")

    # Mirror the Rust behaviour: replace @db: → @localhost: for local runs
    url = url.replace("@db:", "@localhost:")
    return url


def load_json(path: str) -> list[dict]:
    """Load and validate the experiences JSON file."""
    p = Path(path)
    if not p.exists():
        sys.exit(f"JSON file not found: {p}")

    with p.open() as f:
        data = json.load(f)

    if not isinstance(data, list):
        sys.exit("JSON must be a top-level array of experience objects")

    return data


# ─── Database ───────────────────────────────────────────────────────────────

REQUIRED_FIELDS = {
    "company_name", "company_link", "your_position",
    "start_date", "order",
}

def validate(experiences: list[dict]) -> None:
    for i, exp in enumerate(experiences):
        missing = REQUIRED_FIELDS - exp.keys()
        if missing:
            sys.exit(f"Experience [{i}] is missing required fields: {missing}")


def import_experiences(conn, experiences: list[dict]) -> None:
    rows = [
        (
            exp["company_name"],
            exp["company_link"],
            exp["your_position"],
            exp["start_date"],
            exp.get("end_date"),          # nullable
            exp.get("responsibility"),    # nullable
            exp.get("skills"),            # nullable
            int(exp["order"]),
        )
        for exp in experiences
    ]

    with conn.cursor() as cur:
        print("Cleaning up existing experiences...")
        cur.execute("DELETE FROM experiences")

        print(f"Importing {len(rows)} experience(s)...")
        execute_values(
            cur,
            """
            INSERT INTO experiences
                (company_name, company_link, your_position,
                 start_date, end_date, responsibility, skills, "order")
            VALUES %s
            """,
            rows,
        )

    conn.commit()


# ─── Entry point ─────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(description="Import experiences.json into PostgreSQL")
    parser.add_argument(
        "--json",
        default="src/resume/experiences.json",
        help="Path to experiences JSON file (default: src/resume/experiences.json)",
    )
    parser.add_argument(
        "--db-url",
        default=None,
        help="PostgreSQL connection URL (overrides DATABASE_URL in .env)",
    )
    args = parser.parse_args()

    db_url = resolve_db_url(args.db_url)
    # Mask password in log output
    safe_url = db_url.split("@")[-1] if "@" in db_url else db_url
    print(f"Connecting to database: ...@{safe_url}")

    try:
        conn = psycopg2.connect(db_url)
    except psycopg2.OperationalError as e:
        sys.exit(f"Failed to connect to database: {e}")

    print(f"Loading experiences from {args.json}...")
    experiences = load_json(args.json)
    validate(experiences)

    try:
        import_experiences(conn, experiences)
    except psycopg2.Error as e:
        conn.rollback()
        sys.exit(f"Database error: {e}")
    finally:
        conn.close()

    print("Successfully imported experiences!")


if __name__ == "__main__":
    main()
