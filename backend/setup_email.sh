#!/bin/bash

# Email configuration script for Rust CMS Backend
# This script sets up environment variables for real email sending

echo "Setting up email configuration for Rust CMS..."

# Enable real email service
export USE_REAL_EMAIL=true

# SMTP Configuration - Update these with your settings
echo "Please enter your SMTP configuration:"

read -p "SMTP Server (e.g., smtp.gmail.com): " SMTP_SERVER
export SMTP_SERVER="$SMTP_SERVER"

read -p "SMTP Port (default: 587): " SMTP_PORT
export SMTP_PORT="${SMTP_PORT:-587}"

read -p "SMTP Username (your email): " SMTP_USERNAME
export SMTP_USERNAME="$SMTP_USERNAME"

read -s -p "SMTP Password (app password for Gmail): " SMTP_PASSWORD
echo
export SMTP_PASSWORD="$SMTP_PASSWORD"

read -p "From Email (sender address): " FROM_EMAIL
export FROM_EMAIL="$FROM_EMAIL"

read -p "From Name (sender name, default: CMS System): " FROM_NAME
export FROM_NAME="${FROM_NAME:-CMS System}"

read -p "Base URL (default: http://localhost:8080): " BASE_URL
export BASE_URL="${BASE_URL:-http://localhost:8080}"

echo ""
echo "Email configuration set:"
echo "  SMTP Server: $SMTP_SERVER"
echo "  SMTP Port: $SMTP_PORT"
echo "  SMTP Username: $SMTP_USERNAME"
echo "  From Email: $FROM_EMAIL"
echo "  From Name: $FROM_NAME"
echo "  Base URL: $BASE_URL"
echo "  Real Email: $USE_REAL_EMAIL"
echo ""

echo "Starting backend with email configuration..."
cargo run
