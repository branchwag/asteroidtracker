use leptos::prelude::*;

#[component]
pub fn ImpactTable(headers: Vec<String>, rows: Vec<Vec<String>>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto w-full max-w-4xl mx-auto mt-8">
            <table class="min-w-full bg-black text-white border border-gray-700 rounded-lg shadow-lg">
                <thead>
                    <tr>
                        {headers
                            .into_iter()
                            .map(|h| {
                                view! {
                                    <th class="px-4 py-2 border-b border-gray-700 text-left">
                                        {h}
                                    </th>
                                }
                            })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {rows
                        .into_iter()
                        .map(|row| {
                            view! {
                                <tr class="row-hover">
                                    {row
                                        .into_iter()
                                        .map(|cell| {
                                            view! {
                                                <td class="px-4 py-2 border-b border-gray-700">
                                                    {cell}
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
