# Options Pricing Data Downloader

A Python script to download comprehensive option chain data with all inputs needed for Black-Scholes option pricing models.

## Features

- **Option Chain Data**: Downloads complete call and put chains
- **Black-Scholes Inputs**: Automatically calculates/fetches:
  - Current stock price (S)
  - Risk-free rate (r) - from Treasury yields
  - Historical volatility (σ) - from daily returns
  - Dividend yield (q)
  - Time to expiration (T)
- **Implied Volatility Analysis**: Compares market IV to historical volatility
- **CSV Export**: Saves all data with Black-Scholes parameters included

## Installation

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

## Usage

### Basic Usage
```bash
# Activate virtual environment
source venv/bin/activate

# Run with default ticker (AAPL)
python3 download_options.py

# Run with a specific ticker
python3 download_options.py TSLA
python3 download_options.py SPY
python3 download_options.py MSFT
```

### Output Files

The script generates three CSV files per run:

1. **`{TICKER}_{DATE}_{TIMESTAMP}_calls.csv`** - Call options with BS parameters
2. **`{TICKER}_{DATE}_{TIMESTAMP}_puts.csv`** - Put options with BS parameters
3. **`{TICKER}_{DATE}_{TIMESTAMP}_black_scholes_params.csv`** - BS model inputs

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

## Data Sources

- **Option Chains**: Yahoo Finance via yfinance
- **Risk-Free Rate**: 13-week Treasury Bill (^IRX) or 10-year Treasury Note (^TNX)
- **Historical Volatility**: Calculated from daily returns over 1 year
- **Dividend Yield**: Yahoo Finance company info

## Next Steps

You can now use this data to:

1. **Price options** using the Black-Scholes model
2. **Compare market prices** to theoretical values
3. **Identify mispricing** by comparing IV to historical volatility
4. **Backtest strategies** using historical option data
5. **Calculate Greeks** (delta, gamma, theta, vega, rho)

## Requirements

- Python 3.7+
- yfinance >= 0.2.36
- pandas >= 2.0.0
- numpy >= 1.24.0

## License

MIT License - Feel free to use and modify for your options analysis needs.
