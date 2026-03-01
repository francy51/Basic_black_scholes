# Options Pricing Analyzer

A powerful options pricing toolkit built with Rust and Python that implements the Black-Scholes model with real-time data fetching, interactive visualizations, and both CLI and web interfaces.

## Features

- **Black-Scholes Pricing Model**: Calculate theoretical option prices with dividend yield support
- **Real-Time Data**: Fetch live options data from Yahoo Finance
- **Interactive Web GUI**: User-friendly web interface for analysis
- **CLI Tool**: Command-line interface for batch processing
- **Visualizations**: Generate volatility smiles, price comparisons, and volume analysis charts
- **Multiple Expiration Dates**: Support for various option expiration timelines
- **Option Chain Data**: Downloads complete call and put chains
- **Automatic Parameter Calculation**: Risk-free rate, historical volatility, dividend yield

## Prerequisites

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **Python 3.8+** with pip
- **Internet connection** (for fetching market data)

## Quick Start

### 1. Clone the Repository

```bash
git clone <your-repo-url>
cd options_pricing
```

### 2. Set Up Python Environment

```bash
# Create virtual environment
python3 -m venv venv

# Activate virtual environment
source venv/bin/activate  # On macOS/Linux
# OR
.\venv\Scripts\activate   # On Windows

# Install Python dependencies
pip install yfinance pandas numpy
```

### 3. Build the Rust Project

```bash
cargo build --release
```

## Usage

### Option 1: Web GUI (Recommended)

Start the web server:

```bash
cargo run --bin web_gui --release
```

Or simply:

```bash
cargo run --bin web_gui
```

Then open your browser to: **http://localhost:8080**

#### Using the Web Interface:

1. **Fetch Data**: Enter a ticker symbol (e.g., AAPL, TSLA, MSFT) and click "📥 Fetch Latest Data"
2. **Select Expiration**: Choose an expiration date from the dropdown
3. **Analyze**: Click "📊 Analyze Options" to generate visualizations and analysis

### Option 2: Command Line Interface

First, fetch options data using Python:

```bash
# Activate virtual environment
source venv/bin/activate

# Fetch data for a ticker
python3 download_options.py AAPL
```

Then analyze with the CLI tool:

```bash
cargo run --bin options_pricing -- AAPL_2026-03-02_*.csv
```

Replace the CSV filename with the actual generated file.

### Output Files

The tool generates three CSV files per run:

1. **`{TICKER}_{DATE}_{TIMESTAMP}_calls.csv`** - Call options with BS parameters
2. **`{TICKER}_{DATE}_{TIMESTAMP}_puts.csv`** - Put options with BS parameters
3. **`{TICKER}_{DATE}_{TIMESTAMP}_black_scholes_params.csv`** - BS model inputs

## Project Structure

```
options_pricing/
├── src/
│   ├── main.rs              # CLI application
│   ├── web_server.rs        # Web GUI server
│   ├── data_fetcher.rs      # Data loading and parsing
│   └── visualization.rs     # Chart generation
├── static/
│   └── index.html           # Web interface
├── charts/                   # Generated visualizations
├── download_options.py      # Python data fetcher
├── Cargo.toml               # Rust dependencies
└── README.md                # This file
```

## API Endpoints

When running the web server, the following API endpoints are available:

- `GET /api/ticker/{TICKER}` - Get ticker information and available expiration dates
- `POST /api/fetch-data` - Fetch latest options data for a ticker
  - Body: `{"ticker": "AAPL"}`
- `POST /api/analyze` - Analyze options and generate charts
  - Body: `{"ticker": "AAPL", "expiration": 1234567890}`
- `GET /api/sessions` - List all analysis sessions

## Visualizations

The tool generates the following charts:

1. **Volatility Smile/Skew**: Implied volatility across different strike prices
2. **Price Comparison**: Market prices vs. Black-Scholes theoretical prices
3. **Price Difference**: Discrepancies between market and theoretical prices
4. **Volume Analysis**: Trading volume and open interest by strike price

Charts are saved as PNG files in the `charts/` directory.

### Black-Scholes Parameters Included

