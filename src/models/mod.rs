use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct League {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub hardcore: bool,
    pub indexed: bool,
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
}

// Predefined gem lists based on the original JavaScript
pub const RED_GEMS: &[&str] = &[
    "Absolution of Inspiring",
    "Animate Guardian of Smiting",
    "Bladestorm of Uncertainty",
    "Boneshatter of Carnage",
    "Boneshatter of Complex Trauma",
    "Cleave of Rage",
    "Consecrated Path of Endurance",
    "Dominating Blow of Inspiring",
    "Earthquake of Amplification",
    "Earthshatter of Fragility",
    "Earthshatter of Prominence",
    "Exsanguinate of Transmission",
    "Frozen Legion of Rallying",
    "Glacial Hammer of Shattering",
    "Ground Slam of Earthshaking",
    "Holy Flame Totem of Ire",
    "Ice Crash of Cadence",
    "Infernal Blow of Immolation",
    "Leap Slam of Groundbreaking",
    "Molten Strike of the Zenith",
    "Perforate of Bloodshed",
    "Perforate of Duality",
    "Rage Vortex of Berserking",
    "Shield Crush of the Chieftain",
    "Smite of Divine Judgement",
    "Summon Flame Golem of Hordes",
    "Summon Flame Golem of the Meteor",
    "Summon Stone Golem of Hordes",
    "Summon Stone Golem of Safeguarding",
    "Sunder of Earthbreaking",
    "Tectonic Slam of Cataclysm",
    "Volcanic Fissure of Snaking",
];

pub const GREEN_GEMS: &[&str] = &[
    "Animate Weapon of Ranged Arms",
    "Animate Weapon of Self Reflection",
    "Artillery Ballista of Cross Strafe",
    "Artillery Ballista of Focus Fire",
    "Barrage of Volley Fire",
    "Bear Trap of Skewers",
    "Blade Blast of Dagger Detonation",
    "Blade Blast of Unloading",
    "Blade Flurry of Incision",
    "Blade Trap of Greatswords",
    "Blade Trap of Laceration",
    "Blade Vortex of the Scythe",
    "Bladefall of Impaling",
    "Bladefall of Volleys",
    "Blink Arrow of Bombarding Clones",
    "Blink Arrow of Prismatic Clones",
    "Burning Arrow of Vigour",
    "Caustic Arrow of Poison",
    "Cremation of Exhuming",
    "Cremation of the Volcano",
    "Cyclone of Tumult",
    "Detonate Dead of Chain Reaction",
    "Detonate Dead of Scavenging",
    "Double Strike of Impaling",
    "Double Strike of Momentum",
    "Dual Strike of Ambidexterity",
    "Elemental Hit of the Spectrum",
    "Ethereal Knives of Lingering Blades",
    "Ethereal Knives of the Massacre",
    "Explosive Concoction of Destruction",
    "Explosive Trap of Magnitude",
    "Explosive Trap of Shrapnel",
    "Fire Trap of Blasting",
    "Flicker Strike of Power",
    "Frenzy of Onslaught",
    "Frost Blades of Katabasis",
    "Galvanic Arrow of Energy",
    "Galvanic Arrow of Surging",
    "Ice Shot of Penetration",
    "Ice Trap of Hollowness",
    "Lacerate of Butchering",
    "Lacerate of Haemorrhage",
    "Lancing Steel of Spraying",
    "Lightning Arrow of Electrocution",
    "Lightning Strike of Arcing",
    "Mirror Arrow of Bombarding Clones",
    "Mirror Arrow of Prismatic Clones",
    "Poisonous Concoction of Bouncing",
    "Rain of Arrows of Artillery",
    "Rain of Arrows of Saturation",
    "Reave of Refraction",
    "Scourge Arrow of Menace",
    "Seismic Trap of Swells",
    "Shattering Steel of Ammunition",
    "Shrapnel Ballista of Steel",
    "Siege Ballista of Splintering",
    "Spectral Shield Throw of Shattering",
    "Spectral Throw of Materialising",
    "Split Arrow of Splitting",
    "Splitting Steel of Ammunition",
    "Storm Rain of the Conduit",
    "Storm Rain of the Fence",
    "Summon Ice Golem of Hordes",
    "Summon Ice Golem of Shattering",
    "Tornado of Elemental Turbulence",
    "Tornado Shot of Cloudburst",
    "Toxic Rain of Sporeburst",
    "Toxic Rain of Withering",
    "Viper Strike of the Mamba",
    "Volatile Dead of Confinement",
    "Volatile Dead of Seething",
    "Wild Strike of Extremes",
];

