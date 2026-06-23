use crate::stores::use_app_store;
use leptos::prelude::*;

#[component]
pub fn OverviewPage() -> impl IntoView {
    let app_store = use_app_store();
    let portfolio_count = move || app_store.get().portfolios.len();
    let user_name = move || app_store.get().current_user.name.clone();

    view! {
        <div class="overview-content">
            <div class="overview-greeting">
                {move || format!("Welcome, {}", user_name())}
            </div>
            <div class="overview-stat-row">
                <div class="overview-stat">
                    <div class="overview-stat-value">{portfolio_count}</div>
                    <div class="overview-stat-label">"Portfolios"</div>
                </div>
            </div>
        </div>
    }
}
