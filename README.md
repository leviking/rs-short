# rs-short: A Minimal URL-Shortening Service

`rs-short` is a minimalistic URL shortening service built in Rust using the Warp framework.

## Features:

- URL shortening and redirection.
- Simple in-memory database (can be easily replaced with a persistent one).
- Efficient and concurrent request handling.

## Prerequisites

- Rust: To install Rust, follow the guide [here](https://rustup.rs/).

## Getting Started

1. **Clone the Repository**:

    ```bash
    git clone https://github.com/leviking/rs-short.git
    cd rs-short
    ```

2. **Build and Run**:

    ```bash
    cargo run
    ```

    This will start the server on `localhost:3000`.

3. **Test the Service**:

    - **Add a URL**:

      ```bash
      curl -X POST -d "https://www.wikipedia.com/" http://localhost:3000
      ```

      This will return a shortened key. For example, `abcd`.

    - **Access the Shortened URL**:

      Open your browser and navigate to `http://localhost:3000/abcd`. This should redirect you to `https://www.wikipedia.com/`.

## Implementation Details

- Uses Warp for web routing and server functionalities.
- Uses an in-memory `HashMap` guarded by a `Mutex` for thread-safe concurrent access.

## Contributing

Feel free to create issues for suggestions, bugs, or improvements. Pull requests are always welcome.
