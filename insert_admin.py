import os
import sys
import psycopg2
from argon2 import PasswordHasher
from dotenv import load_dotenv

def main():
    # Load .env file variables
    load_dotenv()

    # Get credentials
    email = os.environ.get("WEB_SUPER_ADMIN")
    password = os.environ.get("WEB_PASSWORD")
    db_url = os.environ.get("DATABASE_URL")

    if not all([email, password, db_url]):
        print("Error: WEB_SUPER_ADMIN, WEB_PASSWORD, or DATABASE_URL is missing from .env")
        sys.exit(1)

    print(f"Extracted Admin Email: {email}")

    # If running from the VM host (outside docker), the URL needs to map to localhost instead of the 'db' container
    # You can easily override the DB URL here if needed.
    if "@db:5432" in db_url:
        print("Note: The .env DB URL uses host 'db'. If you execute this from your VM native environment, we will substitute it with 'localhost'.")
        db_url = db_url.replace("@db:5432", "@localhost:5432")

    # Initialize modern Argon2id hash compatible with your Rust Axum verify implementation
    ph = PasswordHasher()
    hashed_password = ph.hash(password)

    try:
        # Connect to Postgres
        print(f"Connecting to Postgres using: {db_url}")
        conn = psycopg2.connect(db_url)
        cursor = conn.cursor()

        # Check if the user already exists to prevent duplicate failures
        cursor.execute("SELECT id FROM admin_users WHERE email = %s;", (email,))
        if cursor.fetchone():
            print(f"Admin user '{email}' already exists safely in the database. Exiting.")
            sys.exit(0)

        # Insert securely into the database
        cursor.execute(
            "INSERT INTO admin_users (email, password) VALUES (%s, %s);",
            (email, hashed_password)
        )
        conn.commit()
        print(f"Successfully inserted admin user: {email} into postgres!")

    except psycopg2.OperationalError as e:
        print(f"\n[Connection Error] Could not connect to PostgreSQL: {e}")
        print("If you closed the port 5432 mapping in production, you must briefly re-expose it, OR run this script inside the docker network.")
    except Exception as e:
        print(f"Database error: {e}")
    finally:
        if 'conn' in locals() and conn:
            cursor.close()
            conn.close()

if __name__ == "__main__":
    main()
