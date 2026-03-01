/// Web server for Options Pricing GUI
/// Provides REST API and serves static files for the web interface
mod data_fetcher;
mod visualization;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use data_fetcher::DataFetcher;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use visualization::{ChartPoint, VisualizationConfig};

#[derive(Debug, Serialize, Deserialize)]
struct TickerRequest {
    ticker: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalyzeRequest {
    ticker: String,
    expiration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct FetchDataRequest {
    ticker: String,
}

#[derive(Debug, Serialize)]
struct TickerInfo {
    ticker: String,
    current_price: f64,
    expirations: Vec<ExpirationDate>,
}

#[derive(Debug, Serialize)]
struct ExpirationDate {
    timestamp: i64,
    date: String,
    days_to_expiry: i64,
}

#[derive(Debug, Serialize)]
struct AnalysisResponse {
    session_id: String,
    ticker: String,
    current_price: f64,
    expiration: String,
    days_to_expiry: i64,
    time_to_expiry: f64,
    risk_free_rate: f64,
    hist_volatility: f64,
    dividend_yield: f64,
    charts: Vec<String>,
    num_contracts: usize,
    atm_strike: f64,
    atm_price: f64,
    atm_iv: f64,
}

struct AppState {
    fetcher: Arc<RwLock<DataFetcher>>,
}

async fn get_ticker_info(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let ticker = path.into_inner().to_uppercase();
    let fetcher = data.fetcher.read().await;

    // Try to load from existing CSV files first
    match fetcher.load_from_csv(&ticker) {
        Ok((_, current_price, bs_params)) => {
            // Create a single expiration date from the CSV
            let expirations = vec![ExpirationDate {
                timestamp: Utc::now().timestamp() + (bs_params.days_to_expiry * 86400),
                date: bs_params.expiration_date.clone(),
                days_to_expiry: bs_params.days_to_expiry,
            }];

            HttpResponse::Ok().json(TickerInfo {
                ticker,
                current_price,
                expirations,
            })
        }
        Err(e) => {
            HttpResponse::NotFound()
                .json(serde_json::json!({
                    "error": format!("No data available for {}. Please run: python3 download_options.py {}", ticker, ticker),
                    "details": e.to_string()
                }))
        }
    }
}

async fn analyze_options(
    req: web::Json<AnalyzeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let ticker = req.ticker.to_uppercase();

    let fetcher = data.fetcher.read().await;

    // Load from CSV file
    let (calls, current_price, bs_params) = match fetcher.load_from_csv(&ticker) {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({
                    "error": format!("No data available for {}. Please run: python3 download_options.py {}", ticker, ticker),
                    "details": e.to_string()
                }));
        }
    };

    // Use parameters from CSV
    let time_to_expiry = bs_params.time_to_expiry_years;
    let days_to_expiry = bs_params.days_to_expiry;
    let risk_free_rate = bs_params.risk_free_rate;
    let hist_volatility = bs_params.historical_volatility;
    let dividend_yield = bs_params.dividend_yield;
    let expiration_date = bs_params.expiration_date;

    // Create session ID
    let session_id = format!("{}_{}_{}", ticker, expiration_date, Uuid::new_v4());

    // Prepare chart data
    let chart_data: Vec<ChartPoint> = calls
        .iter()
        .map(|call| ChartPoint {
            strike: call.strike,
            market_price: call.last_price,
            theoretical_price: 0.0, // Will be calculated in visualization
            implied_volatility: call.implied_volatility,
            volume: call.volume.map(|v| v as f64),
        })
        .collect();