pub const BLUE_GEMS: &[&str] = &[
    "Arc of Oscillating",
    "Arc of Surging",
    "Armageddon Brand of Recall",
    "Armageddon Brand of Volatility",
    "Ball Lightning of Orbiting",
    "Ball Lightning of Static",
    "Bane of Condemnation",
    "Blight of Atrophy",
    "Blight of Contagion",
    "Bodyswap of Sacrifice",
    "Cold Snap of Power",
    "Contagion of Subsiding",
    "Contagion of Transference",
    "Crackling Lance of Branching",
    "Crackling Lance of Disintegration",
    "Discharge of Misery",
    "Divine Ire of Disintegration",
    "Divine Ire of Holy Lightning",
    "Essence Drain of Desperation",
    "Essence Drain of Wickedness",
    "Eye of Winter of Finality",
    "Eye of Winter of Transience",
    "Firestorm of Meteors",
    "Firestorm of Pelting",
    "Flame Dash of Return",
    "Flame Surge of Combusting",
    "Flameblast of Celerity",
    "Flameblast of Contraction",
    "Forbidden Rite of Soul Sacrifice",
    "Frost Bomb of Forthcoming",
    "Frost Bomb of Instability",
    "Frostblink of Wintry Blast",
    "Galvanic Field of Intensity",
    "Glacial Cascade of the Fissure",
    "Hexblast of Contradiction",
    "Hexblast of Havoc",
    "Ice Nova of Deep Freeze",
    "Ice Nova of Frostbolts",
    "Ice Spear of Splitting",
    "Icicle Mine of Fanning",
    "Icicle Mine of Sabotage",
    "Incinerate of Expanse",
    "Incinerate of Venting",
    "Kinetic Blast of Clustering",
    "Kinetic Bolt of Fragmentation",
    "Lightning Conduit of the Heavens",
    "Lightning Spire Trap of Overloading",
    "Lightning Spire Trap of Zapping",
    "Lightning Tendrils of Eccentricity",
    "Lightning Tendrils of Escalation",
    "Lightning Trap of Sparking",
    "Penance Brand of Conduction",
    "Penance Brand of Dissipation",
    "Power Siphon of the Archmage",
    "Purifying Flame of Revelations",
    "Pyroclast Mine of Sabotage",
    "Raise Spectre of Transience",
    "Raise Zombie of Falling",
    "Raise Zombie of Slamming",
    "Righteous Fire of Arcane Devotion",
    "Scorching Ray of Immolation",
    "Soulrend of Reaping",
    "Soulrend of the Spiral",
    "Spark of the Nova",
    "Spark of Unpredictability",
    "Storm Brand of Indecision",
    "Stormbind of Teleportation",
    "Summon Carrion Golem of Hordes",
    "Summon Carrion Golem of Scavenging",
    "Summon Chaos Golem of Hordes",
    "Summon Chaos Golem of the Maelström",
    "Summon Holy Relic of Conviction",
    "Summon Lightning Golem of Hordes",
    "Summon Raging Spirit of Enormity",
    "Summon Reaper of Eviscerating",
    "Summon Reaper of Revenants",
    "Summon Skeletons of Archers",
    "Summon Skeletons of Mages",
    "Void Sphere of Rending",
    "Vortex of Projection",
];

pub fn get_gem_color(gem_name: &str) -> Option<GemColor> {
    if RED_GEMS.contains(&gem_name) {
        Some(GemColor::Red)
    } else if GREEN_GEMS.contains(&gem_name) {
        Some(GemColor::Green)
    } else if BLUE_GEMS.contains(&gem_name) {
        Some(GemColor::Blue)
    } else {
        None
    }
}

pub fn get_gems_by_color(color: &GemColor) -> &'static [&'static str] {
    match color {
        GemColor::Red => RED_GEMS,
        GemColor::Green => GREEN_GEMS,
        GemColor::Blue => BLUE_GEMS,
    }
}
