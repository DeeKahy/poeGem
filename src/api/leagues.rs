use axum::{extract::State, http::StatusCode, response::Json};
use tracing::{error, info, warn};

use crate::{
    models::{League, LeaguesApiResponse, OfficialLeague},
    AppState,
};

/// Fetches available leagues from the official Path of Exile API and caches them.
/// 
/// This endpoint dynamically retrieves the current leagues from GGG's official API instead of poe.ninja,
/// which automatically includes new challenge leagues as they become available.
/// Results are lazily cached for 1 hour to reduce API calls.
/// 
/// Only "main" leagues relevant for economy tracking are returned (excludes SSF, and old leagues).
/// 
/// If the official API is unavailable, falls back to the permanent leagues
/// (Standard and Hardcore) which always exist.
pub async fn get_leagues(State(state): State<AppState>) -> Result<Json<LeaguesApiResponse>, StatusCode> {
    let cache_key = "leagues";

    // Try to get from cache first
    if let Ok(Some(cached_leagues)) = state.cache.get::<LeaguesApiResponse>(cache_key).await {
        info!("Returning cached leagues data");
        return Ok(Json(cached_leagues));
    }

    // Fetch fresh data from official PoE API
    info!("Fetching fresh leagues data from official PoE API");

    let url = "https://api.pathofexile.com/leagues?type=main&realm=pc";
    let response = match state.client.get(url).send().await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to fetch leagues from PoE API: {}", e);
            info!("Using fallback leagues due to network error");
            return Ok(Json(get_fallback_leagues()));
        }
    };

    // Check response status
    if !response.status().is_success() {
        warn!("PoE API returned non-success status: {}", response.status());
        info!("Using fallback leagues due to API error");
        return Ok(Json(get_fallback_leagues()));
    }

    // Get response body as text first for better error diagnostics
    let body_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to read response body: {}", e);
            info!("Using fallback leagues");
            return Ok(Json(get_fallback_leagues()));
        }
    };

    // Check for empty response
    if body_text.is_empty() {
        warn!("PoE API returned empty response body");
        info!("Using fallback leagues");
        return Ok(Json(get_fallback_leagues()));
    }

    // Parse the JSON - official API returns an array of leagues
    let official_leagues: Vec<OfficialLeague> = match serde_json::from_str(&body_text) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse leagues response: {}. Body preview: {}", e, &body_text.chars().take(200).collect::<String>());
            info!("Using fallback leagues due to parse error");
            return Ok(Json(get_fallback_leagues()));
        }
    };

    // Convert official leagues to our format, filtering for economy-relevant leagues
    // We exclude SSF, Ruthless, and other variants that POE Ninja doesn't track
    let leagues: Vec<League> = official_leagues
        .into_iter()
        .filter(|league| is_economy_league(league))
        .map(|league| {
            let is_hardcore = league.rules
                .as_ref()
                .map(|rules| rules.iter().any(|r| r.id == "Hardcore"))
                .unwrap_or(false);
            
            League {
                name: league.id.clone(),
                display_name: Some(league.id),
                hardcore: is_hardcore,
                indexed: true,
            }
        })
        .collect();

    let api_response = LeaguesApiResponse { leagues };

    // Cache the response for 1 hour
    if let Err(e) = state.cache.set(cache_key, &api_response, 60).await {
        error!("Failed to cache leagues data: {}", e);
        // Continue anyway, don't fail the request, since we will still have the cached data.
    }

    info!("Successfully fetched and cached {} leagues", api_response.leagues.len());
    Ok(Json(api_response))
}

/// Determines if a league is relevant for economy tracking on POE Ninja.
/// Excludes SSF, Ruthless, and other variants that don't have economy data.
fn is_economy_league(league: &OfficialLeague) -> bool {
    let id = &league.id;
    
    // Exclude SSF leagues
    if id.contains("SSF") || id.contains("Solo Self-Found") {
        return false;
    }
    
    // Exclude Ruthless leagues
    if id.contains("Ruthless") {
        return false;
    }
    
    // Include Standard, Hardcore, and challenge leagues
    true
}

/// Fallback leagues used only when the official PoE API is unavailable.
/// The actual leagues are dynamically fetched from https://api.pathofexile.com/leagues
/// and cached for 1 hour. This fallback provides the permanent leagues that always exist.
fn get_fallback_leagues() -> LeaguesApiResponse {
    LeaguesApiResponse {
        leagues: vec![
            League {
                name: "Standard".to_string(),
                display_name: Some("Standard".to_string()),
                hardcore: false,
                indexed: true,
            },
            League {
                name: "Hardcore".to_string(),
                display_name: Some("Hardcore".to_string()),
                hardcore: true,
                indexed: true,
            },
        ],
    }
}
