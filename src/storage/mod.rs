/// Portfolio persistence: RocksDB (server-side) + DashMap in-memory cache.
///
/// Only compiled when the `ssr` feature is active. The WASM/hydrate client
/// never touches this module – it reads state from the Leptos AppStore which
/// the server pre-populates via `load_portfolios` / `save_portfolio`.
#[cfg(feature = "ssr")]
pub mod db;

#[cfg(feature = "ssr")]
pub use db::{PortfolioStore, portfolio_store};
