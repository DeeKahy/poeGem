use axum::{extract::{Query, State}, http::StatusCode, response::Json};
use serde::Deserialize;

use tracing::{error, info, warn};

use crate::{
    models::{SkillGemResponse, GemColor, get_gem_color, CalculationResponse, GemValue},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct SkillGemsQuery {
    league: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CalculationQuery {
    league: Option<String>,
    ignore_after_chaos: Option<f64>,
    gem_level: Option<u32>,
    gem_quality: Option<u32>,
}

pub async fn get_skill_gems(
    Query(params): Query<SkillGemsQuery>,
    State(state): State<AppState>,
) -> Result<Json<SkillGemResponse>, StatusCode> {
    let league = params.league.unwrap_or_else(|| "Standard".to_string());
    let cache_key = format!("skillGems_{}", league);

    // Try to get from cache first (TTL: 1 hour)
    if let Ok(Some(cached_gems)) = state.cache.get::<SkillGemResponse>(&cache_key).await {
        info!("Returning cached skill gems data for league: {}", league);
        return Ok(Json(cached_gems));
    }

    // Fetch fresh data from POE Ninja
    info!("Fetching fresh skill gems data for league: {}", league);

    let url = format!(
        "https://poe.ninja/api/data/itemoverview?league={}&type=SkillGem&language=en",
        urlencoding::encode(&league)
    );

    let response = match state.client.get(&url).send().await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to fetch skill gems from POE Ninja: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !response.status().is_success() {
        error!("POE Ninja returned error status: {}", response.status());
        return Err(StatusCode::BAD_GATEWAY);
    }

    let skill_gems_response: SkillGemResponse = match response.json().await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse skill gems response: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Cache the response for 1 hour
    if let Err(e) = state.cache.set(&cache_key, &skill_gems_response, 60).await {
        error!("Failed to cache skill gems data: {}", e);
        // Continue anyway, don't fail the request
    }

    info!(
        "Successfully fetched and cached {} skill gems for league: {}",
        skill_gems_response.lines.len(),
        league
    );

    Ok(Json(skill_gems_response))
}

pub async fn calculate_gem_roi(
    Query(params): Query<CalculationQuery>,
    State(state): State<AppState>,
) -> Result<Json<CalculationResponse>, StatusCode> {
    let league = params.league.unwrap_or_else(|| "Standard".to_string());
    let ignore_after_chaos = params.ignore_after_chaos.unwrap_or(5.0);
    let gem_level = params.gem_level.unwrap_or(1);
    let gem_quality = params.gem_quality.unwrap_or(0);

    info!(
        "Calculating ROI for league: {}, level: {}, quality: {}, ignore_threshold: {}",
        league, gem_level, gem_quality, ignore_after_chaos
    );

    // Get skill gems data
    let cache_key = format!("skillGems_{}", league);
    let skill_gems_response = if let Ok(Some(cached_gems)) = state.cache.get::<SkillGemResponse>(&cache_key).await {
        cached_gems
    } else {
        // Fetch fresh data if not cached
        let url = format!(
            "https://poe.ninja/api/data/itemoverview?league={}&type=SkillGem&language=en",
            urlencoding::encode(&league)
        );

        let response = match state.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to fetch skill gems from POE Ninja: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let gems_response: SkillGemResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse skill gems response: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        // Cache for future use
        if let Err(e) = state.cache.set(&cache_key, &gems_response, 60).await {
            warn!("Failed to cache skill gems data: {}", e);
        }

        gems_response
    };

    // Categorize gems by color and filter by criteria
    let mut red_gems = Vec::new();
    let mut green_gems = Vec::new();
    let mut blue_gems = Vec::new();

    for gem in skill_gems_response.lines {
        // Filter gems based on criteria
        let matches_level = gem.gem_level == Some(gem_level) || (gem_level == 1 && gem.gem_level.is_none());
        let matches_quality = if gem_quality == 0 {
            gem.gem_quality.is_none() || gem.gem_quality == Some(0)
        } else {
            gem.gem_quality == Some(gem_quality)
        };
        let matches_corruption = if gem_level > 20 || gem_quality > 20 {
            gem.corrupted == Some(true)
        } else {
            gem.corrupted.is_none() || gem.corrupted == Some(false)
        };

        // Must have trade filter to be considered
        if gem.trade_filter.is_some() && matches_level && matches_quality && matches_corruption {
            let chaos_value = gem.chaos_value.unwrap_or(0.0);

            if let Some(color) = get_gem_color(&gem.name) {
                let gem_data = (gem.name.clone(), chaos_value);
                match color {
                    GemColor::Red => red_gems.push(gem_data),
                    GemColor::Green => green_gems.push(gem_data),
                    GemColor::Blue => blue_gems.push(gem_data),
                }
            }
        }
    }

    // Sort gems by value (highest first)
    red_gems.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    green_gems.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    blue_gems.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate probabilities and ROI
    let red_roi = calculate_roi_for_gems(&red_gems, ignore_after_chaos);
    let green_roi = calculate_roi_for_gems(&green_gems, ignore_after_chaos);
    let blue_roi = calculate_roi_for_gems(&blue_gems, ignore_after_chaos);

    // Prepare detailed gem information
    let red_probabilities = calculate_probability(red_gems.len());
    let green_probabilities = calculate_probability(green_gems.len());
    let blue_probabilities = calculate_probability(blue_gems.len());

    let red_gem_values = create_gem_values(&red_gems, &red_probabilities);
    let green_gem_values = create_gem_values(&green_gems, &green_probabilities);
    let blue_gem_values = create_gem_values(&blue_gems, &blue_probabilities);

    let response = CalculationResponse {
        red_roi,
        green_roi,
        blue_roi,
        red_gems: red_gem_values,
        green_gems: green_gem_values,
        blue_gems: blue_gem_values,
    };

    info!(
        "ROI calculation complete - Red: {:.2}, Green: {:.2}, Blue: {:.2}",
        red_roi, green_roi, blue_roi
    );

    Ok(Json(response))
}

fn calculate_probability(n: usize) -> Vec<f64> {
    if n < 3 {
        return vec![];
    }

    let mut probabilities = Vec::new();
    let total_outcomes = (n * (n - 1) * (n - 2)) as f64 / 6.0;

    for i in (3..=n).rev() {
        let favorable_outcomes = ((i - 1) * (i - 2)) as f64 / 2.0;
        let probability = favorable_outcomes / total_outcomes;
        probabilities.push(probability);
    }

    probabilities
}

fn calculate_roi_for_gems(gems: &[(String, f64)], ignore_threshold: f64) -> f64 {
    if gems.len() < 3 {
        return 0.0;
    }

    let probabilities = calculate_probability(gems.len());
    let mut roi = 0.0;

    for (i, (_, value)) in gems.iter().enumerate() {
        if i >= probabilities.len() {
            break;
        }

        let gem_value = if *value >= ignore_threshold { *value } else { 0.0 };
        roi += probabilities[i] * gem_value;
    }

    roi
}

fn create_gem_values(gems: &[(String, f64)], probabilities: &[f64]) -> Vec<GemValue> {
    gems.iter()
        .zip(probabilities.iter())
        .map(|((name, chaos_value), probability)| GemValue {
            name: name.clone(),
            chaos_value: *chaos_value,
            probability: *probability,
        })
        .collect()
}
