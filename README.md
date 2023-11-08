# Acantha

Acantha is a full-stack, cloud-native, enterprise-level newsletter service built in Rust, integrating containerized deployment via Docker with PostgreSQL as a database and Redis for caching and session support. The service is designed to be secure, scalable, and highly customizable.

## Table of Contents

- [Pre-requisites](#pre-requisites)
    - [Windows](#windows)
    - [Linux](#linux)
    - [macOS](#macos)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Launch](#launch)
  - [Admin Interface](#admin-interface)
- [Contributing](#contributing)
- [License](#license)

## Pre-requisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/)

### Windows
  
```bash
cargo install -f cargo-binutils
rustup component add llvm-tools-preview
```

```
cargo install --version="~0.7" sqlx-cli --no-default-features --features rustls,postgres
```

### Linux

```bash
# Ubuntu 
sudo apt-get install lld clang libssl-dev postgresql-client
# Arch 
sudo pacman -S lld clang postgresql
```

```
cargo install --version="~0.7" sqlx-cli --no-default-features --features rustls,postgres
```

### MacOS

```bash
brew install michaeleisel/zld/zld
```

```
cargo install --version="~0.7" sqlx-cli --no-default-features --features rustls,postgres
```

## Configuration

The Service can be customized to suit your specific requirements by adjusting the configuration in the `base.yaml` file located within `production_rust/configurations`. Below are the key configuration options and their explanations:

### Application Configuration

- `port`: The port on which the service will listen for incoming requests (e.g., `8000`).
- `host`: The host address to bind the service to (e.g., `0.0.0.0` to listen on all available network interfaces).
- `hmac_secret`: A secret string utilized in securing the service (replace with your own secret).

### Database Configuration

- `host`: The hostname or IP address of your PostgreSQL database server (e.g., `"localhost"`).
- `port`: The port on which PostgreSQL is running (e.g., `"5432"`).
- `username`: The username to connect to the PostgreSQL database (e.g., `"postgres"`).
- `password`: The password for the PostgreSQL user (e.g., `"password"`).
- `database_name`: The name of the PostgreSQL database (e.g., `"newsletter"`).
- `require_ssl`: Set to `true` if you want to enforce SSL/TLS connections to the database; otherwise, set to `false`.

### Email Client Configuration

- `base_url`: The base URL for your email service (e.g., `"localhost"`).
- `sender_email`: The email address from which emails will be sent (e.g., `"test@gmail.com"`).
- `auth_token`: An authentication token for your email service (replace with your own token).
- `timeout_milliseconds`: The timeout duration in milliseconds for email client operations (e.g., `10000`).

### Redis Configuration

- `redis_uri`: The URI to connect to your Redis server (e.g., `"redis://127.0.0.1:6379"`).

To customize your service, open the `base.yaml` file located within `production_rust/configurations` and update the desired values according to your environment and requirements. After making changes, be sure to rebuild and restart the service for the new configuration to take effect.

Please ensure that sensitive information such as passwords, authentication tokens, and cryptographic secrets are kept secure and are not exposed in your version control system.

### Environment Variables

If you've adjusted `base.yaml` you'll need to configure environment variables to connect to the PostgreSQL database. These variables should match the database configuration you've specified in the `base.yaml` file. Follow these steps to configure the database environment variables:

1. Create or open the `.env` file in your project directory if it doesn't already exist.

2. Add the following environment variables to the `.env` file and replace the values with your database configuration:

   ```dotenv
   POSTGRES_HOST=your_database_host
   POSTGRES_PORT=your_database_port
   POSTGRES_USERNAME=your_database_username
   POSTGRES_PASSWORD=your_database_password
   POSTGRES_DB=your_database_name
   ```

3. Enable the .env file by running one of the following commands, depending on your shell:

    ```bash
    source .env
    . .env
    ```

## Usage
### Launch

1. Initialize the PostgreSQL and Redis containers:

```bash
./scripts/init_db.sh
./scripts/init_redis.sh
```

2. Change the directory to `production_rust`:

```bash
cd production_rust
```

3. Build and run via 'cargo':
```bash
cargo build
cargo run
```

### Admin interface
Access the admin interface at http://127.0.0.1:8000/login

    Default account:
        Username: admin
        Password: gloriainvigilata

## Contributing

Contributions are welcome! If you'd like to contribute to this project, please follow these steps:

1. Fork the repository on GitHub.
2. Clone your forked repository to your local machine.
3. Create a new branch for your feature or bug fix.
4. Make your changes and commit them.
5. Push your changes to your fork on GitHub.
6. Open a pull request to the main repository.

## License
This project is inspired by the book `Zero To Production in Rust` by [Luca Palmieri](https://github.com/LukeMathWalker).

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