Each option chain CSV includes these additional columns:

- `BS_ticker`: Stock symbol
- `BS_current_price`: Current stock price (S)
- `BS_expiration_date`: Option expiration date
- `BS_days_to_expiry`: Calendar days to expiration
- `BS_time_to_expiry_years`: Time to expiration in years (T)
- `BS_risk_free_rate`: Risk-free interest rate (r)
- `BS_historical_volatility`: Annualized historical volatility (σ)
- `BS_dividend_yield`: Annual dividend yield (q)
- `BS_download_date`: Data download timestamp

## Black-Scholes Formula Reference

### Call Option
```
C = S·e^(-qT)·N(d1) - K·e^(-rT)·N(d2)
```

### Put Option
```
P = K·e^(-rT)·N(-d2) - S·e^(-qT)·N(-d1)
```

### Where
```
d1 = [ln(S/K) + (r - q + σ²/2)T] / (σ·√T)
d2 = d1 - σ·√T
```

### Variables
- **S** = Current stock price
- **K** = Strike price
- **T** = Time to expiration (years)
- **r** = Risk-free rate
- **σ** = Volatility (historical or implied)
- **q** = Dividend yield
- **N(x)** = Cumulative normal distribution function

## Example Output

```
================================================================================
BLACK-SCHOLES MODEL INPUTS
================================================================================

Ticker: AAPL
Current Stock Price (S): $264.18
Expiration Date: 2026-03-02
Days to Expiration: 1
Time to Expiration (T): 0.0027 years
Risk-Free Rate (r): 0.0358 (3.58%)
Historical Volatility (σ): 0.3248 (32.48%)
Dividend Yield (q): 0.3900 (39.00%)

================================================================================
ATM OPTIONS & IMPLIED VOLATILITY COMPARISON
================================================================================

ATM Call (Strike: $265.00):
  Market Price: $1.72
  Implied Volatility: 0.2251 (22.51%)
  Historical Vol: 0.3248 (32.48%)
  IV vs Hist Vol: -30.7%
```

## Dependencies

### Rust (Cargo.toml)
- `actix-web` - Web framework
- `serde` - Serialization
- `statrs` - Statistical functions
- `plotters` - Chart generation
- `csv` - CSV parsing
- `chrono` - Date/time handling

### Python (pip)
- `yfinance` - Yahoo Finance API
- `pandas` - Data manipulation
- `numpy` - Numerical computations

## Troubleshooting

### "ModuleNotFoundError: No module named 'yfinance'"
Make sure you've activated the virtual environment and installed dependencies:
```bash
source venv/bin/activate
pip install yfinance pandas numpy
```

### "Failed to fetch data"
- Check your internet connection
- Verify the ticker symbol is valid
- Check the server logs for detailed error messages

### Port 8080 already in use
Stop the existing server or change the port in `src/web_server.rs`:
```rust
.bind("127.0.0.1:8080")?  // Change to a different port
```

## Examples

### Analyze Apple (AAPL) Options

**Via Web Interface:**
```bash
# Start server
cargo run --bin web_gui

# Open http://localhost:8080
# 1. Enter ticker: AAPL
# 2. Click "📥 Fetch Latest Data"
# 3. Select expiration date
# 4. Click "📊 Analyze Options"
```

**Via CLI:**
```bash
# Fetch data
source venv/bin/activate
python3 download_options.py AAPL

# Analyze
cargo run --bin options_pricing -- AAPL_*.csv
```

### Analyze Tesla (TSLA) Options
```bash
# Via web interface
cargo run --bin web_gui
# Open http://localhost:8080, enter TSLA, fetch and analyze
```

## Data Sources

- **Option Chains**: Yahoo Finance via yfinance
- **Risk-Free Rate**: 13-week Treasury Bill (^IRX) or 10-year Treasury Note (^TNX)
- **Historical Volatility**: Calculated from daily returns over 1 year
- **Dividend Yield**: Yahoo Finance company info

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Built with ❤️ using Rust and Python

## Acknowledgments

- Black-Scholes model implementation
- Yahoo Finance for options data
- Plotters library for visualizations
- Actix-web for the web framework
