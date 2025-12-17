# FileShare - Detailed Architecture

## System Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT BROWSER                                     │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                         index.html (Dark Theme UI)                       │   │
│  │  ┌───────────────────────────────────────────────────────────────────┐  │   │
│  │  │                      DROP ZONE COMPONENT                          │  │   │
│  │  │  • Drag & Drop Event Listeners                                    │  │   │
│  │  │  • File Input (hidden) + Select Button                            │  │   │
│  │  │  • Visual Feedback on Drag Enter/Leave                            │  │   │
│  │  └───────────────────────────────────────────────────────────────────┘  │   │
│  │  ┌───────────────────────────────────────────────────────────────────┐  │   │
│  │  │                    UPLOAD FORM COMPONENT                          │  │   │
│  │  │  • Custom Name Input (Optional)                                   │  │   │
│  │  │  • Upload Button (triggers FormData POST)                         │  │   │
│  │  │  • Progress Indicator                                             │  │   │
│  │  └───────────────────────────────────────────────────────────────────┘  │   │
│  │  ┌───────────────────────────────────────────────────────────────────┐  │   │
│  │  │                     RESULT COMPONENT                              │  │   │
│  │  │  • Download URL Display                                           │  │   │
│  │  │  • Copy Button (Clipboard API)                                    │  │   │
│  │  │  • QR Code Image (from /qr/{filename})                            │  │   │
│  │  └───────────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        │ HTTP/HTTPS
                                        ▼
┌────────────────────────────────────────────────────────────────────────────────┐
│                              NGINX REVERSE PROXY                               │
│                                                                                │
│   server {                                                                     │
│       server_name fileshare.bjk.ai;                                           │
│       location / {                                                             │
│           proxy_pass http://127.0.0.1:7012;                                   │
│           proxy_set_header Host $host;                                         │
│           proxy_set_header X-Real-IP $remote_addr;                            │
│           client_max_body_size 500M;                                           │
│       }                                                                        │
│   }                                                                            │
└────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        │ localhost:7012
                                        ▼
┌────────────────────────────────────────────────────────────────────────────────┐
│                           ACTIX-WEB SERVER (Rust)                              │
│                                Port 7012                                        │
│                                                                                │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                           ROUTE HANDLERS                                 │  │
│  │                                                                          │  │
│  │  GET  /              →  serve_index()     →  Returns index.html         │  │
│  │  POST /upload        →  upload_files()    →  Process & store files      │  │
│  │  GET  /download/{f}  →  download_file()   →  Stream file to client      │  │
│  │  GET  /qr/{filename} →  generate_qr()     →  Return QR code PNG         │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                                        │                                       │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                         CORE MODULES                                     │  │
│  │                                                                          │  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │  │
│  │  │  File Handler   │  │  ZIP Creator    │  │   QR Code Generator     │  │  │
│  │  │                 │  │                 │  │                         │  │  │
│  │  │ • Parse multipart│  │ • Collect files │  │ • qrcode-generator     │  │  │
│  │  │ • Validate input│  │ • Compress ZIP  │  │ • image crate          │  │  │
│  │  │ • Stream to disk│  │ • Write to disk │  │ • PNG encoding         │  │  │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────────┘  │  │
│  │                                                                          │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐    │  │
│  │  │                    Naming Service                                │    │  │
│  │  │  • Read/Write .zipcount for auto-increment                       │    │  │
│  │  │  • Sanitize custom names                                         │    │  │
│  │  │  • Preserve original extensions                                  │    │  │
│  │  └─────────────────────────────────────────────────────────────────┘    │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        │ Filesystem I/O
                                        ▼
┌────────────────────────────────────────────────────────────────────────────────┐
│                              FILE SYSTEM                                        │
│                                                                                │
│  ./uploads/                                                                     │
│  ├── .zipcount              # Counter file: "3" (next ID is 4)                 │
│  ├── document.pdf           # Single file upload                               │
│  ├── my-custom-name.png     # Single file with custom name                     │
│  ├── zipfile_1.zip          # Multi-file upload (auto-named)                   │
│  ├── zipfile_2.zip          # Multi-file upload (auto-named)                   │
│  └── project-files.zip      # Multi-file upload (custom-named)                 │
└────────────────────────────────────────────────────────────────────────────────┘
```

## Request Flow Diagrams

### Upload Flow (Single File)

```
┌──────────┐      ┌─────────────┐      ┌─────────────┐      ┌──────────┐
│  Browser │      │  Actix-web  │      │ File Handler│      │   Disk   │
└────┬─────┘      └──────┬──────┘      └──────┬──────┘      └────┬─────┘
     │                   │                    │                   │
     │ POST /upload      │                    │                   │
     │ (multipart/form)  │                    │                   │
     │──────────────────>│                    │                   │
     │                   │                    │                   │
     │                   │ parse_multipart()  │                   │
     │                   │───────────────────>│                   │
     │                   │                    │                   │
     │                   │                    │ determine_name()  │
     │                   │                    │──────────────────>│
     │                   │                    │                   │
     │                   │                    │ write_file()      │
     │                   │                    │──────────────────>│
     │                   │                    │                   │
     │                   │                    │<──────────────────│
     │                   │<───────────────────│                   │
     │                   │                    │                   │
     │ JSON Response     │                    │                   │
     │ {url, filename}   │                    │                   │
     │<──────────────────│                    │                   │
     │                   │                    │                   │
