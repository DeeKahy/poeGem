use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct League {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub hardcore: bool,
    pub indexed: bool,
}

/// Response from the official PoE API for leagues
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OfficialLeague {
    pub id: String,
    pub realm: Option<String>,
    #[serde(rename = "endAt")]
    pub end_at: Option<String>,
    pub rules: Option<Vec<LeagueRule>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeagueRule {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaguesResponse {
    #[serde(rename = "economyLeagues")]
    pub economy_leagues: Option<Vec<League>>,
    #[serde(rename = "oldEconomyLeagues")]
    pub old_economy_leagues: Option<Vec<League>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaguesApiResponse {
    pub leagues: Vec<League>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillGem {
    pub id: Option<u32>,
    pub name: String,
    #[serde(rename = "chaosValue")]
    pub chaos_value: Option<f64>,
    #[serde(rename = "exaltedValue")]
    pub exalted_value: Option<f64>,
    #[serde(rename = "divineValue")]
    pub divine_value: Option<f64>,
    pub count: Option<u32>,
    #[serde(rename = "detailsId")]
    pub details_id: Option<String>,
    #[serde(rename = "tradeFilter")]
    pub trade_filter: Option<serde_json::Value>,
    pub corrupted: Option<bool>,
    #[serde(rename = "gemLevel")]
    pub gem_level: Option<u32>,
    #[serde(rename = "gemQuality")]
    pub gem_quality: Option<u32>,
    pub icon: Option<String>,
    #[serde(rename = "mapTier")]
    pub map_tier: Option<u32>,
    #[serde(rename = "levelRequired")]
    pub level_required: Option<u32>,
    #[serde(rename = "baseType")]
    pub base_type: Option<String>,
    #[serde(rename = "stackSize")]
    pub stack_size: Option<u32>,
    pub variant: Option<String>,
    #[serde(rename = "prophecyLink")]
    pub prophecy_link: Option<String>,
    #[serde(rename = "prophecyText")]
    pub prophecy_text: Option<String>,
    #[serde(rename = "artFilename")]
    pub art_filename: Option<String>,
    pub links: Option<u32>,
    #[serde(rename = "itemClass")]
    pub item_class: Option<u32>,
    #[serde(rename = "sparkline")]
    pub sparkline: Option<Sparkline>,
    #[serde(rename = "lowConfidenceSparkline")]
    pub low_confidence_sparkline: Option<Sparkline>,
    pub implicitModifiers: Option<Vec<ImplicitModifier>>,
    pub explicitModifiers: Option<Vec<ExplicitModifier>>,
    pub flavourText: Option<String>,
    #[serde(rename = "tradeInfo")]
    pub trade_info: Option<Vec<serde_json::Value>>,
    #[serde(rename = "listingCount")]
    pub listing_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sparkline {
    pub data: Option<Vec<Option<f64>>>,
    #[serde(rename = "totalChange")]
    pub total_change: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImplicitModifier {
    pub text: String,
    pub optional: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExplicitModifier {
    pub text: String,
    pub optional: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillGemResponse {
    pub lines: Vec<SkillGem>,
    #[serde(rename = "currencyDetails")]
    pub currency_details: Option<Vec<CurrencyDetail>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencyDetail {
    pub id: Option<u32>,
    pub icon: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "tradeId")]
    pub trade_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GemColorsResponse {
    pub red: Vec<String>,
    pub green: Vec<String>,
    pub blue: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculationRequest {
    pub league: Option<String>,
    pub ignore_after_chaos: Option<f64>,
    pub gem_level: Option<u32>,
    pub gem_quality: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculationResponse {
    pub red_roi: f64,
    pub green_roi: f64,
    pub blue_roi: f64,
    pub red_gems: Vec<GemValue>,
    pub green_gems: Vec<GemValue>,
    pub blue_gems: Vec<GemValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GemValue {
    pub name: String,
    pub chaos_value: f64,
    pub probability: f64,
}

#[derive(Debug, Clone)]
pub enum GemColor {
    Red,
    Green,
    Blue,
}

impl GemColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            GemColor::Red => "red",
            GemColor::Green => "green",
            GemColor::Blue => "blue",
        }
    }

    /// Extracts gem color from the icon URL's encoded `gd` (gem display) parameter.
    /// 
    /// The poe.ninja icon URLs contain base64-encoded JSON with a `gd` field:
    /// - gd 5, 6: Red (Strength) gems
    /// - gd 9, 10: Green (Dexterity) gems  
    /// - gd 13, 14: Blue (Intelligence) gems
    pub fn from_icon_url(icon_url: &str) -> Option<GemColor> {
        // Extract base64 segment: https://web.poecdn.com/gen/image/{base64}/hash/name.png
        let encoded = icon_url
            .split("/image/")
            .nth(1)?
            .split('/')
            .next()?;

        let decoded = base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(encoded)
            .ok()?;
        let json_str = String::from_utf8(decoded).ok()?;
        
        // Parse the gd value from JSON like: [30,14,{"f":"...","gd":5}]
        // Simple extraction without full JSON parsing
        let gd_value = json_str
            .split("\"gd\":")
            .nth(1)?
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .ok()?;

        match gd_value {
            5 | 6 => Some(GemColor::Red),
            9 | 10 => Some(GemColor::Green),
            13 | 14 => Some(GemColor::Blue),
            _ => None,
        }
    }
}

/// Check if a gem name looks like a transfigured gem (contains " of ")
pub fn is_transfigured_gem(gem_name: &str) -> bool {
    gem_name.contains(" of ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gem_color_from_icon_url() {
        // Red gem (gd=5): Molten Strike
        let red_url = "https://web.poecdn.com/gen/image/WzMwLDE0LHsiZiI6IjJESXRlbXMvR2Vtcy9Nb2x0ZW5TdHJpa2UiLCJ3IjoxLCJoIjoxLCJzY2FsZSI6MSwiZ2QiOjV9XQ/b3e2ef9d6c/MoltenStrike.png";
        assert!(matches!(GemColor::from_icon_url(red_url), Some(GemColor::Red)));

        // Green gem (gd=9): Viper Strike
        let green_url = "https://web.poecdn.com/gen/image/WzMwLDE0LHsiZiI6IjJESXRlbXMvR2Vtcy9WaXBlclN0cmlrZSIsInciOjEsImgiOjEsInNjYWxlIjoxLCJnZCI6OX1d/8b37a8f02e/ViperStrike.png";
        assert!(matches!(GemColor::from_icon_url(green_url), Some(GemColor::Green)));

        // Blue gem (gd=14): Spark
        let blue_url = "https://web.poecdn.com/gen/image/WzMwLDE0LHsiZiI6IjJESXRlbXMvR2Vtcy9TcGFyayIsInciOjEsImgiOjEsInNjYWxlIjoxLCJnZCI6MTR9XQ/c9038eb883/Spark.png";
        assert!(matches!(GemColor::from_icon_url(blue_url), Some(GemColor::Blue)));

        // No gd field (awakened support gem) - should return None
        let support_url = "https://web.poecdn.com/gen/image/WzI1LDE0LHsiZiI6IjJESXRlbXMvR2Vtcy9TdXBwb3J0L1N1cHBvcnRQbHVzL0luY3JlYXNlZEFPRVBsdXMiLCJ3IjoxLCJoIjoxLCJzY2FsZSI6MX1d/360e9e4ed5/IncreasedAOEPlus.png";
        assert!(GemColor::from_icon_url(support_url).is_none());
    }

    #[test]
    fn test_is_transfigured_gem() {
        assert!(is_transfigured_gem("Spark of Unpredictability"));
        assert!(is_transfigured_gem("Molten Strike of the Zenith"));
        assert!(!is_transfigured_gem("Spark"));
        assert!(!is_transfigured_gem("Awakened Multistrike Support"));
    }
}