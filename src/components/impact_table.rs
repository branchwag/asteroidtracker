use leptos::prelude::*;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub enum SortKey {
    Number(f64),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub display: String,
    pub sort_key: SortKey,
}

impl Cell {
    pub fn text(s: impl Into<String>) -> Self {
        let s = s.into();
        Self {
            sort_key: SortKey::Text(s.clone()),
            display: s,
        }
    }

    pub fn number(display: impl Into<String>, value: f64) -> Self {
        Self {
            display: display.into(),
            sort_key: SortKey::Number(value),
        }
    }
}

fn compare_keys(a: &SortKey, b: &SortKey) -> Ordering {
    match (a, b) {
        (SortKey::Number(x), SortKey::Number(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (SortKey::Text(x), SortKey::Text(y)) => x.to_lowercase().cmp(&y.to_lowercase()),
        (SortKey::Number(_), SortKey::Text(_)) => Ordering::Less,
        (SortKey::Text(_), SortKey::Number(_)) => Ordering::Greater,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SortDir {
    Asc,
    Desc,
}

#[component]
pub fn ImpactTable(headers: Vec<String>, rows: Vec<Vec<Cell>>) -> impl IntoView {
    let sort_state = RwSignal::new(None::<(usize, SortDir)>);
    let rows = StoredValue::new(rows);

    let sorted_rows = move || {
        let mut r = rows.get_value();
        if let Some((col, dir)) = sort_state.get() {
            r.sort_by(|a, b| {
                let cmp = compare_keys(&a[col].sort_key, &b[col].sort_key);
                if dir == SortDir::Desc { cmp.reverse() } else { cmp }
            });
        }
        r
    };

    view! {
        <div class="overflow-x-auto w-full max-w-4xl mx-auto mt-8">
            <table class="min-w-full bg-black text-white border border-gray-700 rounded-lg shadow-lg">
                <thead>
                    <tr>
                        {headers
                            .into_iter()
                            .enumerate()
                            .map(|(i, h)| {
                                view! {
                                    <th
                                        class="px-4 py-2 border-b border-gray-700 text-left sort-header"
                                        on:click=move |_| sort_state.update(|s| {
                                            *s = match *s {
                                                Some((c, SortDir::Asc)) if c == i => {
                                                    Some((i, SortDir::Desc))
                                                }
                                                _ => Some((i, SortDir::Asc)),
                                            };
                                        })
                                    >
                                        {h}
                                        <span class="sort-indicator">
                                            {move || match sort_state.get() {
                                                Some((c, SortDir::Asc)) if c == i => " ▲",
                                                Some((c, SortDir::Desc)) if c == i => " ▼",
                                                _ => " ↕",
                                            }}
                                        </span>
                                    </th>
                                }
                            })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || sorted_rows()
                        .into_iter()
                        .map(|row| {
                            view! {
                                <tr class="row-hover">
                                    {row
                                        .into_iter()
                                        .map(|cell| {
                                            view! {
                                                <td class="px-4 py-2 border-b border-gray-700">
                                                    {cell.display}
                                                </td>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
