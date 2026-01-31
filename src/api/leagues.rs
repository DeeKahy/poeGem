use axum::{extract::State, http::StatusCode, response::Json};
use tracing::{error, info};

use crate::{
    models::{LeaguesApiResponse, LeaguesResponse},
    AppState,
};

pub async fn get_leagues(State(state): State<AppState>) -> Result<Json<LeaguesApiResponse>, StatusCode> {
    let cache_key = "leagues";

    // Try to get from cache first
    if let Ok(Some(cached_leagues)) = state.cache.get::<LeaguesApiResponse>(cache_key).await {
        info!("Returning cached leagues data");
        return Ok(Json(cached_leagues));
    }

    // Fetch fresh data from POE Ninja
    info!("Fetching fresh leagues data from POE Ninja");

    let url = "https://poe.ninja/api/data/getindexstate";
    let response = match state.client.get(url).send().await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to fetch leagues from POE Ninja: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let leagues_response: LeaguesResponse = match response.json().await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse leagues response: {}", e);
            // Fallback to hardcoded leagues when API is unavailable
            info!("Using fallback hardcoded leagues");
            return Ok(Json(get_fallback_leagues()));
        }
    };

    // Combine economy leagues and old economy leagues
    let mut all_leagues = Vec::new();

    if let Some(economy_leagues) = leagues_response.economy_leagues {
        all_leagues.extend(economy_leagues);
    }

    if let Some(old_economy_leagues) = leagues_response.old_economy_leagues {
        all_leagues.extend(old_economy_leagues);
    }

    let api_response = LeaguesApiResponse {
        leagues: all_leagues,
    };

    // Cache the response for 1 hour
    if let Err(e) = state.cache.set(cache_key, &api_response, 60).await {
        error!("Failed to cache leagues data: {}", e);
        // Continue anyway, don't fail the request
    }

    info!("Successfully fetched and cached {} leagues", api_response.leagues.len());
    Ok(Json(api_response))
}

fn get_fallback_leagues() -> LeaguesApiResponse {
    use crate::models::League;

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
            League {
                name: "Mercenaries".to_string(),
                display_name: Some("Mercenaries".to_string()),
                hardcore: false,
                indexed: true,
            },
            League {
                name: "Hardcore Mercenaries".to_string(),
                display_name: Some("Hardcore Mercenaries".to_string()),
                hardcore: true,
                indexed: true,
            },
            League {
                name: "Phrecia 2.0".to_string(),
                display_name: Some("Phrecia 2.0".to_string()),
                hardcore: false,
                indexed: true,
            },
            League {
                name: "Keepers".to_string(),
                display_name: Some("Keepers".to_string()),
                hardcore: false,
                indexed: true,
            },
            League {
                name: "Hardcore Settlers".to_string(),
                display_name: Some("Hardcore Settlers".to_string()),
                hardcore: true,
                indexed: true,
            },
        ],
    }
}
