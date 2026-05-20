use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub relative_velocity: RelativeVelocity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelativeVelocity {
    pub kilometers_per_hour: String,
}

#[server(GetNeoData, "/api")]
pub async fn get_neo_data() -> Result<Vec<NearEarthObject>, ServerFnError> {
    let api_key = std::env::var("NASA_API_KEY").unwrap_or_else(|_| "DEMO_KEY".to_string());
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();

    let url = format!(
        "https://api.nasa.gov/neo/rest/v1/feed?start_date={today}&end_date={today}&api_key={api_key}"
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

    let neos = body
        .get("near_earth_objects")
        .and_then(|v| v.get(&today))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect::<Vec<NearEarthObject>>()
        })
        .unwrap_or_default();

    Ok(neos)
}
