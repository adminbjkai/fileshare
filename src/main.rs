use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use futures_util::StreamExt;
use image::{ImageEncoder, Luma};
use qrcode::QrCode;
use serde::Serialize;
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const UPLOAD_DIR: &str = "./uploads";
const ZIP_COUNT_FILE: &str = "./uploads/.zipcount";
const BASE_URL: &str = "FILESHARE_BASE_URL";
const DEFAULT_BASE_URL: &str = "http://localhost:7012";

#[derive(Serialize)]
struct UploadResponse {
    success: bool,
    filename: String,
    url: String,
    qr_url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

struct FileData {
    name: String,
    data: Vec<u8>,
}

fn get_base_url() -> String {
    std::env::var(BASE_URL).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

fn get_next_zip_id() -> u64 {
    let count = fs::read_to_string(ZIP_COUNT_FILE)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let next = count + 1;
    let _ = fs::write(ZIP_COUNT_FILE, next.to_string());
    next
}

fn sanitize_name(name: &str) -> String {
    sanitize_filename::sanitize(name)
}

async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

async fn upload_files(mut payload: Multipart) -> HttpResponse {
    let mut files: Vec<FileData> = Vec::new();
    let mut custom_name: Option<String> = None;

    // Process multipart form
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    success: false,
                    error: format!("Error processing upload: {}", e),
                });
            }
        };

        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => continue,
        };
        let field_name = content_disposition.get_name().unwrap_or("");

        if field_name == "custom_name" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                if let Ok(bytes) = chunk {
                    data.extend_from_slice(&bytes);
                }
            }
            let name = String::from_utf8_lossy(&data).trim().to_string();
            if !name.is_empty() {
                custom_name = Some(sanitize_name(&name));
            }
        } else if field_name == "files" {
            let filename = content_disposition
                .get_filename()
                .map(|f| sanitize_name(f))
                .unwrap_or_else(|| format!("file_{}", uuid::Uuid::new_v4()));

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                if let Ok(bytes) = chunk {
                    data.extend_from_slice(&bytes);
                }
            }

            if !data.is_empty() {
                files.push(FileData {
                    name: filename,
                    data,
                });
            }
        }
    }

    if files.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "No files uploaded".to_string(),
        });
    }

    // Ensure upload directory exists
    if let Err(e) = fs::create_dir_all(UPLOAD_DIR) {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            success: false,
            error: format!("Failed to create upload directory: {}", e),
        });
    }

    let final_filename: String;

    if files.len() == 1 {
        // Single file
        let file = &files[0];
        let extension = PathBuf::from(&file.name)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();

        final_filename = match &custom_name {
            Some(name) => {
                if name.contains('.') {
                    name.clone()
                } else {
                    format!("{}{}", name, extension)
                }
            }
            None => file.name.clone(),
        };

        let filepath = PathBuf::from(UPLOAD_DIR).join(&final_filename);
        if let Err(e) = fs::write(&filepath, &file.data) {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to save file: {}", e),
            });
        }
    } else {
        // Multiple files - create ZIP
        final_filename = match &custom_name {
            Some(name) => {
                if name.ends_with(".zip") {
                    name.clone()
                } else {
                    format!("{}.zip", name)
                }
            }
            None => format!("zipfile_{}.zip", get_next_zip_id()),
        };

        let filepath = PathBuf::from(UPLOAD_DIR).join(&final_filename);
        let file = match fs::File::create(&filepath) {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    success: false,
                    error: format!("Failed to create zip file: {}", e),
                });
            }
        };

        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        for file_data in &files {
            if let Err(e) = zip.start_file(&file_data.name, options) {
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    success: false,
                    error: format!("Failed to add file to zip: {}", e),
                });
            }
            if let Err(e) = zip.write_all(&file_data.data) {
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    success: false,
                    error: format!("Failed to write file data to zip: {}", e),
                });
            }
        }

        if let Err(e) = zip.finish() {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to finalize zip: {}", e),
            });
        }
    }

    let base_url = get_base_url();
    let download_url = format!("{}/{}", base_url, final_filename);
    let qr_url = format!("/qr/{}", final_filename);

    HttpResponse::Ok().json(UploadResponse {
        success: true,
        filename: final_filename,
        url: download_url,
        qr_url,
    })
}

async fn download_file(req: HttpRequest) -> Result<HttpResponse> {
    let filename: String = req.match_info().query("filename").to_string();
    let sanitized = sanitize_name(&filename);
    let filepath = PathBuf::from(UPLOAD_DIR).join(&sanitized);

    if !filepath.exists() {
        return Ok(HttpResponse::NotFound().json(ErrorResponse {
            success: false,
            error: "File not found".to_string(),
        }));
    }

    let file = NamedFile::open(&filepath)?;
    Ok(file.into_response(&req))
}

async fn generate_qr(req: HttpRequest) -> HttpResponse {
    let filename: String = req.match_info().query("filename").to_string();
    let base_url = get_base_url();
    let download_url = format!("{}/{}", base_url, filename);

    // Generate QR code
    let code = match QrCode::new(download_url.as_bytes()) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: "Failed to generate QR code".to_string(),
            });
        }
    };

    let image = code.render::<Luma<u8>>().min_dimensions(200, 200).build();

    // Encode as PNG
    let mut png_data = Cursor::new(Vec::new());
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    if encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::L8,
        )
        .is_err()
    {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            success: false,
            error: "Failed to encode QR code".to_string(),
        });
    }

    HttpResponse::Ok()
        .content_type("image/png")
        .body(png_data.into_inner())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Ensure directories exist
    fs::create_dir_all(UPLOAD_DIR)?;
    fs::create_dir_all("./static")?;

    // Initialize zip counter if not exists
    if !PathBuf::from(ZIP_COUNT_FILE).exists() {
        fs::write(ZIP_COUNT_FILE, "0")?;
    }

    let host = std::env::var("FILESHARE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("FILESHARE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7012);

    println!("FileShare starting on http://{}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/upload", web::post().to(upload_files))
            .route("/qr/{filename:.*}", web::get().to(generate_qr))
            .route("/{filename:.*}", web::get().to(download_file))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
