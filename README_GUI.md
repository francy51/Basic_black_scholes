# Options Pricing Web GUI

A Rust-first graphical user interface for options pricing analysis using the Black-Scholes model.

## Features

- **Real-time Data**: Fetches live option chain data from Yahoo Finance
- **Black-Scholes Analysis**: Calculates theoretical option prices using the Black-Scholes model
- **Interactive Visualizations**: Generates comprehensive charts including:
  - Volatility Smile/Skew
  - Market vs Theoretical Price Comparison
  - Price Difference Analysis
- **Web Interface**: Clean, responsive web UI for easy interaction
- **Rust-Powered**: Fast, safe, and efficient backend

## Prerequisites

- Rust 1.70 or higher
- Internet connection (for fetching market data)

## Installation

1. Clone the repository:
```bash
cd options_pricing
```

2. Build the project:
```bash
cargo build --release
```

## Usage

### Starting the Web Server

Run the web GUI server:

```bash
./target/release/web_gui
```

The server will start on `http://localhost:8080`.

### Using the Web Interface

1. Open your browser and navigate to `http://localhost:8080`
2. Enter a stock ticker symbol (e.g., AAPL, TSLA, SPY)
3. Select an expiration date from the dropdown
4. Click "Analyze Options"
5. View the generated charts and analysis

### Command Line Interface (Original)

You can also use the original CLI tool:

```bash
# First, generate CSV data using Python
python download_options.py AAPL

# Then analyze with Rust
./target/release/options_pricing AAPL_2026-03-02_20260228_113854_calls.csv
```

## Project Structure

```
options_pricing/
├── src/
│   ├── main.rs              # CLI application
│   ├── web_server.rs        # Web server and API
│   ├── data_fetcher.rs      # Yahoo Finance API integration
│   └── visualization.rs     # Chart generation
├── static/
│   └── index.html           # Web UI
├── charts/                   # Generated charts
├── Cargo.toml               # Rust dependencies
└── README.md
```

## API Endpoints

- `GET /api/ticker/{ticker}` - Get ticker information and available expiration dates
- `POST /api/analyze` - Analyze options for a given ticker and expiration
  - Body: `{"ticker": "AAPL", "expiration": 1234567890}`
- `GET /api/sessions` - List all analysis sessions
- `GET /charts/{session_id}/{filename}` - Retrieve generated charts

## Generated Charts

The application generates three main charts:

1. **Volatility Smile/Skew**: Shows implied volatility across different strike prices
2. **Market vs Theoretical Price**: Compares market prices to Black-Scholes theoretical prices
3. **Price Difference**: Highlights mispricing between market and theoretical values

## Black-Scholes Model

The application uses the Black-Scholes model for European options on dividend-paying stocks:

**Call Option:**
```
C = S·e^(-qT)·N(d1) - K·e^(-rT)·N(d2)
```

**Put Option:**
```
P = K·e^(-rT)·N(-d2) - S·e^(-qT)·N(-d1)
```

Where:
- S = Current stock price
- K = Strike price
- T = Time to expiration (years)
- r = Risk-free rate
- σ = Volatility
- q = Dividend yield

## Technical Stack

- **Backend**: Rust with Actix-web framework
- **Data Source**: Yahoo Finance API
- **Visualization**: Plotters library
- **Frontend**: Vanilla HTML/CSS/JavaScript
- **Analysis**: Black-Scholes model with statistical calculations

## Development

### Building in Debug Mode

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Cleaning Build Artifacts

```bash
cargo clean
```

## Troubleshooting

### Server won't start
- Ensure port 8080 is not already in use
- Check that all dependencies are installed: `cargo check`

### No data returned for ticker
- Verify the ticker symbol is valid
- Check your internet connection
- Some tickers may not have options available

### Charts not displaying
- Ensure the `charts/` directory exists and is writable
- Check browser console for any errors

## License

This project is for educational purposes. Please ensure compliance with Yahoo Finance's API terms of service when using this tool.

## Acknowledgments

- Yahoo Finance for providing free market data
- The Rust community for excellent libraries
- Black-Scholes model pioneers
