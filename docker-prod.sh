#!/bin/bash

# Docker Production Helper Script for Rust CMS
# Usage: ./docker-prod.sh [command]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker is not running."
        exit 1
    fi
}

# Function to check production environment
check_prod_env() {
    if [ ! -f .env.production ]; then
        print_error "No .env.production file found. Please create one with production configurations."
        exit 1
    fi
    
    # Check for required production variables
    source .env.production
    
    if [ -z "$POSTGRES_PASSWORD" ] || [ "$POSTGRES_PASSWORD" = "your_secure_password_here" ]; then
        print_error "Please set a secure POSTGRES_PASSWORD in .env.production"
        exit 1
    fi
    
    if [ -z "$SESSION_SECRET" ] || [ "$SESSION_SECRET" = "your_session_secret_here_min_32_chars" ]; then
        print_error "Please set a secure SESSION_SECRET in .env.production"
        exit 1
    fi
}

# Function to create production environment file
create_prod_env() {
    if [ -f .env.production ]; then
        print_warning ".env.production already exists."
        read -p "Do you want to overwrite it? (y/N): " confirm
        if [[ ! $confirm =~ ^[Yy]$ ]]; then
            return
        fi
    fi
    
    print_status "Creating .env.production file..."
    cp docker.env.example .env.production
    
    # Generate secure passwords
    POSTGRES_PASS=$(openssl rand -base64 32)
    SESSION_SECRET=$(openssl rand -base64 48)
    
    # Update the production file with generated secrets
    sed -i "s/your_secure_password_here/$POSTGRES_PASS/g" .env.production
    sed -i "s/your_session_secret_here_min_32_chars/$SESSION_SECRET/g" .env.production
    
    print_success "Production environment file created with secure defaults."
    print_warning "Please review and customize .env.production before deploying."
}

# Function to build production image
build() {
    print_status "Building production Docker image..."
    check_docker
    check_prod_env
    
    # Build with production optimizations
    docker build \
        --build-arg RUST_LOG=warn \
        --target production \
        -t rustcms:production \
        -t rustcms:latest \
        .
    
    print_success "Production image built successfully!"
}

# Function to deploy to production
deploy() {
    print_status "Deploying to production..."
    check_docker
    check_prod_env
    
    # Create backup before deployment
    backup
    
    # Deploy with production compose file
    docker-compose -f docker-compose.yml -f docker-compose.prod.yml --env-file .env.production up -d
    
    print_success "Production deployment completed!"
    print_status "Application available at the configured domain/port"
}

# Function to backup database
backup() {
    print_status "Creating database backup..."
    check_docker
    
    BACKUP_NAME="backup_$(date +%Y%m%d_%H%M%S).sql"
    
    if docker-compose ps postgres | grep -q "Up"; then
        docker-compose exec postgres pg_dump -U $POSTGRES_USER $POSTGRES_DB > "backups/$BACKUP_NAME"
        print_success "Database backup created: backups/$BACKUP_NAME"
    else
        print_warning "PostgreSQL container is not running. Cannot create backup."
    fi
}

# Function to restore database
restore() {
    if [ -z "$1" ]; then
        print_error "Please specify backup file: ./docker-prod.sh restore backup_file.sql"
        exit 1
    fi
    
    if [ ! -f "backups/$1" ]; then
        print_error "Backup file not found: backups/$1"
        exit 1
    fi
    
    print_status "Restoring database from backup: $1"
    check_docker
    
    docker-compose exec -T postgres psql -U $POSTGRES_USER -d $POSTGRES_DB < "backups/$1"
    print_success "Database restored successfully!"
}

# Function to show production status
status() {
    check_docker
    print_status "Production Services Status:"
    docker-compose --env-file .env.production ps
    echo ""
    print_status "Resource Usage:"
    docker stats --no-stream
}

# Function to view production logs
logs() {
    check_docker
    if [ -n "$2" ]; then
        docker-compose --env-file .env.production logs -f "$2"
    else
        docker-compose --env-file .env.production logs -f --tail=100
    fi
}

# Function to scale services
scale() {
    if [ -z "$1" ]; then
        print_error "Please specify number of web instances: ./docker-prod.sh scale 3"
        exit 1
    fi
    
    print_status "Scaling web service to $1 instances..."
    check_docker
    check_prod_env
    
    docker-compose --env-file .env.production up -d --scale web=$1
    print_success "Scaled to $1 web instances!"
}

# Function to update production
update() {
    print_status "Updating production deployment..."
    
    # Pull latest code (assumes this is run in a deployment pipeline)
    build
    
    # Rolling update
    docker-compose --env-file .env.production up -d --no-deps web
    
    print_success "Production update completed!"
}

# Function to show help
help() {
    echo "Rust CMS Docker Production Helper"
    echo ""
    echo "Usage: ./docker-prod.sh [command]"
    echo ""
    echo "Commands:"
    echo "  create-env  Create secure .env.production file"
    echo "  build       Build production Docker image"
    echo "  deploy      Deploy to production"
    echo "  update      Update production deployment"
    echo "  backup      Create database backup"
    echo "  restore [file] Restore database from backup"
    echo "  scale [n]   Scale web service to n instances"
    echo "  status      Show production status"
    echo "  logs [service] Show production logs"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./docker-prod.sh create-env"
    echo "  ./docker-prod.sh build"
    echo "  ./docker-prod.sh deploy"
    echo "  ./docker-prod.sh backup"
    echo "  ./docker-prod.sh scale 3"
}

# Create backups directory if it doesn't exist
mkdir -p backups

# Main script logic
case "${1:-help}" in
    create-env)
        create_prod_env
        ;;
    build)
        build
        ;;
    deploy)
        deploy
        ;;
    update)
        update
        ;;
    backup)
        backup
        ;;
    restore)
        restore "$2"
        ;;
    scale)
        scale "$2"
        ;;
    status)
        status
        ;;
    logs)
        logs "$@"
        ;;
    help|*)
        help
        ;;
esac
