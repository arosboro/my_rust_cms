#!/bin/bash

# Docker Development Helper Script for Rust CMS
# Usage: ./docker-dev.sh [command]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
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
        print_error "Docker is not running. Please start Docker Desktop."
        exit 1
    fi
}

# Function to check if .env file exists
check_env() {
    if [ ! -f .env ]; then
        print_warning "No .env file found. Creating one from docker.env.example..."
        cp docker.env.example .env
        print_warning "Please edit .env file with your configuration before running Docker services."
        echo "You can use: nano .env"
    fi
}

# Function to build the application
build() {
    print_status "Building Rust CMS Docker image..."
    check_docker
    check_env
    docker-compose build --no-cache
    print_success "Build completed successfully!"
}

# Function to start services
up() {
    print_status "Starting Rust CMS services..."
    check_docker
    check_env
    docker-compose up -d
    print_success "Services started successfully!"
    print_status "Application will be available at: http://localhost:8080"
    print_status "Database admin (Adminer) available at: http://localhost:8081 (use --profile dev)"
}

# Function to start services with logs
up_logs() {
    print_status "Starting Rust CMS services with logs..."
    check_docker
    check_env
    docker-compose up
}

# Function to start with development profile (includes Adminer)
up_dev() {
    print_status "Starting Rust CMS services in development mode..."
    check_docker
    check_env
    docker-compose --profile dev up -d
    print_success "Development services started successfully!"
    print_status "Application available at: http://localhost:8080"
    print_status "Database admin (Adminer) available at: http://localhost:8081"
}

# Function to stop services
down() {
    print_status "Stopping Rust CMS services..."
    check_docker
    docker-compose down
    print_success "Services stopped successfully!"
}

# Function to restart services
restart() {
    print_status "Restarting Rust CMS services..."
    down
    up
}

# Function to view logs
logs() {
    check_docker
    if [ -n "$2" ]; then
        docker-compose logs -f "$2"
    else
        docker-compose logs -f
    fi
}

# Function to run database migrations
migrate() {
    print_status "Running database migrations..."
    check_docker
    docker-compose exec web ./backend --migrate
    print_success "Migrations completed successfully!"
}

# Function to open a shell in the web container
shell() {
    print_status "Opening shell in web container..."
    check_docker
    docker-compose exec web /bin/bash
}

# Function to clean up Docker resources
clean() {
    print_status "Cleaning up Docker resources..."
    check_docker
    docker-compose down -v
    docker system prune -f
    docker volume prune -f
    print_success "Cleanup completed successfully!"
}

# Function to show status
status() {
    check_docker
    print_status "Docker Compose Services Status:"
    docker-compose ps
    echo ""
    print_status "Docker Images:"
    docker images | grep -E "(rustcms|postgres|adminer)"
    echo ""
    print_status "Docker Volumes:"
    docker volume ls | grep rustcms
}

# Function to show help
help() {
    echo "Rust CMS Docker Development Helper"
    echo ""
    echo "Usage: ./docker-dev.sh [command]"
    echo ""
    echo "Commands:"
    echo "  build       Build the Docker image"
    echo "  up          Start services in background"
    echo "  up-logs     Start services with logs in foreground"
    echo "  up-dev      Start services with development tools (Adminer)"
    echo "  down        Stop services"
    echo "  restart     Restart services"
    echo "  logs [service]  Show logs (optionally for specific service)"
    echo "  migrate     Run database migrations"
    echo "  shell       Open shell in web container"
    echo "  clean       Clean up Docker resources (WARNING: removes volumes)"
    echo "  status      Show status of services and Docker resources"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./docker-dev.sh build"
    echo "  ./docker-dev.sh up-dev"
    echo "  ./docker-dev.sh logs web"
    echo "  ./docker-dev.sh shell"
}

# Main script logic
case "${1:-help}" in
    build)
        build
        ;;
    up)
        up
        ;;
    up-logs)
        up_logs
        ;;
    up-dev)
        up_dev
        ;;
    down)
        down
        ;;
    restart)
        restart
        ;;
    logs)
        logs "$@"
        ;;
    migrate)
        migrate
        ;;
    shell)
        shell
        ;;
    clean)
        clean
        ;;
    status)
        status
        ;;
    help|*)
        help
        ;;
esac