    // Create session directory
    let charts_dir = PathBuf::from("charts").join(&session_id);
    if let Err(e) = std::fs::create_dir_all(&charts_dir) {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("Failed to create charts directory: {}", e) }));
    }

    // Generate visualizations
    let config = VisualizationConfig {
        ticker: ticker.clone(),
        current_price,
        time_to_expiry,
        historical_volatility: Some(hist_volatility),
    };

    let charts_dir_str = charts_dir.to_string_lossy().to_string();
    if let Err(e) = visualization::generate_dashboard(&chart_data, &config, &charts_dir_str) {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("Failed to generate charts: {}", e) }));
    }

    // Collect generated chart URLs
    let charts: Vec<String> = std::fs::read_dir(&charts_dir)
        .unwrap_or_else(|_| panic!("Failed to read charts directory"))
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension()?.to_str()? == "png" {
                    Some(format!("/charts/{}/{}", session_id, path.file_name()?.to_str()?))
                } else {
                    None
                }
            })
        })
        .collect();

    // Find ATM option
    let atm_option = calls
        .iter()
        .min_by(|a, b| {
            (a.strike - current_price)
                .abs()
                .partial_cmp(&(b.strike - current_price).abs())
                .unwrap()
        });

    let (atm_strike, atm_price, atm_iv) = atm_option
        .map(|opt| (opt.strike, opt.last_price, opt.implied_volatility))
        .unwrap_or((0.0, 0.0, 0.0));

    let response = AnalysisResponse {
        session_id,
        ticker,
        current_price,
        expiration: expiration_date,
        days_to_expiry,
        time_to_expiry,
        risk_free_rate,
        hist_volatility,
        dividend_yield,
        charts,
        num_contracts: calls.len(),
        atm_strike,
        atm_price,
        atm_iv,
    };

    HttpResponse::Ok().json(response)
}

async fn list_sessions() -> impl Responder {
    let charts_dir = PathBuf::from("charts");

    if !charts_dir.exists() {
        return HttpResponse::Ok().json(Vec::<String>::new());
    }

    let sessions: Vec<String> = std::fs::read_dir(&charts_dir)
        .unwrap_or_else(|_| panic!("Failed to read charts directory"))
        .filter_map(|entry| entry.ok().map(|e| e.file_name().to_string_lossy().to_string()))
        .collect();

    HttpResponse::Ok().json(sessions)
}

#[derive(Debug, Serialize)]
struct FetchDataResponse {
    success: bool,
    ticker: String,
    message: String,
    output: String,
}

async fn fetch_data(
    req: web::Json<FetchDataRequest>,
) -> impl Responder {
    let ticker = req.ticker.to_uppercase();

    println!("\n[FETCH] Downloading options data for {}...", ticker);

    // Try to use venv Python first, fall back to system python3
    let python_path = if std::path::Path::new("venv/bin/python3").exists() {
        "venv/bin/python3"
    } else {
        "python3"
    };

    // Run the Python script
    let output = Command::new(python_path)
        .arg("download_options.py")
        .arg(&ticker)
        .current_dir(".")
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout).to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();

            if result.status.success() {
                println!("[FETCH] Successfully downloaded data for {}", ticker);
                HttpResponse::Ok().json(FetchDataResponse {
                    success: true,
                    ticker: ticker.clone(),
                    message: format!("Successfully downloaded options data for {}", ticker),
                    output: stdout,
                })
            } else {
                println!("[FETCH] Failed to download data for {}: {}", ticker, stderr);
                HttpResponse::InternalServerError().json(FetchDataResponse {
                    success: false,
                    ticker: ticker.clone(),
                    message: format!("Failed to download options data for {}", ticker),
                    output: format!("{}\n{}", stdout, stderr),
                })
            }
        }
        Err(e) => {
            println!("[FETCH] Error running Python script: {}", e);
            HttpResponse::InternalServerError().json(FetchDataResponse {
                success: false,
                ticker: ticker.clone(),
                message: format!("Failed to run data collection script: {}", e),
                output: String::new(),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("\n{}", "=".repeat(80));
    println!("OPTIONS PRICING WEB GUI - RUST SERVER");
    println!("{}", "=".repeat(80));
    println!("\nStarting server on http://localhost:8080");
    println!("Press Ctrl+C to stop the server\n");
    println!("{}\n", "=".repeat(80));

    // Ensure charts directory exists
    std::fs::create_dir_all("charts")?;
    std::fs::create_dir_all("static")?;

    let app_state = web::Data::new(AppState {
        fetcher: Arc::new(RwLock::new(DataFetcher::new())),
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .service(
                web::resource("/api/ticker/{ticker}")
                    .route(web::get().to(get_ticker_info))
            )
            .service(
                web::resource("/api/analyze")
                    .route(web::post().to(analyze_options))
            )
            .service(
                web::resource("/api/sessions")
                    .route(web::get().to(list_sessions))
            )
            .service(
                web::resource("/api/fetch-data")
                    .route(web::post().to(fetch_data))
            )
            .service(fs::Files::new("/charts", "charts").show_files_listing())
            .service(fs::Files::new("/", "static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
