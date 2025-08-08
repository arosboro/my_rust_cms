# Multi-stage build for Rust CMS with separate backend and frontend

# Stage 1: Build the frontend with Trunk
FROM rustlang/rust:nightly-slim as frontend-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install trunk and add wasm target
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /app

# Copy workspace files and create dummy backend
COPY Cargo.toml ./
COPY frontend/ ./frontend/
# Create minimal backend stub to satisfy workspace
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
COPY backend/Cargo.toml ./backend/Cargo.toml

# Build frontend
WORKDIR /app/frontend
RUN trunk build --release

# Stage 2: Build the backend
FROM rustlang/rust:nightly-slim as backend-builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files (need all for workspace to resolve)
COPY Cargo.toml ./
COPY backend/ ./backend/
COPY frontend/Cargo.toml ./frontend/Cargo.toml
# Create minimal frontend src for workspace validation
RUN mkdir -p frontend/src && echo 'fn main() {}' > frontend/src/main.rs
COPY src/ ./src/
COPY migrations/ ./migrations/
COPY Diesel.toml ./

# Build only the backend in release mode
RUN cargo build --release --package backend

# Stage 3: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r rustcms && useradd -r -g rustcms rustcms

# Set working directory
WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/target/release/backend ./backend

# Copy frontend assets
COPY --from=frontend-builder /app/dist ./static

# Create uploads directory
RUN mkdir -p uploads && chown rustcms:rustcms uploads

# Copy any necessary static files
COPY backend/email.env.example ./email.env.example

# Change ownership of the app directory
RUN chown -R rustcms:rustcms /app

# Switch to non-root user
USER rustcms

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the backend server
CMD ["./backend"]