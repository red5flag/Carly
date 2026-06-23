use crate::stores::use_search_store;
use leptos::prelude::*;

#[component]
pub fn SearchFilters() -> impl IntoView {
    let (adv_open, set_adv_open) = signal(false);
    let (lineage_open, set_lineage_open) = signal(false);
    let (tree_open, set_tree_open) = signal(false);
    let (prof1, set_prof1) = signal(false);
    let (prof2, set_prof2) = signal(false);
    let (time_from, set_time_from) = signal(String::new());
    let (time_to, set_time_to) = signal(String::new());
    let (asset_p, set_asset_p) = signal(false);
    let (asset_a, set_asset_a) = signal(false);
    let (asset_addr, set_asset_addr) = signal(false);
    let (chg_add, set_chg_add) = signal(false);
    let (chg_rm, set_chg_rm) = signal(false);
    let (chg_ch, set_chg_ch) = signal(false);
    let (chg_undo, set_chg_undo) = signal(false);
    let (chg_redo, set_chg_redo) = signal(false);

    view! {
        <div class="sd-panel">
            <div class="sd-section">
                <div class="sd-section-header" on:click=move |_| set_adv_open.update(|v| *v = !*v)>
                    <span class="sd-section-title">"ADVANCED SEARCH"</span>
                    <span class="sd-arrow">{move || if adv_open.get() {"▲"} else {"▼"}}</span>
                </div>
                {move || if adv_open.get() { view! {
                    <div class="sd-adv-body">
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"PROFILE"</div>
                            <div class="sd-filter-chips">
                                <button class="sd-chip" class:sd-chip-active=move || prof1.get()
                                    on:click=move |_| set_prof1.update(|v| *v = !*v)>"Stevenson 2"</button>
                                <button class="sd-chip sd-chip-purple" class:sd-chip-active=move || prof2.get()
                                    on:click=move |_| set_prof2.update(|v| *v = !*v)>"Stevenson 3"</button>
                            </div>
                        </div>
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"TIME"</div>
                            <div class="sd-filter-chips">
                                <input class="sd-time-input" type="text" placeholder="From: DD/MM/YYYY"
                                    prop:value=time_from
                                    on:input=move |ev| set_time_from.set(event_target_value(&ev)) />
                                <input class="sd-time-input" type="text" placeholder="To: DD/MM/YYYY"
                                    prop:value=time_to
                                    on:input=move |ev| set_time_to.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"ASSET"</div>
                            <div class="sd-filter-chips">
                                <button class="sd-chip sd-chip-green" class:sd-chip-active=move || asset_p.get()
                                    on:click=move |_| set_asset_p.update(|v| *v = !*v)>"PORTFOLIO"</button>
                                <button class="sd-chip sd-chip-blue" class:sd-chip-active=move || asset_a.get()
                                    on:click=move |_| set_asset_a.update(|v| *v = !*v)>"ASSET"</button>
                                <button class="sd-chip sd-chip-purple" class:sd-chip-active=move || asset_addr.get()
                                    on:click=move |_| set_asset_addr.update(|v| *v = !*v)>"ADDRESS"</button>
                            </div>
                        </div>
                        <div class="sd-filter-row">
                            <div class="sd-filter-label">"CHANGE"</div>
                            <div class="sd-filter-chips">
                                <button class="sd-chip sd-chip-green" class:sd-chip-active=move || chg_add.get()
                                    on:click=move |_| set_chg_add.update(|v| *v = !*v)>"ADD"</button>
                                <button class="sd-chip sd-chip-red" class:sd-chip-active=move || chg_rm.get()
                                    on:click=move |_| set_chg_rm.update(|v| *v = !*v)>"REMOVE"</button>
                                <button class="sd-chip sd-chip-yellow" class:sd-chip-active=move || chg_ch.get()
                                    on:click=move |_| set_chg_ch.update(|v| *v = !*v)>"CHANGE"</button>
                                <button class="sd-chip" class:sd-chip-active=move || chg_undo.get()
                                    on:click=move |_| set_chg_undo.update(|v| *v = !*v)>"UNDO"</button>
                                <button class="sd-chip" class:sd-chip-active=move || chg_redo.get()
                                    on:click=move |_| set_chg_redo.update(|v| *v = !*v)>"REDO"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()} else {().into_any()}}
            </div>

            <div class="sd-section">
                <div class="sd-section-header" on:click=move |_| set_lineage_open.update(|v| *v = !*v)>
                    <span class="sd-section-title">"LINEAGE VIEW"</span>
                    <span class="sd-arrow">{move || if lineage_open.get() {"▲"} else {"▼"}}</span>
                </div>
                {move || if lineage_open.get() { view! {
                    <div class="sd-lineage-body">
                        <div class="sd-lineage-cols">
                            <div class="sd-lineage-col">
                                <div class="sd-lineage-col-header">"STEVENSON 2"</div>
                                <div class="sd-lineage-row-label">"DATE"</div>
                                <div class="sd-lineage-row-label">"PORT"</div>
                                <div class="sd-lineage-row-label">"ASSET"</div>
                                <div class="sd-lineage-row-label">"CHANGE"</div>
                            </div>
                            <div class="sd-lineage-col">
                                <div class="sd-lineage-col-header">"STEVENSON 3"</div>
                                <div class="sd-lineage-row-label">"DATE"</div>
                                <div class="sd-lineage-row-label">"PORT"</div>
                                <div class="sd-lineage-row-label">"ASSET"</div>
                                <div class="sd-lineage-row-label">"CHANGE"</div>
                            </div>
                        </div>
                    </div>
                }.into_any()} else {().into_any()}}
            </div>

            <div class="sd-section">
                <div class="sd-section-header" on:click=move |_| set_tree_open.update(|v| *v = !*v)>
                    <span class="sd-section-title">"TREE VIEW"</span>
                    <span class="sd-arrow">{move || if tree_open.get() {"▲"} else {"▼"}}</span>
                </div>
                {move || if tree_open.get() { view! {
                    <div class="sd-tree-body">
                        <table class="sd-tree-table">
                            <thead>
                                <tr>
                                    <th>"PAGE"</th><th>"DETAILS"</th>
                                    <th>"TIME"</th><th>"CONFIG"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr><td>"Login"</td><td>"Profile"</td><td>"—"</td><td>"—"</td></tr>
                                <tr><td>"Home"</td><td>"Base"</td><td>"—"</td><td>"—"</td></tr>
                                <tr><td>"Overview"</td><td>"Portfolio"</td><td>"—"</td><td>"—"</td></tr>
                                <tr><td>"Group Chat"</td><td>"GroupChatLink"</td><td>"—"</td><td>"—"</td></tr>
                                <tr><td>"Message"</td><td>"OpenLink"</td><td>"—"</td><td>"—"</td></tr>
                            </tbody>
                        </table>
                    </div>
                }.into_any()} else {().into_any()}}
            </div>
        </div>
    }
}

#[component]
pub fn SearchResults() -> impl IntoView {
    let search_store = use_search_store();
    let is_loading = move || search_store.get().is_loading;
    let has_results = move || search_store.get().results.total_count > 0;
    let results_count = move || search_store.get().results.total_count;
    view! {
        <div class="sd-results">
            {move || if is_loading() {
                view! { <div class="loading"><span class="loading-text">"Searching..."</span></div> }.into_any()
            } else if has_results() {
                view! { <div><span class="sd-result-count">{format!("{} results", results_count())}</span></div> }.into_any()
            } else {
                view! { <div class="empty-state"><div class="empty-icon">"🔍"</div><div class="empty-text">"No results"</div></div> }.into_any()
            }}
        </div>
    }
}
