/// Data fetching module for options pricing
/// Fetches option chain data from Yahoo Finance API or uses existing CSV files
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use csv::Reader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YahooOptionChain {
    pub underlying_symbol: String,
    pub expiration_dates: Vec<i64>,
    pub strikes: OptionData,
    pub calls: Vec<OptionContract>,
    pub puts: Vec<OptionContract>,
    pub has_mini_options: bool,
    pub quote: YahooQuote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionData {
    pub underlying_price: Option<f64>,
    pub expiration_date: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    pub contract_symbol: String,
    pub strike: f64,
    pub currency: String,
    #[serde(default)]
    pub last_price: f64,
    #[serde(default)]
    pub change: f64,
    #[serde(default)]
    pub percent_change: f64,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub open_interest: Option<i64>,
    #[serde(default)]
    pub bid: Option<f64>,
    #[serde(default)]
    pub ask: Option<f64>,
    #[serde(default)]
    pub implied_volatility: f64,
    #[serde(default)]
    pub in_the_money: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YahooQuote {
    #[serde(rename = "regularMarketPrice")]
    pub regular_market_price: f64,
    #[serde(rename = "regularMarketDayHigh")]
    pub regular_market_day_high: Option<f64>,
    #[serde(rename = "regularMarketDayLow")]
    pub regular_market_day_low: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackScholesParams {
    pub ticker: String,
    pub current_price: f64,
    pub expiration_date: String,
    pub days_to_expiry: i64,
    pub time_to_expiry_years: f64,
    pub risk_free_rate: f64,
    pub historical_volatility: f64,
    pub dividend_yield: f64,
    pub download_date: String,
}

pub struct DataFetcher {
    client: Client,
}

impl DataFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());
                    headers.insert(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
                    headers
                })
                .build()
                .unwrap(),
        }
    }

    /// Load option data from existing CSV file
    pub fn load_from_csv(&self, ticker: &str) -> Result<(Vec<OptionContract>, f64, BlackScholesParams), Box<dyn Error>> {
        // Look for existing CSV files for this ticker
        let mut csv_file: Option<std::path::PathBuf> = None;

        for entry in std::fs::read_dir(".")? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with(&format!("{}_", ticker)) && filename.ends_with("_calls.csv") {
                csv_file = Some(entry.path());
                break;
            }
        }

        let csv_path = csv_file.ok_or_else(|| {
            format!("No CSV file found for ticker {}. Available tickers: AAPL, PLTR, TSLA", ticker)
        })?;

        let file = File::open(&csv_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = Reader::from_reader(reader);

        let mut calls = Vec::new();
        let mut bs_params: Option<BlackScholesParams> = None;

        // Get headers
        let headers = csv_reader.headers()?.clone();

        for result in csv_reader.records() {
            let record = result?;

            // Extract values using headers
            let strike = record.get(headers.iter().position(|h| h == "strike").unwrap_or(0))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);

            let last_price = record.get(headers.iter().position(|h| h == "lastPrice").unwrap_or(3))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);

            let iv = record.get(headers.iter().position(|h| h == "impliedVolatility").unwrap_or(10))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);

            let volume = record.get(headers.iter().position(|h| h == "volume").unwrap_or(8))
                .and_then(|v| v.parse::<i64>().ok());

            let open_interest = record.get(headers.iter().position(|h| h == "openInterest").unwrap_or(9))
                .and_then(|v| v.parse::<i64>().ok());

            let bid = record.get(headers.iter().position(|h| h == "bid").unwrap_or(4))
                .and_then(|v| v.parse::<f64>().ok());

            let ask = record.get(headers.iter().position(|h| h == "ask").unwrap_or(5))
                .and_then(|v| v.parse::<f64>().ok());

            let in_the_money = record.get(headers.iter().position(|h| h == "inTheMoney").unwrap_or(11))
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false);

            let contract_symbol = record.get(headers.iter().position(|h| h == "contractSymbol").unwrap_or(0))
                .unwrap_or("")
                .to_string();

            calls.push(OptionContract {
                contract_symbol,
                strike,
                currency: "USD".to_string(),
                last_price,
                change: 0.0,
                percent_change: 0.0,
                volume,
                open_interest,
                bid,
                ask,
                implied_volatility: iv,
                in_the_money,
            });

            // Extract BS params from first row
            if bs_params.is_none() {
                let current_price = record.get(headers.iter().position(|h| h == "BS_current_price").unwrap_or(15))
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let expiration_date = record.get(headers.iter().position(|h| h == "BS_expiration_date").unwrap_or(16))
                    .unwrap_or("")
                    .to_string();

                let days_to_expiry = record.get(headers.iter().position(|h| h == "BS_days_to_expiry").unwrap_or(17))
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);

                let time_to_expiry_years = record.get(headers.iter().position(|h| h == "BS_time_to_expiry_years").unwrap_or(18))
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let risk_free_rate = record.get(headers.iter().position(|h| h == "BS_risk_free_rate").unwrap_or(19))
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.05);

                let historical_volatility = record.get(headers.iter().position(|h| h == "BS_historical_volatility").unwrap_or(20))
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.30);

                let dividend_yield = record.get(headers.iter().position(|h| h == "BS_dividend_yield").unwrap_or(21))
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let download_date = record.get(headers.iter().position(|h| h == "BS_download_date").unwrap_or(22))
                    .unwrap_or("")
                    .to_string();

                bs_params = Some(BlackScholesParams {
                    ticker: ticker.to_string(),
                    current_price,
                    expiration_date,
                    days_to_expiry,
                    time_to_expiry_years,
                    risk_free_rate,
                    historical_volatility,
                    dividend_yield,
                    download_date,
                });
            }
        }

        let params = bs_params.ok_or("Could not extract Black-Scholes parameters from CSV")?;
        let current_price = params.current_price;

        Ok((calls, current_price, params))
    }

    /// Fetch current stock price
    pub async fn get_current_price(&self, ticker: &str) -> Result<f64, Box<dyn Error>> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
            ticker
        );

        let response = self.client.get(&url).send().await?;
        let json: serde_json::Value = response.json().await?;

        let price = json["chart"]["result"][0]["meta"]["regularMarketPrice"]
            .as_f64()
            .ok_or("Could not parse current price")?;

        Ok(price)
    }

    /// Get available expiration dates for a ticker
    pub async fn get_expiration_dates(&self, ticker: &str) -> Result<Vec<i64>, Box<dyn Error>> {
        let url = format!(
            "https://query1.finance.yahoo.com/v7/finance/options/{}",
            ticker
        );

        let response = self.client.get(&url).send().await?;

        // Check if response is successful
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let json: serde_json::Value = response.json().await?;

        // Check if optionChain exists
        let option_chain = json.get("optionChain")
            .and_then(|oc| oc.get("result"))
            .and_then(|r| r.as_array())
            .ok_or("Could not find optionChain result in response")?;

        if option_chain.is_empty() {
            return Err("No option chain data available".into());
        }

        let dates = option_chain[0].get("expirationDates")
            .and_then(|d| d.as_array())
            .ok_or("Could not parse expiration dates from response")?
            .iter()
            .filter_map(|v| v.as_i64())
            .collect();

        Ok(dates)
    }

    /// Fetch option chain for a specific expiration date
    pub async fn get_option_chain(
        &self,
        ticker: &str,
        expiration_timestamp: i64,
    ) -> Result<(Vec<OptionContract>, Vec<OptionContract>, f64), Box<dyn Error>> {
        let url = format!(
            "https://query1.finance.yahoo.com/v7/finance/options/{}?date={}",
            ticker, expiration_timestamp
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let json: serde_json::Value = response.json().await?;

        let result = json.get("optionChain")
            .and_then(|oc| oc.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|a| a.get(0))
            .ok_or("Could not find option chain result")?;

        let current_price = result.get("meta")
            .and_then(|m| m.get("regularMarketPrice"))
            .and_then(|p| p.as_f64())
            .ok_or("Could not parse current price")?;

        let options = result.get("options")
            .and_then(|o| o.as_array())
            .and_then(|a| a.get(0))
            .ok_or("Could not find options data")?;

        // Parse calls
        let calls: Vec<OptionContract> = options.get("calls")
            .and_then(|c| serde_json::from_value(c.clone()).ok())
            .unwrap_or_default();

        // Parse puts
        let puts: Vec<OptionContract> = options.get("puts")
            .and_then(|p| serde_json::from_value(p.clone()).ok())
            .unwrap_or_default();

        Ok((calls, puts, current_price))
    }

    /// Get risk-free rate (using hardcoded value or could fetch from FRED API)
    pub fn get_risk_free_rate(&self) -> f64 {
        // Default to 5% - in production you'd fetch this from an API
        0.05
    }

    /// Calculate historical volatility (simplified - uses default)
    pub fn calculate_historical_volatility(&self) -> f64 {
        // Default to 30% - in production you'd calculate from historical data
        0.30
    }

    /// Get dividend yield (simplified - uses default)
    pub fn get_dividend_yield(&self, _ticker: &str) -> f64 {
        // Default to 0% - in production you'd fetch this from API
        0.0
    }

    /// Calculate time to expiration
    pub fn calculate_time_to_expiry(&self, expiration_timestamp: i64) -> (f64, i64) {
        let now = Utc::now().timestamp();
        let seconds_to_expiry = expiration_timestamp - now;
        let days_to_expiry = seconds_to_expiry / 86400;
        let years_to_expiry = days_to_expiry as f64 / 365.25;

        (years_to_expiry, days_to_expiry)
    }

    /// Convert timestamp to date string
    pub fn timestamp_to_date(&self, timestamp: i64) -> String {
        let dt: DateTime<Utc> = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        dt.format("%Y-%m-%d").to_string()
    }
}

impl Default for DataFetcher {
    fn default() -> Self {
        Self::new()
    }
}
