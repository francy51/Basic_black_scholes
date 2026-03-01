use serde::{Deserialize, Serialize};
use std::process::Command;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonOptionData {
    pub ticker: String,
    pub current_price: f64,
    pub expiration_date: String,
    pub days_to_expiry: i64,
    pub time_to_expiry_years: f64,
    pub risk_free_rate: f64,
    pub historical_volatility: f64,
    pub dividend_yield: f64,
}

pub struct PythonDataFetcher;

impl PythonDataFetcher {
    pub fn new() -> Self {
        Self
    }

    pub fn fetch_option_data(&self, ticker: &str) -> Result<PythonOptionData, Box<dyn Error>> {
        // Call Python script to fetch data
        let output = Command::new("python3")
            .arg("download_options.py")
            .arg(ticker)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Python script failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        // Parse the output to extract parameters
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract values from stdout
        let current_price = self.extract_value(&stdout, "Current Stock Price (S): $")?;
        let risk_free_rate = self.extract_value(&stdout, "Risk-Free Rate (r):")?;
        let hist_vol = self.extract_value(&stdout, "Historical Volatility (σ):")?;
        let dividend_yield = self.extract_value(&stdout, "Dividend Yield (q):")?;
        let time_to_expiry = self.extract_value(&stdout, "Time to Expiration (T):")?;
        let days_to_expiry = self.extract_value(&stdout, "Days to Expiration:")?;

        Ok(PythonOptionData {
            ticker: ticker.to_uppercase(),
            current_price,
            expiration_date: String::new(),
            days_to_expiry: days_to_expiry as i64,
            time_to_expiry_years: time_to_expiry,
            risk_free_rate,
            historical_volatility: hist_vol,
            dividend_yield,
        })
    }

    fn extract_value(&self, text: &str, prefix: &str) -> Result<f64, Box<dyn Error>> {
        if let Some(pos) = text.find(prefix) {
            let start = pos + prefix.len();
            let rest = &text[start..];
            if let Some(end) = rest.find(|c: char| c == ' ' || c == '\n' || c == '(') {
                let value_str = &rest[..end].trim();
                // Remove % sign if present
                let cleaned = value_str.trim_end_matches('%');
                return Ok(cleaned.parse()?);
            }
        }
        Err(format!("Could not find value with prefix: {}", prefix).into())
    }
}

impl Default for PythonDataFetcher {
    fn default() -> Self {
        Self::new()
    }
}