```

### Upload Flow (Multiple Files)

```
┌──────────┐      ┌─────────────┐      ┌─────────────┐      ┌──────────┐
│  Browser │      │  Actix-web  │      │ ZIP Handler │      │   Disk   │
└────┬─────┘      └──────┬──────┘      └──────┬──────┘      └────┬─────┘
     │                   │                    │                   │
     │ POST /upload      │                    │                   │
     │ (multiple files)  │                    │                   │
     │──────────────────>│                    │                   │
     │                   │                    │                   │
     │                   │ collect_files()    │                   │
     │                   │───────────────────>│                   │
     │                   │                    │                   │
     │                   │                    │ read .zipcount    │
     │                   │                    │──────────────────>│
     │                   │                    │<──────────────────│
     │                   │                    │                   │
     │                   │                    │ create_zip()      │
     │                   │                    │──────────────────>│
     │                   │                    │                   │
     │                   │                    │ write .zipcount+1 │
     │                   │                    │──────────────────>│
     │                   │                    │                   │
     │                   │<───────────────────│                   │
     │                   │                    │                   │
     │ JSON Response     │                    │                   │
     │ {url, filename}   │                    │                   │
     │<──────────────────│                    │                   │
```

### QR Code Generation Flow

```
┌──────────┐      ┌─────────────┐      ┌─────────────┐
│  Browser │      │  Actix-web  │      │  QR Module  │
└────┬─────┘      └──────┬──────┘      └──────┬──────┘
     │                   │                    │
     │ GET /qr/file.zip  │                    │
     │──────────────────>│                    │
     │                   │                    │
     │                   │ build_url()        │
     │                   │───────────────────>│
     │                   │                    │
     │                   │ generate_qr()      │
     │                   │───────────────────>│
     │                   │                    │
     │                   │ encode_png()       │
     │                   │───────────────────>│
     │                   │                    │
     │                   │<───────────────────│
     │                   │                    │
     │ image/png         │                    │
     │<──────────────────│                    │
```

## Data Structures

### Upload Request

```rust
// Multipart form data
struct UploadRequest {
    files: Vec<TempFile>,       // One or more files
    custom_name: Option<String>, // Optional custom filename
}
```

### Upload Response

```rust
#[derive(Serialize)]
struct UploadResponse {
    success: bool,
    filename: String,           // Final filename on disk
    url: String,                // Full download URL
    qr_url: String,             // QR code image URL
}
```

## Dependency Graph

```
fileshare
├── actix-web (4.x)          # Web framework
│   └── actix-rt              # Async runtime
├── actix-multipart (0.6)    # File upload handling
├── tokio (1.x)              # Async I/O
│   └── tokio::fs             # Async file operations
├── qrcode (0.14)            # QR code generation
├── image (0.25)             # Image encoding (PNG)
├── zip (2.x)                # ZIP archive creation
├── serde (1.x)              # JSON serialization
│   └── serde_json
└── sanitize-filename (0.5)  # Safe filename handling
```

## Security Considerations

| Threat | Mitigation |
|--------|------------|
| Path Traversal | Sanitize all filenames, reject `..` sequences |
| File Size DoS | Configure `client_max_body_size` in nginx |
| Malicious Files | Files stored as-is, no execution |
| Filename Collision | Unique naming with counter |

## Configuration

### Environment Variables (Optional)

```bash
FILESHARE_PORT=7012          # Server port
FILESHARE_HOST=0.0.0.0       # Bind address
FILESHARE_UPLOAD_DIR=./uploads  # Upload directory
FILESHARE_BASE_URL=https://fileshare.bjk.ai  # Public URL for QR codes
```

### Nginx Configuration

```nginx
server {
    listen 80;
    server_name fileshare.bjk.ai;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fileshare.bjk.ai;

    ssl_certificate /etc/letsencrypt/live/fileshare.bjk.ai/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fileshare.bjk.ai/privkey.pem;

    client_max_body_size 500M;

    location / {
        proxy_pass http://127.0.0.1:7012;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## Build & Deploy

```bash
# Development
cargo run

# Production build
cargo build --release

# Binary location
./target/release/fileshare

# Run with systemd (recommended)
# See docs/fileshare.service
```
