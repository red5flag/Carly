use crate::components::tabs::use_tab_edit_mode;
use crate::models::{Asset, AssetGroup, AssetStatus, Document, Portfolio};
use crate::stores::use_app_store;
use crate::types::{AssetType, SortMode, UserRole, ViewMode};
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn PortfoliosPage() -> impl IntoView {
    let app_store = use_app_store();

    // Read portfolios from AppStore
    let portfolios = Memo::new(move |_| app_store.get().portfolios.clone());
    let view_mode = move || app_store.get().portfolio_view_mode.clone();
    let (sort_mode, set_sort_mode) = signal(SortMode::Recent);
    let selected_id = move || app_store.get().selected_portfolio_id;
    let edit_mode = use_tab_edit_mode();
    let can_edit = move || {
        let role = app_store.get().current_user.role.clone();
        edit_mode.get() && (role == UserRole::Owner || role == UserRole::Manager)
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

    // Modal signal for editing portfolio assets
    let (edit_portfolio_id, set_edit_portfolio_id) = signal(Option::<Uuid>::None);
    let (context_menu, set_context_menu) = signal(Option::<(Uuid, i32, i32)>::None);
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
        set_edit_portfolio_id.set(None);
    };

    let on_open_edit = move |id: Uuid| {
        set_edit_portfolio_id.set(Some(id));
    };

    let on_close_edit = move |_| {
        set_edit_portfolio_id.set(None);
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
            // Edit portfolio assets modal
            {move || edit_portfolio_id.get().map(|pid| {
                let pid_add_asset = pid;
                let pid_add_group = pid;
                let pid_delete = pid;
                view! {
                    <div class="modal-overlay" on:click=move |_| on_close_edit(())>
                        <div class="modal" on:click=|ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <span class="modal-title">"Edit Portfolio Assets"</span>
                                <button class="modal-close" on:click=move |_| on_close_edit(())>"×"</button>
                            </div>
                            <div class="modal-body">
                                <div class="edit-actions">
                                    <button
                                        class="login-btn"
                                        on:click=move |_| {
                                            set_show_add_asset.set(AssetTarget::PortfolioDirect(pid_add_asset));
                                            on_close_edit(());
                                        }
                                    >
                                        "+ Add Asset"
                                    </button>
                                    <button
                                        class="login-btn"
                                        on:click=move |_| {
                                            set_show_add_group.set(Some(pid_add_group));
                                            on_close_edit(());
                                        }
                                    >
                                        "+ Add Group"
                                    </button>
                                    <button
                                        class="login-btn sell"
                                        on:click=move |_| {
                                            on_delete_portfolio(pid_delete);
                                        }
                                    >
                                        "🗑 Delete Portfolio"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any()
            })}

            // View Toggle + Sort + Add button
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
                <select
                    class="view-btn"
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        let mode = match v.as_str() {
                            "oldest" => SortMode::Oldest,
                            "highest_value" => SortMode::HighestValue,
                            "lowest_value" => SortMode::LowestValue,
                            "highest_profit" => SortMode::HighestProfit,
                            "lowest_profit" => SortMode::LowestProfit,
                            "highest_revenue" => SortMode::HighestRevenue,
                            "lowest_revenue" => SortMode::LowestRevenue,
                            "by_organization" => SortMode::ByOrganization,
                            _ => SortMode::Recent,
                        };
                        set_sort_mode.set(mode);
                    }
                >
                    <option value="recent">"Recent"</option>
                    <option value="oldest">"Oldest"</option>
                    <option value="highest_value">"Highest Value"</option>
                    <option value="lowest_value">"Lowest Value"</option>
                    <option value="highest_profit">"Highest Profit"</option>
                    <option value="lowest_profit">"Lowest Profit"</option>
                    <option value="highest_revenue">"Highest Revenue"</option>
                    <option value="lowest_revenue">"Lowest Revenue"</option>
                    <option value="by_organization">"By Organization"</option>
                </select>
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
                if view_mode() == ViewMode::Grid { "grid-view" } else { "pf-accordion" }
            }}>
                {move || {
                    let mode = view_mode();
                    let can = can_edit();
                    let sort = sort_mode.get();
                    let mut items: Vec<_> = portfolios.get().into_iter().collect();
                    items.sort_by(|a, b| match sort {
                        SortMode::Recent => b.created_at.cmp(&a.created_at),
                        SortMode::Oldest => a.created_at.cmp(&b.created_at),
                        SortMode::HighestValue => b.total_value.partial_cmp(&a.total_value).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::LowestValue => a.total_value.partial_cmp(&b.total_value).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::HighestProfit => b.profit_loss.partial_cmp(&a.profit_loss).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::LowestProfit => a.profit_loss.partial_cmp(&b.profit_loss).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::HighestRevenue => b.revenue.partial_cmp(&a.revenue).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::LowestRevenue => a.revenue.partial_cmp(&b.revenue).unwrap_or(std::cmp::Ordering::Equal),
                        SortMode::ByOrganization => a.organization_id.cmp(&b.organization_id),
                    });
                    items.into_iter().map(move |portfolio| {
                        let pl_class = if portfolio.profit_loss >= 0.0 { "positive" } else { "negative" };
                        let portfolio_id = portfolio.id;
                        let is_expanded = selected_id() == Some(portfolio_id);

                        if mode == ViewMode::Grid {
                            let pf_name   = portfolio.name.clone();
                            let pf_val    = portfolio.total_value;
                            let pf_groups = portfolio.asset_groups.clone();
                            let pf_direct = portfolio.assets.len();
                            view! {
                                <div
                                    class="pf-grid-card"
                                    on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                                        ev.prevent_default();
                                        set_context_menu.set(Some((portfolio_id, ev.client_x(), ev.client_y())));
                                    }
                                >
                                    // Card header
                                    <div class="pf-grid-header">
                                        <span class="pf-grid-icon">"🏢"</span>
                                        <div class="pf-grid-title-wrap">
                                            <div class="pf-grid-name">{pf_name}</div>
                                            <div class={format!("pf-grid-value {}", pl_class)}>
                                                {format!("${:.2}M", pf_val / 1_000_000.0)}
                                            </div>
                                        </div>
                                    </div>
                                    // Asset groups list
                                    <div class="pf-grid-groups">
                                        {pf_groups.into_iter().map(|g| {
                                            let g_val = g.total_value;
                                            let a_count = g.assets.len();
                                            view! {
                                                <div class="pf-grid-group">
                                                    <span class="pf-grid-group-icon">"📁"</span>
                                                    <div class="pf-grid-group-info">
                                                        <span class="pf-grid-group-name">{g.name.clone()}</span>
                                                        <span class="pf-grid-group-meta">
                                                            {format!("{} asset{} • ${:.1}M",
                                                                a_count,
                                                                if a_count == 1 {""} else {"s"},
                                                                g_val / 1_000_000.0)}
                                                        </span>
                                                    </div>
                                                    // Assets within group
                                                    <div class="pf-grid-assets">
                                                        {g.assets.into_iter().map(|a| {
                                                            let aval = a.current_value;
                                                            let aloc = a.location.clone().unwrap_or_default();
                                                            view! {
                                                                <div class="pf-grid-asset">
                                                                    <span class="pf-grid-asset-name">{a.name.clone()}</span>
                                                                    <span class="pf-grid-asset-loc">{aloc}</span>
                                                                    <span class="pf-grid-asset-val">
                                                                        {format!("${:.0}k", aval / 1000.0)}
                                                                    </span>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                        {if pf_direct > 0 {
                                            view! {
                                                <div class="pf-grid-direct">
                                                    {format!("{} direct asset{}",
                                                        pf_direct,
                                                        if pf_direct == 1 {""} else {"s"})}
                                                </div>
                                            }.into_any()
                                        } else { ().into_any() }}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <PortfolioListItem
                                    portfolio={portfolio}
                                    can_edit={can}
                                    expanded={is_expanded}
                                    on_toggle=move || on_toggle_view(portfolio_id)
                                    on_context=move |ev: leptos::ev::MouseEvent| {
                                        ev.prevent_default();
                                        set_context_menu.set(Some((portfolio_id, ev.client_x(), ev.client_y())));
                                    }
                                    show_add_group={show_add_group.get()}
                                    set_show_add_group={set_show_add_group}
                                    _new_group_name={new_group_name}
                                    set_new_group_name={set_new_group_name}
                                    on_add_group={on_add_group}
                                    show_add_asset={show_add_asset}
                                    set_show_add_asset={set_show_add_asset}
                                    new_asset_name={new_asset_name}
                                    set_new_asset_name={set_new_asset_name}
                                    new_asset_type={new_asset_type}
                                    set_new_asset_type={set_new_asset_type}
                                    new_asset_value={new_asset_value}
                                    set_new_asset_value={set_new_asset_value}
                                    on_add_asset={on_add_asset}
                                    view_mode={view_mode()}
                                />
                            }.into_any()
                        }
                    })
                    .collect::<Vec<_>>()
                }}
            </div>

            // Context menu for portfolio press-and-hold
            {move || context_menu.get().map(|(pid, x, y)| {
                view! {
                    <div
                        class="context-menu-overlay"
                        on:click=move |_| set_context_menu.set(None)
                    >
                        <div
                            class="context-menu"
                            style={format!("left: {}px; top: {}px;", x, y)}
                        >
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    on_open_edit(pid);
                                }
                            >
                                "Edit"
                            </button>
                            <button
                                class="context-menu-item"
                                on:click=move |_| {
                                    set_context_menu.set(None);
                                    on_toggle_view(pid);
                                }
                            >
                                "Overview"
                            </button>
                        </div>
                    </div>
                }.into_any()
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
    view_mode: ViewMode,
    show_add_group: Option<Uuid>,
    set_show_add_group: WriteSignal<Option<Uuid>>,
    _new_group_name: ReadSignal<String>,
    set_new_group_name: WriteSignal<String>,
    on_add_group: Callback<Uuid>,
    show_add_asset: ReadSignal<AssetTarget>,
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
    let (expanded_groups, set_expanded_groups) = signal(HashSet::<Uuid>::new());
    let toggle_group = Callback::new(move |gid: Uuid| {
        set_expanded_groups.update(|set| {
            if !set.remove(&gid) {
                set.insert(gid);
            }
        });
    });

    let (group_edit_mode, set_group_edit_mode) = signal(false);
    let (grid_columns, _set_grid_columns) = signal(3usize);
    let (selected_asset, set_selected_asset) = signal::<Option<Asset>>(None);

    let on_select_asset = Callback::new(move |asset: Asset| {
        set_selected_asset.set(Some(asset));
    });

    let on_close_asset = Callback::new(move |_| {
        set_selected_asset.set(None);
    });

    view! {
        <div class="asset-viewer">
            // Asset Groups section
            <div class="asset-section">
                <div class="asset-section-title">
                    <span>"Asset Groups"</span>
                    <div class="section-title-right">
                        {{
                            let view_mode = view_mode.clone();
                            move || if view_mode == ViewMode::Grid {
                                view! {
                                    <button class="sort-btn">"Sort ↕"</button>
                                }.into_any()
                            } else { ().into_any() }
                        }}
                        {move || if can_edit {
                            let pid2 = pid;
                            let edit_mode_active = group_edit_mode.get();
                            view! {
                                <button
                                    class="add-btn-small"
                                    class:active=edit_mode_active
                                    on:click=move |_| set_group_edit_mode.update(|v| *v = !*v)
                                >
                                    "✎"
                                </button>
                                <button
                                    class="add-btn-small"
                                    on:click=move |_| set_show_add_group.set(Some(pid2))
                                >
                                    "+"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
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
                    let group_class = if view_mode == ViewMode::Grid { "grid-view" } else { "asset-list" };
                    let view_mode_clone = view_mode.clone();
                    let portfolio_name = portfolio.name.clone();
                    view! {
                        <div class={group_class}>
                            {portfolio.asset_groups.into_iter().map(move |group| {
                                let gid = group.id;
                                let pid2 = pid;
                                let is_expanded = Memo::new(move |_| expanded_groups.get().contains(&gid));
                                view! {
                                    <AssetGroupItem
                                        group={group}
                                        can_edit={can_edit}
                                        pid={pid2}
                                        gid={gid}
                                        expanded={is_expanded}
                                        edit_mode={group_edit_mode}
                                        view_mode={view_mode_clone.clone()}
                                        grid_columns={grid_columns.get()}
                                        on_toggle={toggle_group}
                                        show_add_asset={show_add_asset}
                                        set_show_add_asset={set_show_add_asset}
                                        _new_asset_name={new_asset_name}
                                        set_new_asset_name={set_new_asset_name}
                                        _new_asset_type={new_asset_type}
                                        set_new_asset_type={set_new_asset_type}
                                        _new_asset_value={new_asset_value}
                                        set_new_asset_value={set_new_asset_value}
                                        on_add_asset={on_add_asset}
                                        on_select_asset={on_select_asset}
                                        portfolio_name={portfolio_name.clone()}
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </div>

            // Direct Assets section
            <div class="asset-section">
                <div class="asset-section-title">
                    <span>"Direct Assets"</span>
                    <div class="section-title-right">
                        {{
                            let view_mode = view_mode.clone();
                            move || if view_mode == ViewMode::Grid {
                                view! {
                                    <button class="sort-btn">"Sort ↕"</button>
                                }.into_any()
                            } else { ().into_any() }
                        }}
                        {move || if can_edit {
                            let pid2 = pid;
                            view! {
                                <button
                                    class="add-btn-small"
                                    on:click=move |_| set_show_add_asset.set(AssetTarget::PortfolioDirect(pid2))
                                >
                                    "+"
                                </button>
                            }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>

                {move || {
                    if show_add_asset.get() == AssetTarget::PortfolioDirect(pid) {
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
                    let direct_class = if view_mode == ViewMode::Grid {
                        format!("grid-view-{}", grid_columns.get())
                    } else {
                        "asset-list".to_string()
                    };
                    let view_mode_clone = view_mode.clone();
                    let portfolio_name = portfolio.name.clone();
                    view! {
                        <div class={direct_class}>
                            {portfolio.assets.into_iter().map(move |asset| view! {
                                <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} view_mode={view_mode_clone.clone()} on_select={on_select_asset} can_edit={can_edit} edit_mode={group_edit_mode} />
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </div>

            {move || selected_asset.get().map(|asset| view! {
                <AssetDetailView asset={asset} on_close={on_close_asset} />
            })}
        </div>
    }
}

#[component]
fn AssetGroupItem(
    group: AssetGroup,
    can_edit: bool,
    pid: Uuid,
    gid: Uuid,
    expanded: Memo<bool>,
    edit_mode: ReadSignal<bool>,
    view_mode: ViewMode,
    grid_columns: usize,
    on_toggle: Callback<Uuid>,
    show_add_asset: ReadSignal<AssetTarget>,
    set_show_add_asset: WriteSignal<AssetTarget>,
    _new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    _new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    _new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget>,
    on_select_asset: Callback<Asset>,
    portfolio_name: String,
) -> impl IntoView {
    let app_store = use_app_store();
    let _ = view_mode;

    let (show_doc_modal, set_show_doc_modal) = signal(false);
    let (editing, set_editing) = signal(false);
    let (edit_name, set_edit_name) = signal(group.name.clone());
    let (edit_desc, set_edit_desc) = signal(group.description.clone().unwrap_or_default());

    let g_name = group.name.clone();
    let g_desc = group.description.clone().unwrap_or_default();
    let g_name_for_modal = group.name.clone();
    let docs = group.documents.clone();
    let doc_count = docs.len();

    let save_group_edit = move |_| {
        let n = edit_name.get();
        let d = edit_desc.get();
        if n.trim().is_empty() { return; }
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    g.name = n.clone();
                    g.description = if d.trim().is_empty() { None } else { Some(d.clone()) };
                    g.updated_at = chrono::Utc::now();
                }
            }
        });
        set_editing.set(false);
    };

    let add_group_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: "pdf".to_string(),
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: Uuid::nil(),
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                if let Some(g) = p.asset_groups.iter_mut().find(|g| g.id == gid) {
                    g.documents.push(doc);
                }
            }
        });
    };

    view! {
        <div class="asset-group" class:expanded={move || expanded.get()}>
            <div class="asset-group-header"
                on:click=move |_| if !editing.get() { on_toggle.run(gid) }>
                <span class="asset-group-arrow">
                    {move || if expanded.get() { "▲" } else { "▼" }}
                </span>
                <div class="asset-group-icon">"📁"</div>
                <div class="asset-group-info-wrap">
                    {let asset_count = group.assets.len();
                    move || if editing.get() {
                        view! {
                            <div class="asset-group-edit-form" on:click=|ev| ev.stop_propagation()>
                                <input class="pf-edit-input" placeholder="Group name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev)) />
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev)) />
                                <div style="display:flex;gap:4px">
                                    <button class="pf-edit-save" on:click=save_group_edit>"✔"</button>
                                    <button class="pf-edit-cancel" on:click=move |_| set_editing.set(false)>"✕"</button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <div class="asset-group-name">{g_name.clone()}</div>
                                {if !g_desc.is_empty() {
                                    view! { <div class="asset-group-desc">{g_desc.clone()}</div> }.into_any()
                                } else { ().into_any() }}
                                <div class="asset-group-count">{format!("{} assets", asset_count)}</div>
                            </div>
                        }.into_any()
                    }}
                </div>
                // Action buttons
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    <button class="pf-action-btn"
                        class:active=move || show_doc_modal.get()
                        on:click=move |_| set_show_doc_modal.set(true)>
                        {format!("📄 {}", doc_count)}
                    </button>
                    {move || if can_edit && edit_mode.get() {
                        let pid2 = pid; let gid2 = gid;
                        view! {
                            <button class="pf-action-btn pf-edit-btn"
                                class:active=move || editing.get()
                                on:click=move |_| set_editing.update(|v| *v = !*v)>"✎"</button>
                            <button class="pf-action-btn"
                                on:click=move |ev| { ev.stop_propagation(); set_show_add_asset.set(AssetTarget::Group(pid2, gid2)); }>
                                "+ Asset"
                            </button>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>
            // Docs modal for group
            {move || if show_doc_modal.get() {
                let docs_snap = docs.clone();
                let modal_title = g_name_for_modal.clone();
                let add_cb = if can_edit { Some(Callback::new(move |n: String| add_group_doc(n))) } else { None };
                view! {
                    <DocModal
                        docs={docs_snap}
                        title={modal_title}
                        on_close=move || set_show_doc_modal.set(false)
                        can_edit={can_edit}
                        on_add={add_cb}
                    />
                }.into_any()
            } else { ().into_any() }}

            <div class="asset-group-content" class:hidden={move || !expanded.get()}>
                {move || if show_add_asset.get() == AssetTarget::Group(pid, gid) {
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
                } else { ().into_any() }}

                {{
                    let view_mode = view_mode.clone();
                    let group_assets = group.assets;
                    let class_str = if view_mode == ViewMode::Grid {
                        format!("asset-group-assets grid-view-{}", grid_columns)
                    } else {
                        "asset-group-assets asset-list".to_string()
                    };
                    view! {
                        <div class={class_str}>
                            {group_assets.into_iter().map({
                                let view_mode = view_mode.clone();
                                move |asset| view! {
                                    <AssetItem asset={asset} portfolio_name={portfolio_name.clone()} view_mode={view_mode.clone()} on_select={on_select_asset} can_edit={can_edit} edit_mode={edit_mode} />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }}
            </div>
        </div>
    }
}

/// Portfolio list row — accordion style matching AssetGroupItem.
#[component]
fn PortfolioListItem(
    portfolio: crate::models::Portfolio,
    can_edit: bool,
    expanded: bool,
    on_toggle: impl Fn() + 'static,
    on_context: impl Fn(leptos::ev::MouseEvent) + 'static,
    // AssetViewer props forwarded for expanded content
    show_add_group: Option<Uuid>,
    set_show_add_group: WriteSignal<Option<Uuid>>,
    _new_group_name: ReadSignal<String>,
    set_new_group_name: WriteSignal<String>,
    on_add_group: Callback<Uuid>,
    show_add_asset: ReadSignal<AssetTarget>,
    set_show_add_asset: WriteSignal<AssetTarget>,
    new_asset_name: ReadSignal<String>,
    set_new_asset_name: WriteSignal<String>,
    new_asset_type: ReadSignal<AssetType>,
    set_new_asset_type: WriteSignal<AssetType>,
    new_asset_value: ReadSignal<String>,
    set_new_asset_value: WriteSignal<String>,
    on_add_asset: Callback<AssetTarget>,
    view_mode: ViewMode,
) -> impl IntoView {
    let app_store = use_app_store();
    let (show_doc_modal, set_show_doc_modal) = signal(false);
    let (editing, set_editing) = signal(false);
    let (edit_name, set_edit_name) = signal(portfolio.name.clone());
    let (edit_desc, set_edit_desc) = signal(portfolio.description.clone().unwrap_or_default());
    let pid = portfolio.id;
    let doc_count = portfolio.documents.len();
    let docs = portfolio.documents.clone();
    let name = portfolio.name.clone();
    let name_for_modal = portfolio.name.clone();
    let desc = portfolio.description.clone().unwrap_or_default();
    let asset_count = portfolio.get_all_assets().len();

    let save_edit = move |_| {
        let n = edit_name.get();
        let d = edit_desc.get();
        if n.trim().is_empty() { return; }
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.name = n.clone();
                p.description = if d.trim().is_empty() { None } else { Some(d.clone()) };
                p.updated_at = chrono::Utc::now();
            }
        });
        set_editing.set(false);
    };

    let add_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: "pdf".to_string(),
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: Uuid::nil(),
        };
        app_store.update(|s| {
            if let Some(p) = s.get_portfolio_mut(pid) {
                p.documents.push(doc);
            }
        });
    };

    let portfolio_for_viewer = portfolio.clone();

    view! {
        <div class="asset-group" class:expanded={expanded} on:contextmenu=on_context>
            // Header row — same structure as asset-group-header
            <div class="asset-group-header"
                on:click=move |_| if !editing.get() { on_toggle() }>
                <span class="asset-group-arrow">
                    {if expanded { "▲" } else { "▼" }}
                </span>
                <div class="asset-group-icon">"🏢"</div>
                <div class="asset-group-info-wrap">
                    {move || if editing.get() {
                        view! {
                            <div class="asset-group-edit-form" on:click=|ev| ev.stop_propagation()>
                                <input class="pf-edit-input" placeholder="Portfolio name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev)) />
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev)) />
                                <div style="display:flex;gap:4px">
                                    <button class="pf-edit-save" on:click=save_edit>"✔"</button>
                                    <button class="pf-edit-cancel" on:click=move |_| set_editing.set(false)>"✕"</button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <div class="asset-group-name">{name.clone()}</div>
                                {if !desc.is_empty() {
                                    view! { <div class="asset-group-desc">{desc.clone()}</div> }.into_any()
                                } else { ().into_any() }}
                                <div class="asset-group-count">
                                    {format!("{} asset{}", asset_count, if asset_count == 1 { "" } else { "s" })}
                                </div>
                            </div>
                        }.into_any()
                    }}
                </div>
                // Action strip
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    <button class="pf-action-btn"
                        class:active=move || show_doc_modal.get()
                        on:click=move |_| set_show_doc_modal.set(true)>
                        {format!("📄 {}", doc_count)}
                    </button>
                    {if can_edit {
                        view! {
                            <button class="pf-action-btn pf-edit-btn"
                                class:active=move || editing.get()
                                on:click=move |_| set_editing.update(|v| *v = !*v)>"✎"</button>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>

            // Docs modal for portfolio
            {move || if show_doc_modal.get() {
                let docs_snap = docs.clone();
                let modal_title = name_for_modal.clone();
                let add_cb = if can_edit { Some(Callback::new(move |n: String| add_doc(n))) } else { None };
                view! {
                    <DocModal
                        docs={docs_snap}
                        title={modal_title}
                        on_close=move || set_show_doc_modal.set(false)
                        can_edit={can_edit}
                        on_add={add_cb}
                    />
                }.into_any()
            } else { ().into_any() }}

            // Expanded content — AssetViewer
            <div class="asset-group-content" class:hidden={!expanded}>
                <AssetViewer
                    portfolio={portfolio_for_viewer}
                    can_edit={can_edit}
                    view_mode={view_mode}
                    show_add_group={show_add_group}
                    set_show_add_group={set_show_add_group}
                    _new_group_name={_new_group_name}
                    set_new_group_name={set_new_group_name}
                    on_add_group={on_add_group}
                    show_add_asset={show_add_asset}
                    set_show_add_asset={set_show_add_asset}
                    new_asset_name={new_asset_name}
                    set_new_asset_name={set_new_asset_name}
                    new_asset_type={new_asset_type}
                    set_new_asset_type={set_new_asset_type}
                    new_asset_value={new_asset_value}
                    set_new_asset_value={set_new_asset_value}
                    on_add_asset={on_add_asset}
                />
            </div>
        </div>
    }
}

fn asset_placeholder_url(asset_type: &AssetType, name: &str) -> String {
    let text = match asset_type {
        AssetType::RealEstate => "House",
        AssetType::Vehicle => "Car",
        AssetType::Equipment => "Gear",
        AssetType::Stock => "Stock",
        AssetType::Bond => "Bond",
        AssetType::Commodity => "Goods",
        AssetType::Digital => "Crypto",
        AssetType::IntellectualProperty => "IP",
        AssetType::Custom(_) => "Asset",
    };
    let seed = name.replace(' ', "+");
    let seed = if seed.len() > 12 { &seed[..12] } else { &seed };
    format!("https://placehold.co/400x400/2d3748/FFF?text={}%2B{}", text, seed)
}

#[component]
fn AssetItem(
    asset: Asset,
    portfolio_name: String,
    view_mode: ViewMode,
    on_select: Callback<Asset>,
    #[prop(default = false)] can_edit: bool,
    #[prop(optional)] edit_mode: Option<ReadSignal<bool>>,
) -> impl IntoView {
    let app_store = use_app_store();
    let image_url = asset
        .images
        .first()
        .cloned()
        .unwrap_or_else(|| asset_placeholder_url(&asset.asset_type, &asset.name));

    let (expanded_detail, set_expanded_detail) = signal(false);
    let (show_doc_modal, set_show_doc_modal) = signal(false);
    let (editing, set_editing) = signal(false);
    let (edit_name, set_edit_name) = signal(asset.name.clone());
    let (edit_desc, set_edit_desc) = signal(asset.description.clone().unwrap_or_default());
    let (edit_loc, set_edit_loc) = signal(asset.location.clone().unwrap_or_default());
    // doc sort: 0 = recent, 1 = name
    let (doc_sort, set_doc_sort) = signal(0u8);

    let asset_id = asset.id;
    let pname = portfolio_name.clone();
    let docs = asset.documents.clone();
    let _doc_count = docs.len();
    let a_name = asset.name.clone();
    let a_addr = asset.location.clone().unwrap_or_default();
    let asset_name_for_modal = asset.name.clone();
    // snapshot values for the detail panel
    let a_type     = format!("{:?}", asset.asset_type);
    let a_desc     = asset.description.clone().unwrap_or_else(|| "—".to_string());
    let a_status   = format!("{:?}", asset.status);
    let a_purchase_val = asset.purchase_value;
    let a_current_val  = asset.current_value;
    let a_pl           = asset.profit_loss;
    let a_pl_pct       = asset.profit_loss_percent;
    let a_revenue      = asset.revenue;
    let a_pl_cls       = if asset.profit_loss >= 0.0 { "positive" } else { "negative" };
    let a_purchase_date = asset.purchase_date.format("%d %b %Y").to_string();

    let save_edit = move |_| {
        let n = edit_name.get();
        let d = edit_desc.get();
        let l = edit_loc.get();
        if n.trim().is_empty() { return; }
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p.assets.iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == asset_id {
                        a.name = n.clone();
                        a.description = if d.trim().is_empty() { None } else { Some(d.clone()) };
                        a.location = if l.trim().is_empty() { None } else { Some(l.clone()) };
                        return;
                    }
                }
            }
        });
        set_editing.set(false);
    };

    let add_doc = move |n: String| {
        if n.trim().is_empty() { return; }
        let doc = crate::models::Document {
            id: Uuid::new_v4(),
            name: n.clone(),
            file_type: "pdf".to_string(),
            url: "#".to_string(),
            uploaded_at: chrono::Utc::now(),
            uploaded_by: Uuid::nil(),
        };
        app_store.update(|s| {
            for p in s.portfolios.iter_mut() {
                let all: Vec<_> = p.assets.iter_mut()
                    .chain(p.asset_groups.iter_mut().flat_map(|g| g.assets.iter_mut()))
                    .collect();
                for a in all {
                    if a.id == asset_id {
                        a.documents.push(doc.clone());
                        return;
                    }
                }
            }
        });
    };

    if view_mode == ViewMode::Grid {
        let asset_for_click = asset.clone();
        view! {
            <div class="asset-grid-card" on:click=move |_| on_select.run(asset_for_click.clone())>
                <div class="asset-grid-portfolio">{pname.clone()}</div>
                <img class="asset-grid-image" src={image_url.clone()} alt={a_name.clone()} />
                <div class="asset-grid-name">{a_name.clone()}</div>
            </div>
        }.into_any()
    } else {
    view! {
        <div class="ai-item" class:ai-item-expanded={move || expanded_detail.get()}>
            // ── Collapsed header row ──────────────────────────────────
            <div class="ai-header" on:click=move |_| {
                if !editing.get() { set_expanded_detail.update(|v| *v = !*v); }
            }>
                <span class="ai-arrow">{move || if expanded_detail.get() { "▲" } else { "▼" }}</span>
                <img class="ai-thumb" src={image_url.clone()} alt={a_name.clone()} />
                <div class="ai-header-info">
                    {move || if editing.get() {
                        view! {
                            <div class="asset-edit-form" on:click=|ev| ev.stop_propagation()>
                                <input class="pf-edit-input" placeholder="Name"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| set_edit_name.set(event_target_value(&ev)) />
                                <input class="pf-edit-input" placeholder="Description"
                                    prop:value=move || edit_desc.get()
                                    on:input=move |ev| set_edit_desc.set(event_target_value(&ev)) />
                                <input class="pf-edit-input" placeholder="Location / Address"
                                    prop:value=move || edit_loc.get()
                                    on:input=move |ev| set_edit_loc.set(event_target_value(&ev)) />
                                <div class="asset-edit-actions">
                                    <button class="pf-edit-save" on:click=save_edit>"✔ Save"</button>
                                    <button class="pf-edit-cancel" on:click=move |_| set_editing.set(false)>"✕"</button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <span class="ai-name">{a_name.clone()}</span>
                            <span class="ai-addr">{a_addr.clone()}</span>
                        }.into_any()
                    }}
                </div>
                <div class="pf-list-actions" on:click=|ev| ev.stop_propagation()>
                    {move || if can_edit && edit_mode.map(|s| s.get()).unwrap_or(true) {
                        view! {
                            <button class="pf-action-btn pf-edit-btn"
                                class:active=move || editing.get()
                                on:click=move |_| set_editing.update(|v| *v = !*v)
                            >"✎"</button>
                        }.into_any()
                    } else { ().into_any() }}
                </div>
            </div>

            // ── Expanded detail panel ─────────────────────────────────
            {move || if expanded_detail.get() {
                let docs_for_strip = docs.clone();
                let docs_for_modal = docs.clone();
                let modal_title    = asset_name_for_modal.clone();
                let add_cb = if can_edit { Some(Callback::new(move |name: String| { add_doc(name); })) } else { None };
                view! {
                    <div class="ai-detail-panel" on:click=|ev| ev.stop_propagation()>

                        // ── Details + Stats two-column ────────────────
                        <div class="ai-detail-cols">
                            // Left: details
                            <div class="ai-detail-section">
                                <div class="ai-detail-heading">"Details"</div>
                                <div class="ai-detail-row">
                                    <span class="ai-detail-lbl">"Type"</span>
                                    <span class="ai-detail-val">{a_type.clone()}</span>
                                </div>
                                <div class="ai-detail-row">
                                    <span class="ai-detail-lbl">"Status"</span>
                                    <span class="ai-detail-val">{a_status.clone()}</span>
                                </div>
                                <div class="ai-detail-row">
                                    <span class="ai-detail-lbl">"Portfolio"</span>
                                    <span class="ai-detail-val">{pname.clone()}</span>
                                </div>
                                <div class="ai-detail-row">
                                    <span class="ai-detail-lbl">"Location"</span>
                                    <span class="ai-detail-val">{a_addr.clone()}</span>
                                </div>
                                <div class="ai-detail-row">
                                    <span class="ai-detail-lbl">"Acquired"</span>
                                    <span class="ai-detail-val">{a_purchase_date.clone()}</span>
                                </div>
                                {if !a_desc.is_empty() && a_desc != "—" {
                                    view! {
                                        <div class="ai-detail-row">
                                            <span class="ai-detail-lbl">"Notes"</span>
                                            <span class="ai-detail-val ai-detail-notes">{a_desc.clone()}</span>
                                        </div>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>

                            // Right: pricing & revenue stats
                            <div class="ai-detail-section">
                                <div class="ai-detail-heading">"Pricing & Revenue"</div>
                                <div class="ai-stat-grid">
                                    <div class="ai-stat">
                                        <span class="ai-stat-lbl">"Purchase Value"</span>
                                        <span class="ai-stat-val">
                                            {format!("${:.2}M", a_purchase_val / 1_000_000.0)}
                                        </span>
                                    </div>
                                    <div class="ai-stat">
                                        <span class="ai-stat-lbl">"Current Value"</span>
                                        <span class="ai-stat-val ai-stat-highlight">
                                            {format!("${:.2}M", a_current_val / 1_000_000.0)}
                                        </span>
                                    </div>
                                    <div class="ai-stat">
                                        <span class="ai-stat-lbl">"Profit / Loss"</span>
                                        <span class={format!("ai-stat-val {}", a_pl_cls)}>
                                            {format!("${:+.0}K ({:+.1}%)", a_pl / 1000.0, a_pl_pct)}
                                        </span>
                                    </div>
                                    <div class="ai-stat">
                                        <span class="ai-stat-lbl">"Revenue"</span>
                                        <span class="ai-stat-val ai-stat-highlight">
                                            {format!("${:.0}K", a_revenue / 1000.0)}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // ── Documents horizontal strip ────────────────
                        <div class="ai-docs-section">
                            <div class="ai-docs-heading-row">
                                <span class="ai-detail-heading">"Documents"</span>
                                <div class="ai-docs-sort-btns">
                                    <button class="ai-docs-sort-btn"
                                        class:active=move || doc_sort.get() == 0
                                        on:click=move |_| set_doc_sort.set(0)>
                                        "Recent"
                                    </button>
                                    <button class="ai-docs-sort-btn"
                                        class:active=move || doc_sort.get() == 1
                                        on:click=move |_| set_doc_sort.set(1)>
                                        "Name"
                                    </button>
                                    {if can_edit {
                                        view! {
                                            <button class="ai-docs-sort-btn ai-docs-add-btn"
                                                on:click=move |_| set_show_doc_modal.set(true)>
                                                "+ Add"
                                            </button>
                                        }.into_any()
                                    } else { ().into_any() }}
                                </div>
                            </div>
                            {move || {
                                let mut sorted_docs = docs_for_strip.clone();
                                if doc_sort.get() == 1 {
                                    sorted_docs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                                }
                                // Recent order is insertion order (already newest-last in seed)
                                if sorted_docs.is_empty() {
                                    view! { <div class="ai-docs-empty">"No documents attached"</div> }.into_any()
                                } else {
                                    view! {
                                        <div class="ai-docs-strip">
                                            {sorted_docs.into_iter().map(|doc| {
                                                let icon  = document_icon(&doc.file_type);
                                                let ft    = doc.file_type.to_uppercase();
                                                let dname = doc.name.clone();
                                                let doc_for_view = doc.clone();
                                                let (viewing, set_viewing) = signal(false);
                                                view! {
                                                    <div class="ai-doc-card" on:click=move |_| set_viewing.set(true)>
                                                        <span class="ai-doc-card-icon">{icon}</span>
                                                        <span class="ai-doc-card-name">{dname}</span>
                                                        <span class="ai-doc-card-ft">{ft}</span>
                                                    </div>
                                                    {move || if viewing.get() {
                                                        let d = doc_for_view.clone();
                                                        view! {
                                                            <div class="doc-modal-overlay" on:click=move |_| set_viewing.set(false)>
                                                                <div class="doc-modal" on:click=|ev| ev.stop_propagation()>
                                                                    <DocumentViewer
                                                                        doc={d.clone()}
                                                                        on_close=move || set_viewing.set(false)
                                                                        can_edit={can_edit}
                                                                    />
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else { ().into_any() }}
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // Doc modal for adding new
                    {move || if show_doc_modal.get() {
                        let ds = docs_for_modal.clone();
                        let mt = modal_title.clone();
                        let ac = add_cb.clone();
                        view! {
                            <DocModal
                                docs={ds}
                                title={mt}
                                on_close=move || set_show_doc_modal.set(false)
                                can_edit={can_edit}
                                on_add={ac}
                            />
                        }.into_any()
                    } else { ().into_any() }}
                }.into_any()
            } else { ().into_any() }}
        </div>
    }.into_any()
    }
}

#[component]
fn AssetDetailView(asset: Asset, on_close: Callback<()>) -> impl IntoView {
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
    let pl_class = if asset.profit_loss >= 0.0 { "positive" } else { "negative" };

    view! {
        <div class="asset-detail-overlay" on:click=move |_| on_close.run(())>
            <div class="asset-detail" on:click=|ev| ev.stop_propagation()>
                <div class="asset-detail-header">
                    <div class="asset-detail-icon">{icon}</div>
                    <div class="asset-detail-title">{asset.name}</div>
                    <button class="asset-detail-close" on:click=move |_| on_close.run(())>"✕"</button>
                </div>
                <div class="asset-detail-body">
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Type"</span>
                        <span class="asset-detail-value">{format!("{:?}", asset.asset_type)}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Location"</span>
                        <span class="asset-detail-value">{asset.location.clone().unwrap_or_else(|| "—".to_string())}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Current Value"</span>
                        <span class="asset-detail-value">{format!("${:.2}M", asset.current_value / 1000000.0)}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Profit/Loss"</span>
                        <span class={format!("asset-detail-value {}", pl_class)}>{format!("${:+.0}K", asset.profit_loss / 1000.0)}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Organization"</span>
                        <span class="asset-detail-value">{asset.organization_id.map(|id| id.to_string()).unwrap_or_else(|| "Unassigned".to_string())}</span>
                    </div>
                    <div class="asset-detail-row">
                        <span class="asset-detail-label">"Status"</span>
                        <span class="asset-detail-value">{format!("{:?}", asset.status)}</span>
                    </div>
                    <div class="asset-detail-images">
                        {if asset.images.is_empty() {
                            view! { <div class="asset-detail-no-image">"No images"</div> }.into_any()
                        } else {
                            asset.images.into_iter().map(|url| view! {
                                <img class="asset-detail-img" src={url} alt="Asset image" />
                            }).collect::<Vec<_>>().into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

// Helper functions to create mock data
fn create_mock_asset(name: &str, asset_type: AssetType, purchase: f64, current: f64) -> Asset {
    let id = Uuid::new_v4();
    let image_url = asset_placeholder_url(&asset_type, name);
    let docs = vec![
        ("Title Deed", "pdf"),
        ("Inspection Report", "pdf"),
        ("Valuation", "xlsx"),
        ("Photos", "zip"),
        ("Contract", "docx"),
        ("Insurance", "pdf"),
        ("Notes", "txt"),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (n, ext))| crate::models::Document {
        id: Uuid::new_v4(),
        name: format!("{} {}", n, i + 1),
        file_type: ext.to_string(),
        url: "#".to_string(),
        uploaded_at: chrono::Utc::now(),
        uploaded_by: Uuid::nil(),
    })
    .collect();
    Asset {
        id,
        name: name.to_string(),
        description: Some(format!("Open Rose Rental Duplex 112, Open Rose Court, Coolangatta, QLD, 4269.").to_string()),
        asset_type,
        location: Some("Coolangatta, QLD, 4269".to_string()),
        organization_id: None,
        purchase_value: purchase,
        current_value: current,
        profit_loss: current - purchase,
        profit_loss_percent: ((current - purchase) / purchase) * 100.0,
        revenue: 0.0,
        purchase_date: chrono::Utc::now(),
        images: vec![image_url],
        documents: docs,
        tags: vec![],
        status: AssetStatus::Active,
        metadata: serde_json::json!({}),
        assigned_workers: vec![],
        quick_sale_enabled: false,
        notification_settings: vec![],
    }
}

fn document_icon(file_type: &str) -> &'static str {
    match file_type.to_lowercase().as_str() {
        "pdf" => "📕",
        "doc" | "docx" => "📘",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "txt" | "md" | "rs" | "js" | "ts" | "html" | "css" => "📄",
        "zip" | "rar" | "7z" | "tar" => "🗜️",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" => "🖼️",
        "mp4" | "mov" | "avi" | "mkv" => "🎬",
        "mp3" | "wav" | "flac" => "🎵",
        _ => "📎",
    }
}

fn shorthand_name(name: &str) -> String {
    if name.len() <= 16 {
        name.to_string()
    } else {
        format!("{}...", &name[..13])
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

/// Generate mock document content for the in-app viewer based on name and type.
fn mock_doc_content(name: &str, file_type: &str) -> String {
    match file_type.to_lowercase().as_str() {
        "pdf" => format!(
"DOCUMENT: {name}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Generated: {date}
Reference: DOC-{ref_num}
Status: ACTIVE

SUMMARY
This document serves as an official record pertaining to {name}.
All details contained herein have been verified and are accurate
as of the date of generation.

CONTENT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Section 1 — Overview
This section provides a high-level summary of the subject matter
described by this document. All parties are advised to review
the complete contents before proceeding.

Section 2 — Details
Full legal description and relevant information specific to the
named subject has been recorded. Supporting evidence is appended
at the rear of this document.

Section 3 — Certification
This document has been certified and notarised. Any alterations
render this document void. Contact the issuing authority for
certified copies.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page 1 of 1  |  CONFIDENTIAL",
            name = name,
            date = "22 Jun 2025",
            ref_num = "28471",
        ),
        "docx" => format!(
"{name}

Prepared by: Carly Asset Management
Date: 22 June 2025
Version: 1.0

──────────────────────────────────────

INTRODUCTION

This document outlines the key terms and conditions associated
with {name}. It has been prepared in accordance with applicable
regulations and internal policy.

MAIN BODY

The following details apply to the subject of this document:

  • All parties have been duly notified of their obligations.
  • The effective date is confirmed as 1 January 2025.
  • Terms are binding for a period of 12 months unless varied.
  • Renewal is subject to mutual agreement in writing.

SIGNATURE BLOCK

Authorised by: ________________________
Position:      Portfolio Manager
Date:          22 / 06 / 2025",
            name = name,
        ),
        "xlsx" => format!(
"┌─────────────────────────────────────────────────────────┐
│  {name:<55}│
│  Generated: 22 Jun 2025                                 │
├───────────────────┬──────────────┬──────────────────────┤
│  Description      │  Value       │  Notes               │
├───────────────────┼──────────────┼──────────────────────┤
│  Opening Balance  │  $1,200,000  │  FY2024              │
│  Acquisitions     │  $340,000    │  Q1-Q2               │
│  Disposals        │  -$80,000    │  Q3                  │
│  Revaluations     │  $120,000    │  Per valuer report   │
│  Closing Balance  │  $1,580,000  │  FY2025              │
├───────────────────┼──────────────┼──────────────────────┤
│  Net Change       │  +$380,000   │  +31.7%              │
└───────────────────┴──────────────┴──────────────────────┘

  Notes:
  All figures are in AUD. Subject to audit adjustment.
  Prepared by Finance — Internal Use Only.",
            name = name,
        ),
        "txt" => format!(
"Document: {name}
Date: 22 June 2025

This is a plain-text record associated with the above document.
No special formatting is required for this file type.

Key points:
- Document is current as of the date above.
- Retain for a minimum of 7 years per policy.
- Any queries should be directed to the portfolio manager.",
            name = name,
        ),
        _ => format!("Document: {name}\n\nNo preview available for this file type ({file_type}).",
            name = name, file_type = file_type),
    }
}

/// Document list modal — multi-tab viewer: open multiple docs simultaneously.
/// Tabs are pinned at the top; the list is always accessible via the "List" tab.
#[component]
pub fn DocModal(
    docs: Vec<Document>,
    title: String,
    on_close: impl Fn() + 'static,
    can_edit: bool,
    on_add: Option<Callback<String>>,
) -> impl IntoView {
    // open_tabs: vec of (tab_id, Document); tab_id=0 is reserved for the list tab
    let (open_tabs, set_open_tabs) = signal::<Vec<(u32, Document)>>(vec![]);
    let (active_tab, set_active_tab) = signal::<u32>(0); // 0 = list view
    let (next_id, set_next_id) = signal(1u32);
    let (new_doc_name, set_new_doc_name) = signal(String::new());
    let docs_sig = StoredValue::new(docs);
    let title_stored = StoredValue::new(title);
    let on_close = std::rc::Rc::new(on_close);
    let on_close2 = on_close.clone();

    let open_doc_tab = move |doc: Document| {
        // don't duplicate — if already open, switch to it
        let existing = open_tabs.get().into_iter().find(|(_, d)| d.id == doc.id).map(|(id, _)| id);
        if let Some(id) = existing {
            set_active_tab.set(id);
            return;
        }
        let id = next_id.get();
        set_next_id.set(id + 1);
        set_open_tabs.update(|v| v.push((id, doc)));
        set_active_tab.set(id);
    };

    let close_tab = move |tid: u32| {
        set_open_tabs.update(|v| v.retain(|(id, _)| *id != tid));
        // fall back to list if this was the active tab
        set_active_tab.update(|cur| { if *cur == tid { *cur = 0; } });
    };

    view! {
        <div class="doc-modal-overlay" on:click=move |_| on_close()>
            <div class="doc-modal doc-modal-tabbed" on:click=|ev| ev.stop_propagation()>

                // ── Modal header ───────────────────────────────────────
                <div class="doc-modal-header">
                    <span class="doc-modal-title">"📄 " {title_stored.get_value()}</span>
                    <button class="doc-modal-close" on:click=move |_| on_close2()>"✕"</button>
                </div>

                // ── Tab strip (always visible at top) ──────────────────
                <div class="dv-tab-strip">
                    // List tab (always present)
                    <div class="dv-tab"
                        class:dv-tab-active=move || active_tab.get() == 0
                        on:click=move |_| set_active_tab.set(0)>
                        <span class="dv-tab-icon">"☰"</span>
                        <span class="dv-tab-name">"List"</span>
                    </div>
                    // Open document tabs
                    {move || open_tabs.get().into_iter().map(|(tid, doc)| {
                        let icon  = document_icon(&doc.file_type);
                        let dname = shorthand_name(&doc.name);
                        view! {
                            <div class="dv-tab"
                                class:dv-tab-active=move || active_tab.get() == tid
                                on:click=move |_| set_active_tab.set(tid)>
                                <span class="dv-tab-icon">{icon}</span>
                                <span class="dv-tab-name">{dname}</span>
                                <button class="dv-tab-close"
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        close_tab(tid);
                                    }>"✕"</button>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                // ── Panel: list view (tab 0) ───────────────────────────
                {move || if active_tab.get() == 0 {
                    let on_add_cb = on_add.clone();
                    view! {
                        <div class="doc-modal-body">
                            <div class="doc-modal-list">
                                {docs_sig.get_value().into_iter().map(|doc| {
                                    let icon = document_icon(&doc.file_type);
                                    let ft   = doc.file_type.to_uppercase();
                                    let doc_for_open = doc.clone();
                                    view! {
                                        <div class="doc-modal-row">
                                            <span class="doc-modal-icon">{icon}</span>
                                            <div class="doc-modal-info">
                                                <span class="doc-modal-name">{doc.name.clone()}</span>
                                                <span class="doc-modal-ft">{ft}</span>
                                            </div>
                                            <button class="doc-modal-open-btn"
                                                on:click=move |_| open_doc_tab(doc_for_open.clone())>
                                                "Open"
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            {if can_edit {
                                view! {
                                    <div class="doc-modal-add-row">
                                        <input class="doc-modal-add-input" type="text"
                                            placeholder="New document name…"
                                            prop:value=move || new_doc_name.get()
                                            on:input=move |ev| set_new_doc_name.set(event_target_value(&ev)) />
                                        <button class="doc-modal-add-btn"
                                            on:click=move |_| {
                                                let n = new_doc_name.get();
                                                if !n.trim().is_empty() {
                                                    if let Some(cb) = &on_add_cb { cb.run(n); }
                                                    set_new_doc_name.set(String::new());
                                                }
                                            }>
                                            "+ Add"
                                        </button>
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // ── Panel: document viewer tabs ────────────────────────
                {move || {
                    let cur = active_tab.get();
                    open_tabs.get().into_iter().filter_map(|(tid, doc)| {
                        if tid != cur { return None; }
                        Some(view! {
                            <DocumentViewer
                                doc={doc}
                                on_close=move || close_tab(tid)
                                can_edit={can_edit}
                            />
                        })
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

/// In-app document viewer — sticky toolbar, zoom, edit mode, inline editing, image popup.
#[component]
pub fn DocumentViewer(
    doc: Document,
    on_close: impl Fn() + 'static,
    #[prop(default = false)] can_edit: bool,
) -> impl IntoView {
    let initial_content = mock_doc_content(&doc.name, &doc.file_type);
    let icon      = document_icon(&doc.file_type);
    let ft        = doc.file_type.to_uppercase();
    let is_sheet  = doc.file_type == "xlsx" || doc.file_type == "csv";
    let doc_name  = StoredValue::new(doc.name.clone());

    // viewer state
    let (zoom, set_zoom)         = signal(100u32);       // percent
    let (edit_mode, set_edit_mode) = signal(false);
    let (content, set_content)   = signal(initial_content);
    // image popup: Some((x_px, y_px))
    let (img_popup, set_img_popup) = signal::<Option<(i32, i32)>>(None);
    let (link_val, set_link_val) = signal(String::new());

    let on_close = std::rc::Rc::new(on_close);
    let on_close_toolbar = on_close.clone();

    view! {
        <div class="docviewer">
            // ── Sticky toolbar ────────────────────────────────────────
            <div class="docviewer-toolbar">
                <span class="docviewer-icon">{icon}</span>
                <span class="docviewer-name">{doc_name.get_value()}</span>
                <span class="docviewer-ft">{ft}</span>

                // Zoom controls
                <div class="dv-zoom-group">
                    <button class="dv-toolbar-btn"
                        on:click=move |_| set_zoom.update(|z| *z = (*z).saturating_sub(10).max(50))>
                        "−"
                    </button>
                    <span class="dv-zoom-label">{move || format!("{}%", zoom.get())}</span>
                    <button class="dv-toolbar-btn"
                        on:click=move |_| set_zoom.update(|z| *z = (*z + 10).min(300))>
                        "+"
                    </button>
                    <button class="dv-toolbar-btn"
                        on:click=move |_| set_zoom.set(100)>
                        "⟳"
                    </button>
                </div>

                // Edit toggle (only when can_edit)
                {if can_edit {
                    view! {
                        <button class="dv-toolbar-btn dv-edit-toggle"
                            class:dv-edit-active=move || edit_mode.get()
                            on:click=move |_| set_edit_mode.update(|v| *v = !*v)>
                            {move || if edit_mode.get() { "👁 Read" } else { "✎ Edit" }}
                        </button>
                    }.into_any()
                } else { ().into_any() }}

                <button class="docviewer-back" on:click=move |_| on_close_toolbar()>"← Back"</button>
            </div>

            // ── Document body ─────────────────────────────────────────
            <div
                class={move || if is_sheet { "docviewer-body docviewer-sheet".to_string() } else { "docviewer-body".to_string() }}
                style=move || format!("font-size: {}%;", zoom.get())
                on:click=move |_| { if img_popup.get().is_some() { set_img_popup.set(None); } }
            >
                // Image area (shown for image-type docs or as a doc header image)
                {if can_edit {
                    view! {
                        <div class="dv-image-row">
                            <div
                                class="dv-doc-image-placeholder"
                                class:dv-editable=move || edit_mode.get()
                                on:click=move |ev: leptos::ev::MouseEvent| {
                                    if edit_mode.get() {
                                        ev.stop_propagation();
                                        set_img_popup.set(Some((ev.client_x(), ev.client_y())));
                                    }
                                }
                            >
                                {move || if edit_mode.get() {
                                    view! { <span class="dv-img-hint">"🖼 Click to set image"</span> }.into_any()
                                } else { view! { <span class="dv-img-hint dv-img-muted">"🖼"</span> }.into_any() }}
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Image option popup (appears at cursor position)
                {move || if let Some((cx, cy)) = img_popup.get() {
                    view! {
                        <div class="dv-img-popup"
                            style=move || format!("left:{}px;top:{}px;", cx, cy)
                            on:click=|ev| ev.stop_propagation()>
                            <div class="dv-img-popup-opt"
                                on:click=move |_| {
                                    // Simulate upload — in a real app this opens a file picker
                                    set_img_popup.set(None);
                                }
                            >
                                <span class="dv-img-opt-icon">"📁"</span>
                                <span>"Upload"</span>
                            </div>
                            <div class="dv-img-popup-opt">
                                <span class="dv-img-opt-icon">"🔗"</span>
                                <input
                                    class="dv-img-link-input"
                                    placeholder="Paste URL…"
                                    prop:value=move || link_val.get()
                                    on:input=move |ev| set_link_val.set(event_target_value(&ev))
                                    on:click=|ev| ev.stop_propagation()
                                />
                                <button class="dv-img-link-ok"
                                    on:click=move |_| { set_img_popup.set(None); }>
                                    "OK"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                // Text content — editable textarea in edit mode, pre otherwise
                {move || if edit_mode.get() {
                    view! {
                        <textarea
                            class="docviewer-content dv-editable-text"
                            prop:value=move || content.get()
                            on:input=move |ev| set_content.set(event_target_value(&ev))
                        />
                    }.into_any()
                } else {
                    view! {
                        <pre class="docviewer-content">{move || content.get()}</pre>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
