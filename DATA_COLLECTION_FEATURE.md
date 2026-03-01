# Data Collection Feature

The web GUI now includes the ability to fetch options data directly from the web interface.

## New Feature: Fetch Data Button

### Overview
A "Fetch Latest Data" button has been added to the web interface that allows you to download fresh options data without leaving the browser.

### How It Works

1. **Enter a Ticker Symbol**: Type any valid stock ticker (e.g., AAPL, TSLA, MSFT) in the input field
2. **Click "📥 Fetch Latest Data"**: The system will:
   - Run the Python data collection script automatically
   - Download the latest options chain data from Yahoo Finance
   - Save CSV files to the project directory
   - Display the output and results
3. **Analyze**: Once data is fetched, you can immediately analyze it using the "📊 Analyze Options" button

### Technical Details

#### Backend Changes
- **New Endpoint**: `/api/fetch-data` (POST)
  - Accepts: `{"ticker": "SYMBOL"}`
  - Returns: Success/failure status with detailed output
  - Uses the virtual environment Python (`venv/bin/python3`) if available
  - Falls back to system `python3` if venv not found

#### Frontend Changes
- **New Button**: Green "Fetch Latest Data" button with loading state
- **Success Messages**: Displays detailed output from the data collection script
- **Auto-Refresh**: Automatically reloads expiration dates after successful data fetch

### Example Usage

```bash
# Via API
curl -X POST http://localhost:8080/api/fetch-data \
  -H 'Content-Type: application/json' \
  -d '{"ticker":"GOOGL"}'

# Via Web Interface
1. Open http://localhost:8080
2. Enter ticker: GOOGL
3. Click "📥 Fetch Latest Data"
4. Wait for success message
5. Select expiration date
6. Click "📊 Analyze Options"
```

### Files Created
When you fetch data for a ticker, the following files are created:
- `{TICKER}_{EXPIRATION}_{TIMESTAMP}_calls.csv` - Call options chain
- `{TICKER}_{EXPIRATION}_{TIMESTAMP}_puts.csv` - Put options chain
- `{TICKER}_{EXPIRATION}_{TIMESTAMP}_black_scholes_params.csv` - Model parameters

### Requirements
- Python 3 with yfinance, pandas, numpy installed
- Virtual environment at `venv/` (recommended)
- Internet connection to fetch data from Yahoo Finance

### Troubleshooting

**"ModuleNotFoundError: No module named 'yfinance'"**
- Solution: Activate the virtual environment and install dependencies:
  ```bash
  source venv/bin/activate
  pip install yfinance pandas numpy
  ```

**"Failed to fetch data"**
- Check that the Python script is in the project root
- Verify internet connectivity
- Check server logs for detailed error messages

### Server Logs
The server prints detailed logs when fetching data:
```
[FETCH] Downloading options data for MSFT...
[FETCH] Successfully downloaded data for MSFT
```

Check these logs in the terminal running the web server for debugging.
