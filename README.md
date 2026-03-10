# Food Ordering API

A robust and scalable backend for a Food Ordering System, built with Rust and Actix-web. This API handles user authentication, restaurant management, menu items, and order processing with high performance and safety.

## 🚀 Tech Stack

- **Framework**: [Actix-web](https://actix.rs/) (High-performance web framework for Rust)
- **Database**: [PostgreSQL](https://www.postgresql.org/) with [SQLx](https://github.com/launchbadge/sqlx) (Async, type-safe SQL)
- **Authentication**: JWT (JSON Web Tokens)
- **Security**: Argon2 for password hashing
- **Validation**: [validator](https://github.com/Keats/validator) (Struct-level validation)
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) (Asynchronous diagnostics)

## ✨ Key Features

- **User Management**: Registration and Login with role-based access control (Customer, Restaurant Owner, Admin).
- **Menu Management**: Manage restaurant menu items including categories, availability, and pricing.
- **Order Processing**: Real-time order placement and status tracking.
- **Error Handling**: Custom, descriptive error responses.
- **Environment Driven**: Fully configurable via `.env` files.

## 🛠️ Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [PostgreSQL](https://www.postgresql.org/download/)
- `sqlx-cli` (optional, for migrations)

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Samwoker/food-ordering-api.git
   cd food-ordering-api
   ```

2. **Configure environment**:
   Create a `.env` file in the root directory:
   ```env
   DATABASE_URL=postgres://username:password@localhost:5432/food_ordering
   JWT_SECRET=your_secret_key
   JWT_EXPIRY_HOURS=24
   SERVER_HOST=127.0.0.1
   SERVER_PORT=8080
   RUST_LOG=info
   ```

3. **Setup Database**:
   ```bash
   psql -U postgres -c "CREATE DATABASE food_ordering;"
   ```

4. **Run the application**:
   ```bash
   cargo run
   ```

## 📜 License

This project is licensed under the MIT License.
