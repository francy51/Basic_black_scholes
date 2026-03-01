/// Visualization module for options pricing using plotters
use plotters::prelude::*;
use plotters::style::IntoFont;
use std::error::Error;
use std::f64;

/// Chart data point for visualization
#[derive(Debug, Clone)]
pub struct ChartPoint {
    pub strike: f64,
    pub market_price: f64,
    pub theoretical_price: f64,
    pub implied_volatility: f64,
    pub volume: Option<f64>,
}

/// Configuration for generating visualizations
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub ticker: String,
    pub current_price: f64,
    pub time_to_expiry: f64,
    pub historical_volatility: Option<f64>,
}

/// Generate a volatility smile/skew chart
pub fn plot_volatility_smile(
    data: &[ChartPoint],
    config: &VisualizationConfig,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_iv = data.iter().map(|d| d.implied_volatility).fold(f64::INFINITY, f64::min);
    let max_iv = data.iter().map(|d| d.implied_volatility).fold(f64::NEG_INFINITY, f64::max);
    let min_strike = data.iter().map(|d| d.strike).fold(f64::INFINITY, f64::min);
    let max_strike = data.iter().map(|d| d.strike).fold(f64::NEG_INFINITY, f64::max);

    let iv_range = max_iv - min_iv;
    let strike_range = max_strike - min_strike;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} Volatility Smile/Skew", config.ticker),
            ("sans-serif", 30).into_font(),
        )
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (min_strike - strike_range * 0.05)..(max_strike + strike_range * 0.05),
            (min_iv - iv_range * 0.1)..(max_iv + iv_range * 0.1),
        )?;

    chart
        .configure_mesh()
        .x_desc("Strike Price ($)")
        .y_desc("Implied Volatility")
        .x_labels(10)
        .y_labels(10)
        .draw()?;

    // Connect points with line (sorted)
    let sorted_data: Vec<_> = {
        let mut d = data.to_vec();
        d.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap());
        d
    };

    // Draw IV line with legend
    chart.draw_series(LineSeries::new(
        sorted_data.iter().map(|d| (d.strike, d.implied_volatility)),
        RED.stroke_width(2),
    ))?
    .label("Implied Volatility")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));

    // Draw ATM line with legend
    chart.draw_series(LineSeries::new(
        vec![(config.current_price, min_iv - iv_range * 0.1),
             (config.current_price, max_iv + iv_range * 0.1)],
        BLACK.stroke_width(2).filled(),
    ))?
    .label("ATM Strike")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(2)));

    // Draw historical volatility line if available
    if let Some(hv) = config.historical_volatility {
        chart.draw_series(LineSeries::new(
            vec![(min_strike - strike_range * 0.05, hv),
                 (max_strike + strike_range * 0.05, hv)],
            GREEN.stroke_width(2),
        ))?
        .label("Historical Volatility")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.stroke_width(2)));
    }

    // Add legend
    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 15))
        .draw()?;

    root.present()?;

    println!("Volatility smile chart saved to: {}", output_path);
    Ok(())
}

/// Generate a market vs theoretical price comparison chart
pub fn plot_price_comparison(
    data: &[ChartPoint],
    config: &VisualizationConfig,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let all_prices: Vec<f64> = data.iter()
        .flat_map(|d| vec![d.market_price, d.theoretical_price])
        .collect();

    let min_price = all_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_price = all_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let min_strike = data.iter().map(|d| d.strike).fold(f64::INFINITY, f64::min);
    let max_strike = data.iter().map(|d| d.strike).fold(f64::NEG_INFINITY, f64::max);

    let price_range = max_price - min_price;
    let strike_range = max_strike - min_strike;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} Market vs Theoretical Price", config.ticker),
            ("sans-serif", 30).into_font(),
        )
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (min_strike - strike_range * 0.05)..(max_strike + strike_range * 0.05),
            (0.0_f64)..(max_price + price_range * 0.1),
        )?;

    chart
        .configure_mesh()
        .x_desc("Strike Price ($)")
        .y_desc("Option Price ($)")
        .x_labels(10)
        .y_labels(10)
        .draw()?;

    // Sort data by strike
    let sorted_data: Vec<_> = {
        let mut d = data.to_vec();
        d.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap());
        d
    };

    // Draw market price line
    chart.draw_series(LineSeries::new(
        sorted_data.iter().map(|d| (d.strike, d.market_price)),
        BLUE.stroke_width(2),
    ))?
    .label("Market Price")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));

    // Draw theoretical price line
    chart.draw_series(LineSeries::new(
        sorted_data.iter().map(|d| (d.strike, d.theoretical_price)),
        RED.stroke_width(2),
    ))?
    .label("Theoretical Price (BS)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));

    // Draw ATM line
    chart.draw_series(LineSeries::new(
        vec![(config.current_price, 0.0),
             (config.current_price, max_price + price_range * 0.1)],
        BLACK.stroke_width(2).filled(),
    ))?
    .label("ATM Strike")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(2)));

    // Add legend
    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 15))
        .draw()?;

    root.present()?;

    println!("Price comparison chart saved to: {}", output_path);
    Ok(())
}

