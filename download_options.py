#!/usr/bin/env python3
"""
Options Chain Data Downloader using yfinance

This script downloads option chain data for a specified ticker and displays
key information including calls, puts, and their greeks/implied volatility.

Enhanced with Black-Scholes model inputs:
- Historical volatility
- Risk-free rate ( Treasury yield)
- Dividend yield
- Time to expiration
"""

import yfinance as yf
import pandas as pd
import numpy as np
from datetime import datetime, timedelta
import sys


def get_risk_free_rate():
    """
    Get the current risk-free rate using 13-week Treasury Bill (^IRX) or 10-year Treasury (^TNX).

    Returns:
        float: Risk-free rate as a decimal (e.g., 0.05 for 5%)
    """
    try:
        # Try to get 13-week T-bill rate first (better for short-term options)
        t_bill = yf.Ticker("^IRX")
        rf_rate = t_bill.history(period="5d")['Close'].iloc[-1] / 100
        return rf_rate
    except:
        try:
            # Fallback to 10-year Treasury
            t_note = yf.Ticker("^TNX")
            rf_rate = t_note.history(period="5d")['Close'].iloc[-1] / 100
            return rf_rate
        except:
            print("Warning: Could not fetch risk-free rate, using default 5%")
            return 0.05


def calculate_historical_volatility(ticker_symbol, period='1y'):
    """
    Calculate historical volatility from daily returns.

    Args:
        ticker_symbol (str): Stock ticker symbol
        period (str): Period to calculate volatility (default: '1y')

    Returns:
        float: Annualized volatility as a decimal
    """
    ticker = yf.Ticker(ticker_symbol)
    hist = ticker.history(period=period)

    if hist.empty:
        print("Warning: Could not fetch historical data, using default volatility 30%")
        return 0.30

    # Calculate daily returns
    daily_returns = hist['Close'].pct_change().dropna()

    # Calculate annualized volatility (252 trading days)
    volatility = daily_returns.std() * np.sqrt(252)

    return volatility


def get_dividend_yield(ticker_symbol):
    """
    Get the dividend yield for a stock.

    Args:
        ticker_symbol (str): Stock ticker symbol

    Returns:
        float: Dividend yield as a decimal
    """
    ticker = yf.Ticker(ticker_symbol)

    try:
        # Get dividend yield from info
        dividend_yield = ticker.info.get('dividendYield', 0)
        if dividend_yield is None:
            dividend_yield = 0
        return dividend_yield
    except:
        print("Warning: Could not fetch dividend yield, assuming 0%")
        return 0.0


def calculate_time_to_expiration(expiration_date):
    """
    Calculate time to expiration in years.

    Args:
        expiration_date (str): Expiration date in 'YYYY-MM-DD' format

    Returns:
        float: Time to expiration in years
    """
    expiry = datetime.strptime(expiration_date, '%Y-%m-%d')
    today = datetime.now()

    # Calculate days to expiration
    days_to_expiry = (expiry - today).days

    # Convert to years (using 365.25 for accuracy)
    time_to_expiry = days_to_expiry / 365.25

    return time_to_expiry, days_to_expiry


def get_option_chain(ticker_symbol, expiration_date=None):
    """
    Download option chain data for a given ticker.

    Args:
        ticker_symbol (str): Stock ticker symbol (e.g., 'AAPL', 'SPY')
        expiration_date (str, optional): Specific expiration date (YYYY-MM-DD).
                                        If None, shows available dates.

    Returns:
        yfinance Options object
    """
    print(f"\n{'='*60}")
    print(f"Fetching option chain data for: {ticker_symbol}")
    print(f"{'='*60}\n")

    # Create ticker object
    ticker = yf.Ticker(ticker_symbol)

    # Get available expiration dates
    expirations = ticker.options

    if not expirations:
        print(f"No options available for {ticker_symbol}")
        return None

    print(f"Available expiration dates: {len(expirations)}")
    for i, exp in enumerate(expirations[:10]):  # Show first 10
        print(f"  {i+1}. {exp}")
    if len(expirations) > 10:
        print(f"  ... and {len(expirations) - 10} more dates\n")

    # If no specific date provided, use the first available
    if expiration_date is None:
        expiration_date = expirations[0]
        print(f"\nUsing nearest expiration: {expiration_date}")
    elif expiration_date not in expirations:
        print(f"Error: {expiration_date} is not a valid expiration date")
        print(f"Available dates: {', '.join(expirations[:5])}")
        return None

    # Get option chain
    option_chain = ticker.option_chain(expiration_date)

    return option_chain, expiration_date


