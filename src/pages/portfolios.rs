use crate::models::{Asset, AssetGroup, AssetStatus, Portfolio};
use crate::stores::use_app_store;
use crate::types::{AssetType, UserRole, ViewMode};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn PortfoliosPage() -> impl IntoView {
    let app_store = use_app_store();

    // Read portfolios from AppStore
    let portfolios = Memo::new(move |_| app_store.get().portfolios.clone());
    let view_mode = move || app_store.get().portfolio_view_mode.clone();
    let selected_id = move || app_store.get().selected_portfolio_id;
    let can_edit = move || {
        let role = app_store.get().current_user.role.clone();
        role == UserRole::Owner || role == UserRole::Manager
    };

    // Form signals for add portfolio
    let (show_add_portfolio, set_show_add_portfolio) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    let (new_desc, set_new_desc) = signal(String::new());

    // Form signals for add asset group
    let (show_add_group, set_show_add_group) = signal(Option::<Uuid>::None);
    let (new_group_name, set_new_group_name) = signal(String::new());

    // Form signals for add asset
    let (show_add_asset, set_show_add_asset) = signal(AssetTarget::default());
    let (new_asset_name, set_new_asset_name) = signal(String::new());
    let (new_asset_type, set_new_asset_type) = signal(AssetType::RealEstate);
    let (new_asset_value, set_new_asset_value) = signal(String::new());

    let on_toggle_view = move |id: Uuid| {
        app_store.update(|s| {
            if s.selected_portfolio_id == Some(id) {
                s.selected_portfolio_id = None;
            } else {
                s.selected_portfolio_id = Some(id);
            }
        });
    };

    let on_add_portfolio = move |_| {
        let name = new_name.get();
        if name.trim().is_empty() {
            return;
        }
        let owner_id = app_store.get().current_user.id;
        let mut p = Portfolio::new(name, owner_id, crate::types::Currency::USD);
        p.description = if new_desc.get().trim().is_empty() {
            None
        } else {
            Some(new_desc.get())
        };
        app_store.update(|s| s.add_portfolio(p));
        set_new_name.set(String::new());
        set_new_desc.set(String::new());
        set_show_add_portfolio.set(false);
    };

    let on_delete_portfolio = move |id: Uuid| {
        app_store.update(|s| {
            s.remove_portfolio(id);
            if s.selected_portfolio_id == Some(id) {
                s.selected_portfolio_id = None;
            }
        });
    };

    let on_add_group = Callback::new(move |portfolio_id: Uuid| {
        let name = new_group_name.get();
        if name.trim().is_empty() {
            return;
        }
        let group = create_mock_asset_group(&name, vec![]);
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(portfolio_id) {
                p.asset_groups.push(group);
                p.recalculate_values();
            }
        });
        set_new_group_name.set(String::new());
        set_show_add_group.set(None);
    });

    let on_add_asset = Callback::new(move |target: AssetTarget| {
        let name = new_asset_name.get();
        if name.trim().is_empty() {
            return;
        }
        let value: f64 = new_asset_value.get().parse().unwrap_or(0.0);
        let _asset = create_mock_asset(&name, new_asset_type.get(), value, value);
        app_store.update(|s| {
            match target {
                AssetTarget::PortfolioDirect(pid) => {
                    if let Some(p) = s.get_portfolio_mut(pid) {
                        p.assets.push(_asset);
                        p.recalculate_values();
                    }
                }
                AssetTarget::Group(pid, gid) => {
                    if let Some(p) = s.get_portfolio_mut(pid) {
                        if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                            g.assets.push(_asset);
                            g.recalculate_values();
                        }
                        p.recalculate_values();
                    }
                }
                AssetTarget::None => {}
            }
        });
        set_new_asset_name.set(String::new());
        set_new_asset_value.set(String::new());
        set_show_add_asset.set(AssetTarget::default());
    });

    let selected_portfolio = move || {
        selected_id().and_then(|id| portfolios.get().into_iter().find(|p| p.id == id))
    };

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Portfolios"</h1>
                <p>"Manage your asset portfolios"</p>
            </div>

            // View Toggle + Add button
            <div class="view-toggle">
                <button
                    class="view-btn"
                    class:active={move || view_mode() == ViewMode::List}
                    on:click=move |_| {
                        app_store.update(|s| s.portfolio_view_mode = ViewMode::List);
                    }
                >
                    "📋 List"
                </button>
                <button
                    class="view-btn"
                    class:active={move || view_mode() == ViewMode::Grid}
                    on:click=move |_| {
                        app_store.update(|s| s.portfolio_view_mode = ViewMode::Grid);
                    }
                >
                    "⊞ Grid"
                </button>
                {move || if can_edit() {
                    view! {
                        <button
                            class="view-btn"
                            class:active=show_add_portfolio
                            on:click=move |_| set_show_add_portfolio.update(|v| *v = !*v)
                        >
                            "+ Add Portfolio"
                        </button>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            // Add Portfolio Form
            {move || show_add_portfolio.get().then(|| view! {
                <div class="add-form">
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Portfolio name"
                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                    />
                    <input
                        class="login-input"
                        type="text"
                        placeholder="Description (optional)"
                        on:input=move |ev| set_new_desc.set(event_target_value(&ev))
                    />
                    <button class="login-btn" on:click=on_add_portfolio>"Create Portfolio"</button>
                </div>
            })}

            // Portfolios List
            <div class={move || {
                if view_mode() == ViewMode::Grid {
                    "grid-view"
                } else {
                    "data-list"
                }
            }}>
                {move || {
                    let mode = view_mode();
                    let can = can_edit();
                    portfolios.get()
                        .into_iter()
                        .map(move |portfolio| {
                            let pl_class = if portfolio.profit_loss >= 0.0 {
                                "positive"
                            } else {
                                "negative"
                            };
                            let is_selected = selected_id() == Some(portfolio.id);
                            let portfolio_id = portfolio.id;
                            let pid_del = portfolio.id;

                            if mode == ViewMode::Grid {
                                view! {
                                    <div class="grid-item" on:click=move |_| on_toggle_view(portfolio_id)>
                                        <div class="grid-item-img">"🏢"</div>
                                        <div>
                                            <div class="list-item-title">{portfolio.name.clone()}</div>
                                            <div class={format!("list-item-value {}", pl_class)}>
                                                {format!("${:.1}M", portfolio.total_value / 1000000.0)}
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="data-card">
                                        <div class="card-header">
                                            <span class="card-title">{portfolio.name.clone()}</span>
                                            <div class="card-actions">
                                                <button
                                                    class="card-btn"
                                                    class:active=is_selected
                                                    on:click=move |_| on_toggle_view(portfolio_id)
                                                >
                                                    {if is_selected { "Hide Assets" } else { "View Assets" }}
                                                </button>
                                                {move || if can {
                                                    view! {
                                                        <button
                                                            class="card-btn sell"
                                                            on:click=move |_| on_delete_portfolio(pid_del)
                                                        >
                                                            "🗑 Delete"
                                                        </button>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                            </div>
                                        </div>
                                        <div class="card-stats">
                                            <div class="stat-item">
                                                <div class="stat-label">"Current Value"</div>
                                                <div class="stat-value">
                                                    {format!("${:.2}M", portfolio.total_value / 1000000.0)}
                                                </div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">"Profit/Loss"</div>
                                                <div class={format!("stat-value {}", pl_class)}>
                                                    {format!("${:+.0}K", portfolio.profit_loss / 1000.0)}
                                                </div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">"Asset Groups"</div>
                                                <div class="stat-value">
                                                    {portfolio.asset_groups.len()}
                                                </div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">"Total Assets"</div>
                                                <div class="stat-value">
                                                    {portfolio.get_all_assets().len()}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>

            // Asset Hierarchy Viewer
            {move || selected_portfolio().map(|portfolio| view! {
                <AssetViewer
                    portfolio={portfolio}
                    can_edit={can_edit()}
                    show_add_group={show_add_group.get()}
                    set_show_add_group={set_show_add_group}
                    _new_group_name={new_group_name}
                    set_new_group_name={set_new_group_name}
                    on_add_group={on_add_group}
                    show_add_asset={show_add_asset.get()}
                    set_show_add_asset={set_show_add_asset}
                    new_asset_name={new_asset_name}
                    set_new_asset_name={set_new_asset_name}
                    new_asset_type={new_asset_type}
                    set_new_asset_type={set_new_asset_type}
                    new_asset_value={new_asset_value}
                    set_new_asset_value={set_new_asset_value}
                    on_add_asset={on_add_asset}
                />
            })}
        </div>
    }
}

#[derive(Clone, PartialEq, Default)]
pub enum AssetTarget {
    #[default]
    None,
    PortfolioDirect(Uuid),
    Group(Uuid, Uuid),
}

#[component]
fn AssetViewer(
    portfolio: Portfolio,
    can_edit: bool,
    show_add_group: Option<Uuid>,
    set_show_add_group: WriteSignal<Option<Uuid>>,
    _new_group_name: ReadSignal<String>,
    set_new_group_name: WriteSignal<String>,
    on_add_group: Callback<Uuid>,
    show_add_asset: AssetTarget,
    set_show_add_asset: WriteSignal<AssetTarget>,
    new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget>,
) -> impl IntoView {
    let pid = portfolio.id;
    let show_add_asset_for_groups = show_add_asset.clone();

    view! {
        <div class="asset-viewer">
            <div class="welcome-header">
                <h1>{format!("{} Asset Hierarchy", portfolio.name)}</h1>
                <p>{format!("{} direct assets, {} asset groups", portfolio.assets.len(), portfolio.asset_groups.len())}</p>
            </div>

            // Direct Assets section
            <div class="asset-section">
                <div class="asset-section-title">
                    "Direct Assets"
                    {move || if can_edit {
                        let pid2 = pid;
                        view! {
                            <button
                                class="add-btn-small"
                                on:click=move |_| set_show_add_asset.set(AssetTarget::PortfolioDirect(pid2))
                            >
                                "+ Add Asset"
                            </button>
                        }.into_any()
                    } else { ().into_any() }}
                </div>

                {move || {
                    let target = show_add_asset.clone();
                    if target == AssetTarget::PortfolioDirect(pid) {
                        view! {
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Asset name"
                                    on:input=move |ev| set_new_asset_name.set(event_target_value(&ev)) />
                                <select class="login-input"
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        let t = match v.as_str() {
                                            "RealEstate" => AssetType::RealEstate,
                                            "Vehicle" => AssetType::Vehicle,
                                            "Equipment" => AssetType::Equipment,
                                            "Stock" => AssetType::Stock,
                                            "Bond" => AssetType::Bond,
                                            "Commodity" => AssetType::Commodity,
                                            "Digital" => AssetType::Digital,
                                            "IntellectualProperty" => AssetType::IntellectualProperty,
                                            _ => AssetType::RealEstate,
                                        };
                                        set_new_asset_type.set(t);
                                    }
                                >
                                    <option value="RealEstate">"Real Estate"</option>
                                    <option value="Vehicle">"Vehicle"</option>
                                    <option value="Equipment">"Equipment"</option>
                                    <option value="Stock">"Stock"</option>
                                    <option value="Bond">"Bond"</option>
                                    <option value="Commodity">"Commodity"</option>
                                    <option value="Digital">"Digital"</option>
                                    <option value="IntellectualProperty">"IP"</option>
                                </select>
                                <input class="login-input" type="number" placeholder="Value ($)"
                                    on:input=move |ev| set_new_asset_value.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| on_add_asset.run(AssetTarget::PortfolioDirect(pid))>
                                    "Add Asset"
                                </button>
                            </div>
                        }.into_any()
                    } else { ().into_any() }
                }}

                {if portfolio.assets.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-text">"No direct assets"</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="asset-list">
                            {portfolio.assets.into_iter().map(|asset| view! {
                                <AssetItem asset={asset} />
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </div>

            // Asset Groups section
            <div class="asset-section">
                <div class="asset-section-title">
                    "Asset Groups"
                    {move || if can_edit {
                        let pid2 = pid;
                        view! {
                            <button
                                class="add-btn-small"
                                on:click=move |_| set_show_add_group.set(Some(pid2))
                            >
                                "+ Add Group"
                            </button>
                        }.into_any()
                    } else { ().into_any() }}
                </div>

                {move || show_add_group.map(|gp| {
                    if gp == pid {
                        view! {
                            <div class="add-form">
                                <input class="login-input" type="text" placeholder="Group name"
                                    on:input=move |ev| set_new_group_name.set(event_target_value(&ev)) />
                                <button class="login-btn" on:click=move |_| on_add_group.run(pid)>
                                    "Add Group"
                                </button>
                            </div>
                        }.into_any()
                    } else { ().into_any() }
                })}

                {if portfolio.asset_groups.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-text">"No asset groups"</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="asset-list">
                            {portfolio.asset_groups.into_iter().map(|group| {
                                let gid = group.id;
                                let pid2 = pid;
                                view! {
                                    <AssetGroupItem
                                        group={group}
                                        can_edit={can_edit}
                                        pid={pid2}
                                        gid={gid}
                                        show_add_asset={show_add_asset_for_groups.clone()}
                                        set_show_add_asset={set_show_add_asset}
                                        _new_asset_name={new_asset_name}
                                        set_new_asset_name={set_new_asset_name}
                                        _new_asset_type={new_asset_type}
                                        set_new_asset_type={set_new_asset_type}
                                        _new_asset_value={new_asset_value}
                                        set_new_asset_value={set_new_asset_value}
                                        on_add_asset={on_add_asset}
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn AssetGroupItem(
    group: AssetGroup,
    can_edit: bool,
    pid: Uuid,
    gid: Uuid,
    show_add_asset: AssetTarget,
    set_show_add_asset: WriteSignal<AssetTarget>,
    _new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    _new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    _new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget>,
) -> impl IntoView {
    view! {
        <div class="asset-group">
            <div class="asset-group-header">
                <div class="asset-group-icon">"📁"</div>
                <div>
                    <div class="asset-group-name">{group.name}</div>
                    <div class="asset-group-count">{format!("{} assets", group.assets.len())}</div>
                </div>
                {move || if can_edit {
                    let pid2 = pid;
                    let gid2 = gid;
                    view! {
                        <button
                            class="add-btn-small"
                            on:click=move |_| set_show_add_asset.set(AssetTarget::Group(pid2, gid2))
                        >
                            "+ Add Asset"
                        </button>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            {move || {
                if show_add_asset == AssetTarget::Group(pid, gid) {
                    view! {
                        <div class="add-form">
                            <input class="login-input" type="text" placeholder="Asset name"
                                on:input=move |ev| set_new_asset_name.set(event_target_value(&ev)) />
                            <select class="login-input"
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    let t = match v.as_str() {
                                        "RealEstate" => AssetType::RealEstate,
                                        "Vehicle" => AssetType::Vehicle,
                                        "Equipment" => AssetType::Equipment,
                                        "Stock" => AssetType::Stock,
                                        "Bond" => AssetType::Bond,
                                        "Commodity" => AssetType::Commodity,
                                        "Digital" => AssetType::Digital,
                                        "IntellectualProperty" => AssetType::IntellectualProperty,
                                        _ => AssetType::RealEstate,
                                    };
                                    set_new_asset_type.set(t);
                                }
                            >
                                <option value="RealEstate">"Real Estate"</option>
                                <option value="Vehicle">"Vehicle"</option>
                                <option value="Equipment">"Equipment"</option>
                                <option value="Stock">"Stock"</option>
                                <option value="Bond">"Bond"</option>
                                <option value="Commodity">"Commodity"</option>
                                <option value="Digital">"Digital"</option>
                                <option value="IntellectualProperty">"IP"</option>
                            </select>
                            <input class="login-input" type="number" placeholder="Value ($)"
                                on:input=move |ev| set_new_asset_value.set(event_target_value(&ev)) />
                            <button class="login-btn"
                                on:click=move |_| on_add_asset.run(AssetTarget::Group(pid, gid))>
                                "Add Asset"
                            </button>
                        </div>
                    }.into_any()
                } else { ().into_any() }
            }}

            <div class="asset-group-assets">
                {group.assets.into_iter().map(|asset| view! {
                    <AssetItem asset={asset} />
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn AssetItem(asset: Asset) -> impl IntoView {
    let pl_class = if asset.profit_loss >= 0.0 { "positive" } else { "negative" };
    let icon = match asset.asset_type {
        AssetType::RealEstate => "🏢",
        AssetType::Vehicle => "🚗",
        AssetType::Equipment => "⚙️",
        AssetType::Stock => "📈",
        AssetType::Bond => "📜",
        AssetType::Commodity => "🌾",
        AssetType::Digital => "💻",
        AssetType::IntellectualProperty => "💡",
        AssetType::Custom(_) => "📦",
    };

    view! {
        <div class="asset-item">
            <div class="asset-icon">{icon}</div>
            <div class="asset-info">
                <div class="asset-name">{asset.name}</div>
                <div class="asset-type">{format!("{:?}", asset.asset_type)}</div>
            </div>
            <div class="asset-value">
                <div class="asset-current">{format!("${:.2}M", asset.current_value / 1000000.0)}</div>
                <div class={format!("asset-pl {}", pl_class)}>{format!("${:+.0}K", asset.profit_loss / 1000.0)}</div>
            </div>
        </div>
    }
}

// Helper functions to create mock data
fn create_mock_asset(name: &str, asset_type: AssetType, purchase: f64, current: f64) -> Asset {
    Asset {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: None,
        asset_type,
        purchase_value: purchase,
        current_value: current,
        profit_loss: current - purchase,
        profit_loss_percent: ((current - purchase) / purchase) * 100.0,
        revenue: 0.0,
        purchase_date: chrono::Utc::now(),
        images: vec![],
        documents: vec![],
        tags: vec![],
        status: AssetStatus::Active,
        metadata: serde_json::json!({}),
        assigned_workers: vec![],
        quick_sale_enabled: false,
        notification_settings: vec![],
    }
}

fn create_mock_asset_group(name: &str, assets: Vec<Asset>) -> AssetGroup {
    let mut group = AssetGroup {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: None,
        assets,
        total_value: 0.0,
        purchase_value: 0.0,
        profit_loss: 0.0,
        profit_loss_percent: 0.0,
        revenue: 0.0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        tags: vec![],
        documents: vec![],
    };
    group.recalculate_values();
    group
}