/// Generate a price difference chart showing mispricing
pub fn plot_price_difference(
    data: &[ChartPoint],
    config: &VisualizationConfig,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let differences: Vec<f64> = data.iter()
        .map(|d| d.market_price - d.theoretical_price)
        .collect();

    let min_diff = differences.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_diff = differences.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let min_strike = data.iter().map(|d| d.strike).fold(f64::INFINITY, f64::min);
    let max_strike = data.iter().map(|d| d.strike).fold(f64::NEG_INFINITY, f64::max);

    let diff_range = max_diff - min_diff;
    let strike_range = max_strike - min_strike;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} Price Difference (Market - Theoretical)", config.ticker),
            ("sans-serif", 30).into_font(),
        )
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (min_strike - strike_range * 0.05)..(max_strike + strike_range * 0.05),
            (min_diff - diff_range * 0.1)..(max_diff + diff_range * 0.1),
        )?;

    chart
        .configure_mesh()
        .x_desc("Strike Price ($)")
        .y_desc("Price Difference ($)")
        .x_labels(10)
        .y_labels(10)
        .draw()?;

    // Draw zero line
    let y_min = min_diff - diff_range * 0.1;
    let y_max = max_diff + diff_range * 0.1;
    chart.draw_series(LineSeries::new(
        vec![(min_strike - strike_range * 0.05, 0.0),
             (max_strike + strike_range * 0.05, 0.0)],
        BLACK.stroke_width(1),
    ))?
    .label("Fair Value (Zero)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(1)));

    // Draw ATM line
    chart.draw_series(LineSeries::new(
        vec![(config.current_price, y_min),
             (config.current_price, y_max)],
        BLACK.stroke_width(2).filled(),
    ))?
    .label("ATM Strike")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(2)));

    // Sort data by strike
    let mut sorted_with_diff: Vec<_> = data.iter()
        .zip(differences.iter())
        .map(|(d, &diff)| (d.strike, diff))
        .collect();
    sorted_with_diff.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Draw bar-like chart using lines (green for positive, red for negative)
    for (strike, diff) in &sorted_with_diff {
        let color = if *diff >= 0.0 { &GREEN } else { &RED };
        chart.draw_series(LineSeries::new(
            vec![(*strike, 0.0), (*strike, *diff)],
            color.stroke_width(3),
        ))?;
    }

    // Draw line connecting differences
    chart.draw_series(LineSeries::new(
        sorted_with_diff.clone(),
        BLUE.stroke_width(2),
    ))?
    .label("Price Difference")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));

    // Add legend
    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 15))
        .draw()?;

    root.present()?;

    println!("Price difference chart saved to: {}", output_path);
    Ok(())
}

/// Generate a comprehensive dashboard with multiple charts
pub fn generate_dashboard(
    data: &[ChartPoint],
    config: &VisualizationConfig,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    use std::path::Path;

    // Create output directory if it doesn't exist
    let path = Path::new(output_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }

    let base_name = format!(
        "{}_T{:.2}y",
        config.ticker,
        config.time_to_expiry
    );

    // Generate all individual charts
    plot_volatility_smile(
        data,
        config,
        &format!("{}/{}_volatility_smile.png", output_dir, base_name),
    )?;

    plot_price_comparison(
        data,
        config,
        &format!("{}/{}_price_comparison.png", output_dir, base_name),
    )?;

    plot_price_difference(
        data,
        config,
        &format!("{}/{}_price_difference.png", output_dir, base_name),
    )?;

    println!("\nDashboard generated successfully in: {}", output_dir);
    println!("  - Volatility Smile/Skew chart");
    println!("  - Price Comparison chart");
    println!("  - Price Difference chart");

    Ok(())
}

/// Generate a volume/open interest chart if data is available
pub fn plot_volume_analysis(
    data: &[ChartPoint],
    config: &VisualizationConfig,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    // Filter out data without volume
    let volume_data: Vec<_> = data.iter()
        .filter(|d| d.volume.is_some())
        .collect();

    if volume_data.is_empty() {
        println!("No volume data available for chart");
        return Ok(());
    }

    let max_volume = volume_data.iter()
        .map(|d| d.volume.unwrap())
        .fold(f64::NEG_INFINITY, f64::max);

    let min_strike = data.iter().map(|d| d.strike).fold(f64::INFINITY, f64::min);
    let max_strike = data.iter().map(|d| d.strike).fold(f64::NEG_INFINITY, f64::max);
    let strike_range = max_strike - min_strike;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} Volume by Strike", config.ticker),
            ("sans-serif", 30).into_font(),
        )
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (min_strike - strike_range * 0.05)..(max_strike + strike_range * 0.05),
            0.0_f64..(max_volume * 1.1),
        )?;

    chart
        .configure_mesh()
        .x_desc("Strike Price ($)")
        .y_desc("Volume")
        .x_labels(10)
        .y_labels(10)
        .draw()?;

    // Sort data by strike
    let mut sorted_data: Vec<_> = volume_data.into_iter().collect();
    sorted_data.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap());

    // Draw ATM line
    chart.draw_series(LineSeries::new(
        vec![(config.current_price, 0.0),
             (config.current_price, max_volume * 1.1)],
        BLACK.stroke_width(2).filled(),
    ))?
    .label("ATM Strike")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(2)));

    // Draw bar-like chart using lines
    for point in &sorted_data {
        chart.draw_series(LineSeries::new(
            vec![(point.strike, 0.0), (point.strike, point.volume.unwrap())],
            BLUE.stroke_width(3),
        ))?;
    }

    // Draw line connecting volumes
    chart.draw_series(LineSeries::new(
        sorted_data.iter().map(|d| (d.strike, d.volume.unwrap())),
        BLUE.stroke_width(2),
    ))?
    .label("Volume")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));

    // Add legend
    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 15))
        .draw()?;

    root.present()?;

    println!("Volume analysis chart saved to: {}", output_path);
    Ok(())
}
