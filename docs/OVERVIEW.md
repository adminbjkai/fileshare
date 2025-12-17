# FileShare - High-Level Overview

## Project Summary

**FileShare** is a lightweight, efficient file-sharing web application built with Rust and Actix-web. It provides instant file uploads with shareable links and QR codes for easy mobile access.

## Core Features

| Feature | Description |
|---------|-------------|
| **Drag & Drop Upload** | Drop single or multiple files directly onto the upload zone |
| **File Browser** | Traditional file selection via system dialog |
| **Auto-Zip** | Multiple files automatically bundled into a ZIP archive |
| **Custom Naming** | Optional custom filename for uploads |
| **QR Code Generation** | Server-side QR code for instant mobile sharing |
| **Copy Link** | One-click URL copying to clipboard |

## Tech Stack

```
┌─────────────────────────────────────────────┐
│              Frontend (Static)              │
│  HTML5 + CSS3 + Vanilla JS (Dark Theme)     │
└─────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│           Actix-web Server (Rust)           │
│  • File Upload Handler                      │
│  • ZIP Compression                          │
│  • QR Code Generation                       │
│  • Static File Serving                      │
└─────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│              File Storage                   │
│           ./uploads directory               │
└─────────────────────────────────────────────┘
```

## API Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/` | Serve main UI |
| POST | `/upload` | Handle file upload |
| GET | `/download/{filename}` | Download uploaded file |
| GET | `/qr/{filename}` | Generate QR code PNG |

## Deployment Target

- **Domain**: `fileshare.bjk.ai`
- **Internal Port**: `7012`
- **Reverse Proxy**: Nginx
- **Platform**: Linux server

## Performance Characteristics

- **Memory**: Minimal footprint (~10-20MB)
- **Startup**: Sub-second cold start
- **Throughput**: Handles concurrent uploads efficiently
- **Dependencies**: Minimal, compiled to single binary

## File Naming Logic

```
Single File + No Custom Name  →  original_filename.ext
Single File + Custom Name     →  custom_name.ext
Multiple Files + No Custom    →  zipfile_{next_id}.zip
Multiple Files + Custom Name  →  custom_name.zip
```

## Directory Structure

```
fileshare/
├── Cargo.toml           # Dependencies
├── src/
│   └── main.rs          # Application code
├── static/
│   └── index.html       # Frontend UI
├── uploads/             # Uploaded files storage
│   └── .zipcount        # Counter for auto-naming
└── docs/
    ├── OVERVIEW.md      # This file
    └── ARCHITECTURE.md  # Detailed architecture
```