def display_option_chain(option_chain, ticker_symbol, expiration_date, option_type='all'):
    """
    Display option chain data in a formatted table.

    Args:
        option_chain: yfinance OptionChain object
        ticker_symbol (str): Stock ticker symbol
        expiration_date (str): Expiration date
        option_type (str): 'calls', 'puts', or 'all'
    """
    calls = option_chain.calls
    puts = option_chain.puts

    # Select columns to display
    columns_to_show = [
        'strike',
        'lastPrice',
        'bid',
        'ask',
        'volume',
        'openInterest',
        'impliedVolatility',
        'inTheMoney'
    ]

    print(f"\n{'='*80}")
    print(f"{ticker_symbol} Options - Expiration: {expiration_date}")
    print(f"{'='*80}")

    if option_type in ['all', 'calls']:
        print(f"\n--- CALLS ({len(calls)} contracts) ---\n")
        print(calls[columns_to_show].to_string(index=False))

    if option_type in ['all', 'puts']:
        print(f"\n--- PUTS ({len(puts)} contracts) ---\n")
        print(puts[columns_to_show].to_string(index=False))


def save_to_csv(option_chain, ticker_symbol, expiration_date, bs_data, output_dir='.'):
    """
    Save option chain data and Black-Scholes inputs to CSV files.

    Args:
        option_chain: yfinance OptionChain object
        ticker_symbol (str): Stock ticker symbol
        expiration_date (str): Expiration date
        bs_data (dict): Black-Scholes input data
        output_dir (str): Directory to save CSV files
    """
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    base_filename = f"{output_dir}/{ticker_symbol}_{expiration_date}_{timestamp}"

    # Save calls with Black-Scholes inputs
    calls_file = f"{base_filename}_calls.csv"
    calls_with_bs = option_chain.calls.copy()
    # Add BS parameters to each row
    for key, value in bs_data.items():
        calls_with_bs[f'BS_{key}'] = value
    calls_with_bs.to_csv(calls_file, index=False)
    print(f"Calls saved to: {calls_file}")

    # Save puts with Black-Scholes inputs
    puts_file = f"{base_filename}_puts.csv"
    puts_with_bs = option_chain.puts.copy()
    # Add BS parameters to each row
    for key, value in bs_data.items():
        puts_with_bs[f'BS_{key}'] = value
    puts_with_bs.to_csv(puts_file, index=False)
    print(f"Puts saved to: {puts_file}")

    # Save Black-Scholes parameters to a separate file
    bs_file = f"{base_filename}_black_scholes_params.csv"
    bs_df = pd.DataFrame([bs_data])
    bs_df.to_csv(bs_file, index=False)
    print(f"Black-Scholes parameters saved to: {bs_file}")

    return calls_file, puts_file, bs_file


def get_atm_options(option_chain, current_price):
    """
    Find at-the-money (ATM) options.

    Args:
        option_chain: yfinance OptionChain object
        current_price (float): Current stock price

    Returns:
        tuple: (ATM call, ATM put) as DataFrames
    """
    calls = option_chain.calls
    puts = option_chain.puts

    # Find ATM options (strike closest to current price)
    atm_call = calls.iloc[(calls['strike'] - current_price).abs().argsort()[:1]]
    atm_put = puts.iloc[(puts['strike'] - current_price).abs().argsort()[:1]]

    return atm_call, atm_put


