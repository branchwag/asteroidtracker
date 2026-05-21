use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::api::get_neo_data;
use crate::components::{Cell, ImpactTable, StarryBackground};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="color-scheme" content="dark" />
                <meta name="theme-color" content="#000000" />
                <link rel="icon" href="data:," />
                <style>"html,body{background:#000;color:#ededed;margin:0}"</style>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/asteroidtracker.css" />
        <Link
            rel="stylesheet"
            href="https://fonts.googleapis.com/css2?family=Special+Elite&display=swap"
        />
        <Title text="Near Earth Objects" />
        <Router>
            <main>
                <Routes fallback=|| view! { <p>"Not found."</p> }>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let neo_data = Resource::new(|| (), |_| async move { get_neo_data().await });

    view! {
        <main class="page">
            <StarryBackground />
            <div class="z-10 text-white typewriter-font content">
                <h1 class="text-4xl font-bold mb-8 typewriter-font">"Asteroid Tracker"</h1>
                <p class="text-xl typewriter-font">"Welcome to the universe!"</p>
                <p class="text-sm mt-4 typewriter-font">
                    "Near-earth objects are listed below. All data is from "
                    <a
                        href="https://api.nasa.gov/"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="underline text-blue-400 hover-text-blue-200"
                    >
                        "https://api.nasa.gov/"
                    </a>
                </p>

                <Suspense fallback=|| view! {
                    <p class="mt-6 text-center typewriter-font">"Loading asteroid data..."</p>
                }>
                    {move || Suspend::new(async move {
                        match neo_data.await {
                            Ok(neos) if !neos.is_empty() => {
                                let headers = vec![
                                    "Name".to_string(),
                                    "Approach Date (UTC)".to_string(),
                                    "Diameter (m)".to_string(),
                                    "Velocity (km/h)".to_string(),
                                    "Miss Distance (LD)".to_string(),
                                    "Hazardous".to_string(),
                                ];
                                let rows: Vec<Vec<Cell>> = neos
                                    .iter()
                                    .map(|obj| {
                                        let dmin = obj
                                            .estimated_diameter
                                            .meters
                                            .estimated_diameter_min
                                            .round() as i64;
                                        let dmax = obj
                                            .estimated_diameter
                                            .meters
                                            .estimated_diameter_max
                                            .round() as i64;
                                        let ca = obj.close_approach_data.first();
                                        let approach_display = ca
                                            .map(|c| c.close_approach_date_full.clone())
                                            .unwrap_or_default();
                                        let approach_epoch = ca
                                            .map(|c| c.epoch_date_close_approach)
                                            .unwrap_or(0);
                                        let velocity = ca
                                            .and_then(|c| {
                                                c.relative_velocity
                                                    .kilometers_per_hour
                                                    .parse::<f64>()
                                                    .ok()
                                            })
                                            .unwrap_or(0.0);
                                        let miss_lunar = ca
                                            .and_then(|c| c.miss_distance.lunar.parse::<f64>().ok())
                                            .unwrap_or(0.0);
                                        let hazardous_display = if obj
                                            .is_potentially_hazardous_asteroid
                                        {
                                            "YES"
                                        } else {
                                            "No"
                                        };
                                        let hazardous_key = if obj
                                            .is_potentially_hazardous_asteroid
                                        {
                                            1.0
                                        } else {
                                            0.0
                                        };
                                        vec![
                                            Cell::text(obj.name.clone()),
                                            Cell::number(approach_display, approach_epoch as f64),
                                            Cell::number(
                                                format!("{dmin} - {dmax}"),
                                                dmin as f64,
                                            ),
                                            Cell::number(
                                                format_with_commas(velocity),
                                                velocity,
                                            ),
                                            Cell::number(format!("{miss_lunar:.2}"), miss_lunar),
                                            Cell::number(hazardous_display, hazardous_key),
                                        ]
                                    })
                                    .collect();
                                view! {
                                    <div class="mt-6">
                                        <ImpactTable headers=headers rows=rows />
                                    </div>
                                }
                                .into_any()
                            }
                            Ok(_) => view! {
                                <p class="mt-6 text-center text-red-500 typewriter-font">
                                    "Error loading asteroid data. Please try again later."
                                </p>
                            }
                            .into_any(),
                            Err(_) => view! {
                                <p class="mt-6 text-center text-red-500 typewriter-font">
                                    "Error loading asteroid data. Please try again later."
                                </p>
                            }
                            .into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </main>
    }
}

fn format_with_commas(value: f64) -> String {
    let formatted = format!("{value:.2}");
    let (int_part, frac_part) = formatted.split_once('.').unwrap_or((&formatted, ""));
    let mut with_commas = String::new();
    for (i, ch) in int_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            with_commas.insert(0, ',');
        }
        with_commas.insert(0, ch);
    }
    if frac_part.is_empty() {
        with_commas
    } else {
        format!("{with_commas}.{frac_part}")
    }
}
