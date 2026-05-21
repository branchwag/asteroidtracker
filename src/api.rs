use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

#[cfg(feature = "ssr")]
static CACHE: LazyLock<Mutex<HashMap<String, Vec<NearEarthObject>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NearEarthObject {
    pub name: String,
    pub estimated_diameter: EstimatedDiameter,
    pub close_approach_data: Vec<CloseApproach>,
    pub is_potentially_hazardous_asteroid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstimatedDiameter {
    pub meters: DiameterRange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiameterRange {
    pub estimated_diameter_min: f64,
    pub estimated_diameter_max: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloseApproach {
    pub close_approach_date: String,
    pub close_approach_date_full: String,
    pub epoch_date_close_approach: i64,
    pub relative_velocity: RelativeVelocity,
    pub miss_distance: MissDistance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelativeVelocity {
    pub kilometers_per_hour: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissDistance {
    pub lunar: String,
}

#[server(GetNeoData, "/api")]
pub async fn get_neo_data() -> Result<Vec<NearEarthObject>, ServerFnError> {
    let today = chrono::Local::now().date_naive();
    let end = today + chrono::Duration::days(7);
    let today_str = today.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    if let Some(cached) = CACHE.lock().unwrap().get(&today_str).cloned() {
        return Ok(cached);
    }

    let api_key = std::env::var("NASA_API_KEY").unwrap_or_else(|_| "DEMO_KEY".to_string());
    let url = format!(
        "https://api.nasa.gov/neo/rest/v1/feed?start_date={today_str}&end_date={end_str}&api_key={api_key}"
    );

    let res = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("NASA request failed: {e}")))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "NASA API returned {}",
            res.status()
        )));
    }

    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;

    let mut neos: Vec<NearEarthObject> = body
        .get("near_earth_objects")
        .and_then(|v| v.as_object())
        .map(|map| {
            map.values()
                .filter_map(|day| day.as_array())
                .flatten()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    neos.sort_by_key(|neo| {
        neo.close_approach_data
            .first()
            .map(|ca| ca.epoch_date_close_approach)
            .unwrap_or(i64::MAX)
    });

    if !neos.is_empty() {
        CACHE.lock().unwrap().insert(today_str, neos.clone());
    }

    Ok(neos)
}
