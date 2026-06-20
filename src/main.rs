use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::Router;
        use farley::app::{shell, App};
        use farley::pages::email_valid::{email_valid_page, api_signup, api_validate, api_login, api_stats};
        use leptos::config::get_configuration;
        use leptos_axum::{generate_route_list, LeptosRoutes};
        use tracing::info;

        #[tokio::main]
        async fn main() {
            tracing_subscriber::fmt().init();

            info!("Starting Farley server...");

            let conf = get_configuration(None).unwrap();
            let addr = conf.leptos_options.site_addr;
            let leptos_options = conf.leptos_options;
            let routes = generate_route_list(App);

            let app = Router::new()
                .route("/emailvalid", axum::routing::get(email_valid_page))
                .route("/api/signup", axum::routing::post(api_signup))
                .route("/api/validate", axum::routing::post(api_validate))
                .route("/api/login", axum::routing::post(api_login))
                .route("/api/stats", axum::routing::get(api_stats))
                .leptos_routes(&leptos_options, routes, {
                    let leptos_options = leptos_options.clone();
                    move || shell(leptos_options.clone())
                })
                .fallback(leptos_axum::file_and_error_handler(shell))
                .with_state(leptos_options);

            info!("Server listening on http://{}", addr);
            info!("Email validation inbox at http://{}/emailvalid", addr);

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        }
    } else if #[cfg(feature = "hydrate")] {
        // Hydration is handled in lib.rs
        fn main() {}
    } else {
        fn main() {
            // Client-side rendering (CSR) entry point
            use farley::app::App;

            leptos::mount::mount_to_body(App);
        }
    }
}
