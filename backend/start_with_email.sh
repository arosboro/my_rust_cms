#!/bin/bash

# Kill any existing backend processes
pkill -f backend 2>/dev/null || true

# Wait a moment for processes to stop
sleep 2

# Start the backend with email environment variables
echo "🚀 Starting backend with real email service enabled..."
echo "📧 Using Gmail SMTP settings from database..."

USE_REAL_EMAIL=true \
SMTP_SERVER= \
SMTP_PORT=587 \
SMTP_USERNAME= \
SMTP_PASSWORD= \
FROM_EMAIL= \
FROM_NAME="My Rust CMS" \
BASE_URL=http://localhost:8080 \
cargo run

echo "✅ Backend started with real email service!"