def main():
    """Main function to run the option chain downloader."""

    # Default configuration
    ticker_symbol = 'AAPL'  # Change this to your desired ticker

    # Check if ticker provided as command line argument
    if len(sys.argv) > 1:
        ticker_symbol = sys.argv[1].upper()

    print(f"\n{'='*80}")
    print(f"BLACK-SCHOLES OPTION DATA DOWNLOADER")
    print(f"{'='*80}")

    # Get current stock price
    ticker = yf.Ticker(ticker_symbol)
    current_price = ticker.history(period='1d')['Close'].iloc[-1]

    print(f"\nTicker: {ticker_symbol}")
    print(f"Current Stock Price (S): ${current_price:.2f}")

    # Download option chain
    result = get_option_chain(ticker_symbol)

    if result is None:
        return

    option_chain, expiration_date = result

    # Calculate Black-Scholes inputs
    print(f"\n{'='*80}")
    print("BLACK-SCHOLES MODEL INPUTS")
    print(f"{'='*80}")

    # 1. Time to expiration
    time_to_expiry, days_to_expiry = calculate_time_to_expiration(expiration_date)
    print(f"\nExpiration Date: {expiration_date}")
    print(f"Days to Expiration: {days_to_expiry}")
    print(f"Time to Expiration (T): {time_to_expiry:.4f} years")

    # 2. Risk-free rate
    risk_free_rate = get_risk_free_rate()
    print(f"Risk-Free Rate (r): {risk_free_rate:.4f} ({risk_free_rate*100:.2f}%)")

    # 3. Historical volatility
    hist_volatility = calculate_historical_volatility(ticker_symbol)
    print(f"Historical Volatility (σ): {hist_volatility:.4f} ({hist_volatility*100:.2f}%)")

    # 4. Dividend yield
    dividend_yield = get_dividend_yield(ticker_symbol)
    print(f"Dividend Yield (q): {dividend_yield:.4f} ({dividend_yield*100:.2f}%)")

    # Prepare Black-Scholes data dictionary
    bs_data = {
        'ticker': ticker_symbol,
        'current_price': current_price,
        'expiration_date': expiration_date,
        'days_to_expiry': days_to_expiry,
        'time_to_expiry_years': time_to_expiry,
        'risk_free_rate': risk_free_rate,
        'historical_volatility': hist_volatility,
        'dividend_yield': dividend_yield,
        'download_date': datetime.now().strftime('%Y-%m-%d %H:%M:%S')
    }

    # Display option chain
    display_option_chain(option_chain, ticker_symbol, expiration_date)

    # Show ATM options with Black-Scholes context
    print(f"\n{'='*80}")
    print("ATM OPTIONS & IMPLIED VOLATILITY COMPARISON")
    print(f"{'='*80}")
    atm_call, atm_put = get_atm_options(option_chain, current_price)

    atm_call_iv = atm_call['impliedVolatility'].values[0]
    atm_put_iv = atm_put['impliedVolatility'].values[0]

    print(f"\nATM Call (Strike: ${atm_call['strike'].values[0]:.2f}):")
    print(f"  Market Price: ${atm_call['lastPrice'].values[0]:.2f}")
    print(f"  Implied Volatility: {atm_call_iv:.4f} ({atm_call_iv*100:.2f}%)")
    print(f"  Historical Vol: {hist_volatility:.4f} ({hist_volatility*100:.2f}%)")
    print(f"  IV vs Hist Vol: {((atm_call_iv/hist_volatility)-1)*100:+.1f}%")
    print(f"  Volume: {atm_call['volume'].values[0]:,.0f}")
    print(f"  Open Interest: {atm_call['openInterest'].values[0]:,.0f}")

    print(f"\nATM Put (Strike: ${atm_put['strike'].values[0]:.2f}):")
    print(f"  Market Price: ${atm_put['lastPrice'].values[0]:.2f}")
    print(f"  Implied Volatility: {atm_put_iv:.4f} ({atm_put_iv*100:.2f}%)")
    print(f"  Historical Vol: {hist_volatility:.4f} ({hist_volatility*100:.2f}%)")
    print(f"  IV vs Hist Vol: {((atm_put_iv/hist_volatility)-1)*100:+.1f}%")
    print(f"  Volume: {atm_put['volume'].values[0]:,.0f}")
    print(f"  Open Interest: {atm_put['openInterest'].values[0]:,.0f}")

    # Save to CSV with Black-Scholes parameters
    print(f"\n{'='*80}")
    print("SAVING DATA TO CSV FILES")
    print(f"{'='*80}")
    save_to_csv(option_chain, ticker_symbol, expiration_date, bs_data)

    print(f"\n{'='*80}")
    print("BLACK-SCHOLES FORMULA REFERENCE")
    print(f"{'='*80}")
    print("\nFor a European option on a dividend-paying stock:")
    print("\nCall Option:")
    print("  C = S·e^(-qT)·N(d1) - K·e^(-rT)·N(d2)")
    print("\nPut Option:")
    print("  P = K·e^(-rT)·N(-d2) - S·e^(-qT)·N(-d1)")
    print("\nWhere:")
    print("  d1 = [ln(S/K) + (r - q + σ²/2)T] / (σ·√T)")
    print("  d2 = d1 - σ·√T")
    print("\nVariables:")
    print(f"  S = {current_price:.2f} (Current stock price)")
    print(f"  K = Strike price")
    print(f"  T = {time_to_expiry:.4f} (Time to expiration in years)")
    print(f"  r = {risk_free_rate:.4f} (Risk-free rate)")
    print(f"  σ = {hist_volatility:.4f} (Volatility)")
    print(f"  q = {dividend_yield:.4f} (Dividend yield)")

    print(f"\n{'='*80}")
    print("DATA DOWNLOAD COMPLETE")
    print(f"{'='*80}\n")


if __name__ == "__main__":
    main()
