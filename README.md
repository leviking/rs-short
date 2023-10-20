# rs-short - A Simple URL Shortener in Rust

`rs-short` is a lightweight URL shortener built with Warp and SQLx.

## Features

- Shorten URLs with a simple POST request.
- Retrieve and redirect to the original URL with the shortened path.
- Tracks the number of times each shortened URL is accessed.
- (Optional) Assign a user to a shortened URL.

## Setup

### Prerequisites

1. Rust (latest stable version)
2. PostgreSQL Database

### Configuration

1. Setup your PostgreSQL database and create the `urls` table:

```sql
CREATE TABLE urls (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    user_id TEXT,
    visit_count INTEGER DEFAULT 0
);
```

2. Set your database URL in an environment variable:

```
export DATABASE_URL=postgres://username:password@localhost:5432/mydatabase
```

Or use a `.env` file with:

```
DATABASE_URL=postgres://username:password@localhost:5432/mydatabase
```

### Running the Application

Navigate to the project directory and run:

```
cargo run
```

The application will start and listen on `127.0.0.1:3000`.

## Usage

### Shortening a URL

Make a POST request with JSON data:

```json
{
    "url": "https://example.com",
    "user": "optional_username"
}
```

The response will contain the shortened key.

### Accessing a URL

Navigate to `http://localhost:3000/<key>` to be redirected to the original URL. Each access will increment the visit count for that URL.
