# 🦀 My Rust CMS

A full-stack content management system built entirely in Rust, featuring a sophisticated visual page builder with nested component support. Built with the **RAYDT Stack** (Rust • Axum • Yew • Diesel • Tower) for maximum performance, safety, and developer productivity.

[![Built with RAYDT Stack](https://img.shields.io/badge/Built%20with-RAYDT%20Stack-orange.svg)](./RAYDT-STACK.md)
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/Frontend-WebAssembly-blue.svg)](https://webassembly.org/)
[![PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL-blue.svg)](https://www.postgresql.org/)
[![Page Builder](https://img.shields.io/badge/Feature-Visual%20Page%20Builder-green.svg)](#-page-builder)
[![Nested Components](https://img.shields.io/badge/Feature-Nested%20Components-blue.svg)](#-nested-component-system)

## Overview

A professional content management system built entirely in Rust, featuring an advanced visual page builder with enterprise-grade performance through WebAssembly. Built with the **RAYDT Stack** for maximum performance, safety, and developer productivity.

### Why Choose This CMS?

**🚀 Performance**
- **WebAssembly Frontend**: Near-native browser performance with minimal overhead  
- **Efficient Serialization**: Optimized data handling and transfer
- **Smart Rendering**: Targeted component updates with minimal re-rendering

**🛡️ Safety & Security**
- **Memory Safety**: Rust's ownership system eliminates entire classes of vulnerabilities
- **Type Safety**: Compile-time error prevention across the entire stack
- **Secure Architecture**: Built-in protection against common web security issues

**🎨 Developer Experience**  
- **Unified Language**: Single language across frontend, backend, and database layers
- **Rich Tooling**: Comprehensive debugging and development tools
- **Clean Architecture**: Well-organized codebase with clear separation of concerns

**🧩 Advanced Features**
- **Visual Page Builder**: Intuitive drag-and-drop interface with real-time preview
- **Nested Components**: Sophisticated component hierarchy with unlimited depth
- **Production Ready**: Battle-tested features with comprehensive error handling

## ✨ Features

### 🎨 Page Builder

Professional visual page builder with advanced component management:

- **🧩 Nested Architecture**: 
  - Recursive component nesting with unlimited depth
  - Container, TwoColumn, and ThreeColumn layout components
  - Intuitive drag-and-drop interface
  - Visual selection indicators with clear hierarchy

- **⚙️ Component Management**:
  - **Edit** (✏️): Real-time property modification with modal interface
  - **Duplicate** (📋): One-click component cloning with unique identifiers
  - **Delete** (🗑️): Safe component removal with proper cleanup
  - Live preview with instant visual feedback

- **🎯 Component Library**:
  - **Layout**: Container, TwoColumn, ThreeColumn, Hero, Card
  - **Content**: Text, Heading, Subheading, Quote, List
  - **Media**: Image, Video, Gallery with upload integration
  - **Interactive**: Button, Link, ContactForm, Newsletter
  - **Utility**: Spacer, Divider, PostsList for content organization

- **🎨 Visual Styling**:
  - Code-free visual style editor
  - Typography, color, and spacing controls
  - Border and background customization
  - Responsive design system

### 🖥️ Admin Interface

Comprehensive administrative interface with intuitive design:

- **🏠 Dashboard**: Real-time statistics and system overview
- **📝 Post Editor**: Advanced Markdown editor with live preview
- **📚 Media Library**: Secure file upload and organization system
- **👥 User Management**: Role-based access control with granular permissions
- **💬 Comment Moderation**: Advanced filtering and bulk moderation tools
- **🧭 Navigation Manager**: Dynamic menu configuration interface
- **📊 Analytics**: User engagement and content performance metrics
- **📱 Mobile Responsive**: Optimized for all device sizes
- **🌐 Public Rendering**: High-performance page rendering with nested components

### ⚡ Backend

High-performance Axum-based backend with enterprise features:

- **RESTful API**: Complete CRUD operations with type-safe endpoints
- **Authentication**: Secure session-based authentication with HMAC-signed tokens
- **Authorization**: Role-based access control with granular permissions  
- **Password Security**: bcrypt hashing with automatic salt generation (OWASP compliant)
- **Session Management**: Database-backed sessions with automatic cleanup and cryptographic signing
- **File Upload System**: Secure media handling with comprehensive validation
- **Database Migrations**: Version-controlled schema management with Diesel
- **Rate Limiting**: Built-in protection against abuse and DDoS attacks
- **CORS Configuration**: Proper cross-origin resource sharing setup
- **Health Monitoring**: System health checks and performance metrics
- **Background Processing**: Automated maintenance and cleanup tasks

### 🗃️ Database

Production-grade PostgreSQL database with comprehensive schema:

- **ACID Compliance**: Full transactional integrity and data consistency
- **Normalized Schema**: 12+ well-designed tables with proper relationships
- **Component Serialization**: Efficient JSON storage for complex page structures
- **Media Management**: Comprehensive file metadata and organization
- **Session Storage**: Secure session tracking with automatic expiration
- **Content Hierarchy**: Flexible categorization and taxonomies
- **Navigation System**: Dynamic menu structures with nested support

## 🏗️ Architecture

### Technology Stack

- **🦀 Backend**: Rust + Axum + Diesel + Tower + PostgreSQL
- **🎨 Frontend**: Rust + Yew + WebAssembly + CSS3
- **🔧 Build Tools**: Cargo workspace + Trunk + wasm-bindgen
- **🐳 Deployment**: Docker + Docker Compose with helper scripts
- **🧪 Testing**: Comprehensive test suite with WASM testing

### Project Structure

```text
my_rust_cms/
├── 📁 backend/                 # Axum API server
│   ├── src/
│   │   ├── controllers/        # Request handlers
│   │   ├── models/            # Database models
│   │   ├── middleware/        # Auth, validation, error handling
│   │   ├── services/          # Business logic
│   │   └── schema.rs          # Database schema
│   └── Cargo.toml
├── 📁 frontend/               # Yew WebAssembly app
│   ├── src/
│   │   ├── components/        # Reusable UI components
│   │   ├── pages/            # Application pages
│   │   ├── services/         # API communication
│   │   └── styles/           # CSS stylesheets
│   └── Cargo.toml
├── 📁 migrations/            # Database migrations
├── 📁 static/               # Static assets
├── 🐳 docker-compose.yml    # Development environment
├── 🐳 Dockerfile           # Production deployment
├── 🐳 DOCKER_README.md     # Docker setup and deployment guide
└── 📚 RAYDT-STACK.md       # Architecture documentation
```

## 🚀 Quick Start

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (latest stable): [Install from rustup.rs](https://rustup.rs/)
- **PostgreSQL** (13+): [Install PostgreSQL](https://www.postgresql.org/download/)
- **Trunk** (WebAssembly build tool): `cargo install trunk`
- **Diesel CLI**: `cargo install diesel_cli --features postgres`
- **Docker & Docker Compose** (optional): [Install Docker](https://docs.docker.com/get-docker/)

### Environment Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/yourusername/my_rust_cms.git
   cd my_rust_cms
   ```

2. **Set up environment variables**:

   ```bash
   cp .env.example .env  # Create .env file
   ```

      Configure your `.env` file:

   ```env
   # Database Configuration
   DATABASE_URL=postgresql://username:password@localhost:5432/my_rust_cms
   
   # Server Configuration  
   BACKEND_HOST=127.0.0.1
   BACKEND_PORT=8081
   
   # Environment
   RUST_ENV=development
   RUST_LOG=info
   
   # Security (Change in production!)
   # Authentication uses secure session-based tokens (not JWTs)
   SESSION_SECRET=your-super-secret-session-key-change-this-in-production
   
   # File Upload
   MAX_FILE_SIZE=10485760
      UPLOAD_DIR=./uploads
   ```

3. **Set up the database**:

   ```bash
   # Create database
   createdb my_rust_cms
   
   # Run migrations
   cd backend
   diesel setup
      diesel migration run
   ```

### 🚀 Running the Application

#### Option 1: Development Mode

1. **Start the Backend**:

   ```bash
   cd backend
      cargo run
   ```

   Backend will be available at `http://localhost:8081`

2. **Start the Frontend** (in a new terminal):

   ```bash
   cd frontend
      trunk serve
   ```

   Frontend will be available at `http://localhost:3000`

#### Option 2: Docker Compose (Recommended)

For complete Docker setup and deployment instructions, see **[DOCKER_README.md](./DOCKER_README.md)**.

**Quick Start with Docker:**
```bash
# Copy environment template
cp docker.env.example .env

# Start entire stack with PostgreSQL
./docker-dev.sh up

# View logs
./docker-dev.sh logs
```

Access the application at `http://localhost:8080`

### 🔐 Default Credentials

The system automatically creates a default admin user:

- **Username**: `admin`
- **Password**: `admin`

> ⚠️ **Security**: Change the default credentials immediately in production!

## 📚 API Documentation

### Authentication

The API uses **secure session-based authentication** instead of JWTs for enhanced security:

#### 🔒 Security Features
- **Session-based tokens**: Database-backed with immediate revocation capability
- **HMAC-SHA256 signing**: Cryptographically signed tokens prevent tampering  
- **OWASP-compliant passwords**: bcrypt with automatic salt generation
- **Automatic expiration**: Configurable session lifetimes with cleanup
- **Multi-session support**: Multiple concurrent sessions per user (with limits)

#### 🚀 Why Session-based over JWTs?
- **Immediate revocation**: Sessions can be invalidated instantly
- **Server-side control**: All session data controlled server-side
- **Better for CMS**: Ideal for user-facing applications with logout functionality
- **Enhanced security**: Combined database + cryptographic validation

Include the session token in the `Authorization: Bearer {token}` header for authenticated requests.

#### Public Endpoints

```http
POST /api/auth/login          # User login
GET  /api/posts               # List all posts  
GET  /api/posts/:id           # Get specific post
GET  /api/pages               # List all pages
GET  /api/pages/:id           # Get specific page
GET  /api/pages/slug/:slug    # Get page by slug
GET  /api/categories          # List categories
GET  /api/navigation          # Get navigation items
GET  /health                  # Health check
GET  /api/test               # Test endpoint
```

#### Authenticated Endpoints (Requires Login)

```http
POST /api/auth/logout                    # User logout
GET  /api/auth/me                       # Get current user
GET  /api/auth/sessions                 # Get user sessions
POST /api/auth/sessions/logout-all      # Logout all sessions
```

#### Admin-Only Endpoints (Requires Admin Role)

```http
# User Management
GET    /api/users              # List all users
POST   /api/users              # Create new user
PUT    /api/users/:id          # Update user
DELETE /api/users/:id          # Delete user

# Content Management
POST   /api/posts              # Create post
PUT    /api/posts/:id          # Update post  
DELETE /api/posts/:id          # Delete post

GET    /api/comments           # List comments
POST   /api/comments           # Create comment
PUT    /api/comments/:id       # Update comment
DELETE /api/comments/:id       # Delete comment

# Media Management
GET    /api/media              # List media files
POST   /api/media/upload       # Upload media file
DELETE /api/media/:id          # Delete media file

# Page Management
POST   /api/pages              # Create page
PUT    /api/pages/:id          # Update page
DELETE /api/pages/:id          # Delete page

# Navigation Management
POST   /api/navigation              # Create navigation item
PUT    /api/navigation/:id          # Update navigation item  
DELETE /api/navigation/:id          # Delete navigation item

# System Administration
GET    /api/stats                          # System statistics
GET    /api/sessions                       # All sessions
GET    /api/settings                       # System settings
GET    /api/templates                      # Page templates
GET    /api/components                     # Page components
GET    /api/admin/sessions                 # Session statistics
POST   /api/admin/sessions/cleanup         # Manual cleanup
GET    /api/admin/users/:id/sessions       # User sessions
POST   /api/admin/users/:id/force-logout   # Force logout user
```

### Request/Response Examples

#### Login

```bash
curl -X POST http://localhost:8081/api/auth/login \
  -H "Content-Type: application/json" \
    -d '{"username": "admin", "password": "admin"}'
```

#### Create Post (Admin)

```bash
curl -X POST http://localhost:8081/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN" \
    -d '{"title": "New Post", "content": "Post content", "category_id": 1}'
```

#### Upload Media (Admin)

```bash
curl -X POST http://localhost:8081/api/media/upload \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN" \
    -F "file=@image.jpg"
```

## 🚀 Deployment

### Production Deployment

#### Using Docker (Recommended)

For complete Docker deployment instructions, see **[DOCKER_README.md](./DOCKER_README.md)**.

**Quick Production Setup:**
```bash
# Create production environment
cp docker.env.example .env.production

# Deploy to production
./docker-prod.sh deploy

# Monitor production
./docker-prod.sh logs
```

#### Environment Configuration

Create a production `.env` file:

```env
# Production Database
DATABASE_URL=postgresql://prod_user:secure_password@db_host:5432/my_rust_cms_prod

# Production Server
BACKEND_HOST=0.0.0.0
BACKEND_PORT=8080
RUST_ENV=production
RUST_LOG=warn

# Security (Generate secure keys!)
# Authentication uses secure session-based tokens (not JWTs)
SESSION_SECRET=your-production-session-secret-at-least-32-characters-long

# File Upload (Adjust for your needs)
MAX_FILE_SIZE=52428800  # 50MB
UPLOAD_DIR=/app/uploads
```

#### Security Checklist

- [ ] Change default admin credentials
- [ ] Generate secure session secrets
- [ ] Configure proper CORS origins
- [ ] Set up HTTPS/TLS termination
- [ ] Configure firewall rules
- [ ] Set up regular database backups
- [ ] Enable monitoring and logging
- [ ] Review file upload restrictions

### Cloud Deployment

#### AWS Deployment

```bash
# Example using AWS ECS/Fargate
aws ecs create-cluster --cluster-name my-rust-cms
# Configure task definition and service
```

#### Docker Hub

```bash
# Build and push to Docker Hub
docker tag my-rust-cms:latest yourusername/my-rust-cms:latest
docker push yourusername/my-rust-cms:latest
```

## 🧪 Development

### Development Workflow

1. **Make changes to code**
2. **Run tests**:

   ```bash
   # Backend tests
   cd backend && cargo test
   
   # Frontend tests (WASM)
   cd frontend &&    wasm-pack test --headless --firefox
   ```

3. **Check code quality**:

   ```bash
   # Format code
   cargo fmt
   
   # Lint code  
   cargo clippy
   
   # Check for issues
      cargo audit
   ```

### Frontend Development Notes

When developing the frontend (Yew WebAssembly), you may encounter build issues depending on the complexity of your changes:

#### Build Mode Guidelines

- **Simple changes**: Use debug mode for faster builds:
  ```bash
  cd frontend
  trunk serve --port 8080
  ```

- **Complex changes**: Use release mode to avoid WASM compilation errors:
  ```bash
  cd frontend
  trunk serve --port 8080 --release
  ```

#### "Too Many Locals" Error

If you encounter the error `too many locals: locals exceed maximum`, this is a WASM compilation limitation in debug mode. To resolve:

1. **Always use release mode** for complex applications:
   ```bash
   trunk serve --port 8080 --release
   ```

2. **Clear build cache** if switching between modes:
   ```bash
   rm -rf target
   trunk serve --port 8080 --release
   ```

3. **Production deployments** should always use release mode for optimal performance.

> **Note**: Release builds take longer but are necessary for complex Rust/Yew applications to avoid WASM local variable limits.

### Testing

The project includes comprehensive testing:

- **Unit Tests**: Core business logic testing
- **Integration Tests**: API endpoint testing  
- **WASM Tests**: Frontend component testing
- **Network Tests**: API communication testing

Run all tests:

```bash
# Backend tests
cd backend && cargo test

# Frontend WASM tests
cd frontend && wasm-pack test --headless --firefox

# Integration tests
cd src/tests && cargo test
```

### Database Management

#### Migrations

```bash
# Create new migration
diesel migration generate migration_name

# Apply migrations
diesel migration run

# Revert migrations
diesel migration revert
```

#### Reset Database

```bash
# Drop and recreate (development only!)
diesel database reset
```

### Performance Monitoring

#### Backend Metrics

- Request/response times
- Database query performance
- Memory usage monitoring
- Session management statistics

#### Frontend Metrics

- WASM bundle size optimization
- Page load performance
- Component render times
- Network request efficiency

## 🧩 Nested Component System

Advanced nested component architecture enabling complex page layouts:

### 🏗️ Component Hierarchy
```
Page
├── Container Component
│   ├── Text Component
│   ├── Button Component
│   └── TwoColumn Component
│       ├── Column 1: Image Component
│       └── Column 2: Text + Button Components
├── ThreeColumn Component
│   ├── Column 1: Hero Component
│   ├── Column 2: Card Component
│   └── Column 3: Newsletter Component
└── Container Component
    └── Gallery Component
```

### 🎯 What It Does

- **🔄 Recursive Nesting**: Unlimited component depth with intelligent hierarchy management
- **🎨 Visual Selection**: Precise component selection with clear visual indicators
- **⚙️ Interactive Controls**: Context-sensitive edit, duplicate, and delete operations
- **🔗 Event Management**: Sophisticated event propagation with proper isolation
- **💾 State Management**: Efficient nested state updates with immutable patterns
- **🔍 Component Discovery**: Advanced algorithms for rapid component location
- **📱 Responsive Rendering**: Adaptive layouts that work across all screen sizes

### 🛠️ How It Works

- **Helper Functions**: `find_component_by_id`, `remove_nested_component`, `duplicate_nested_component`
- **Event System**: Hierarchical click handling with proper event isolation
- **State Updates**: Efficient updates with minimal re-rendering overhead
- **Serialization**: Robust JSON serialization maintaining component relationships
- **Public Rendering**: Seamless component rendering on public-facing pages

### What Works Right Now ✅

- ✅ **Visual Page Builder**: Complete drag-and-drop interface with nested component support
- ✅ **Component Architecture**: Robust hierarchy system with unlimited nesting depth
- ✅ **Component Library**: 15+ production-ready components for all content types
- ✅ **Interactive Selection**: Precise component selection with visual feedback
- ✅ **Public Rendering**: High-performance component rendering on public pages
- ✅ **Full CRUD Operations**: Complete content management across all entities
- ✅ **Authentication System**: Secure session-based authentication with role management
- ✅ **Administrative Interface**: Comprehensive dashboard with intuitive navigation
- ✅ **Media Management**: Secure file upload system with comprehensive validation
- ✅ **Database Migrations**: Version-controlled schema with automated deployment
- ✅ **Container Support**: Docker configuration for streamlined deployment
- ✅ **Session Management**: Automatic session cleanup and security management
- ✅ **Test Coverage**: Comprehensive testing suite for reliability assurance

## 🎯 What You Can Build

Perfect for building sophisticated web applications with complex content requirements:

### 🏢 Business & Corporate
- **Landing Pages**: Professional multi-column layouts with nested call-to-actions
- **Product Showcases**: Rich media galleries with detailed feature presentations
- **Corporate Blogs**: Structured content with professional typography and layout

### 🛍️ E-commerce & Marketing
- **Product Catalogs**: Complex layouts with specifications, reviews, and recommendations
- **Campaign Pages**: Hero sections with nested promotional components
- **Category Displays**: Grid layouts with filters and interactive elements

### 📰 Publishing & Media
- **Editorial Content**: Magazine-style layouts with rich media integration
- **News Portals**: Structured articles with sidebar content and related links
- **Content Hubs**: Organized information architecture with nested navigation

### 🎓 Education & Documentation
- **Course Platforms**: Interactive lesson modules with multimedia content
- **Technical Documentation**: Code examples with syntax highlighting and organization
- **Knowledge Management**: Searchable content with hierarchical organization

## 🤝 Contributing

We welcome contributions to make this CMS even better!

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes**
4. **Add tests** for new functionality
5. **Ensure all tests pass**: `cargo test && wasm-pack test --headless --firefox`
6. **Format code**: `cargo fmt`
7. **Run lints**: `cargo clippy`
8. **Commit changes**: `git commit -m "Add amazing feature"`
9. **Push to branch**: `git push origin feature/amazing-feature`
10. **Create Pull Request**

### Development Guidelines

- **Follow Rust conventions** and idiomatic patterns
- **Write comprehensive tests** for new features
- **Document public APIs** and complex logic
- **Keep commits focused** and descriptive
- **Update README** for significant changes
- **Maintain backward compatibility** when possible

### Code Style

- Use `cargo fmt` for consistent formatting
- Follow `cargo clippy` recommendations
- Write self-documenting code with clear variable names
- Add comments for complex business logic
- Keep functions focused and single-purpose

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.



## 🙏 Acknowledgments

- **🦀 Rust Community** for creating an exceptional systems programming ecosystem
- **⚡ Axum Team** for building a powerful and ergonomic async web framework
- **🎨 Yew Contributors** for pioneering WebAssembly frontend development
- **🗃️ Diesel Maintainers** for providing type-safe database interactions
- **🔒 Tower Ecosystem** for composable middleware architecture
- **🌟 RAYDT Stack** for demonstrating full-stack Rust capabilities

## 📞 Support & Community

- **🐛 Issues**: [GitHub Issues](https://github.com/yourusername/my_rust_cms/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/yourusername/my_rust_cms/discussions)
- **📚 Documentation**: [Documentation Index](./DOCUMENTATION_INDEX.md)
- **🐳 Docker Setup**: [Docker Guide](./DOCKER_README.md)
- **🏗️ RAYDT Stack**: [Learn More](./RAYDT-STACK.md)
- **🔒 Security**: [Security Guide](./SECURITY.md)

---

**🚀 Built with the [RAYDT Stack](./RAYDT-STACK.md) - Full-stack Rust that actually works**

*A CMS that respects your time, your users' experience, and your sanity*
