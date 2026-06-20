use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::components::tabs::TabsContainer;
use crate::pages::LoginPage;
use crate::stores::{AppStore, SearchStore, UndoRedoStore};
use leptos::prelude::*;
use leptos_meta::*;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="stylesheet" href="/pkg/farley.css"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context for managing stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Initialize stores as signals
    let app_store = RwSignal::new(AppStore::default());
    let search_store = RwSignal::new(SearchStore::default());
    let undo_store = RwSignal::new(UndoRedoStore::default());

    // Provide stores to all children
    provide_context(app_store);
    provide_context(search_store);
    provide_context(undo_store);

    let is_authenticated = Memo::new(move |_| app_store.get().is_authenticated);

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=move || view! {
                <div class="app-container">
                    <LoginPage />
                </div>
            }
        >
            <div class="app-container">
                // Main content area with collapsible tabs
                <main class="main-content">
                    <TabsContainer />
                </main>

                // Navigation Bar (fixed at top)
                <Navbar />

                // Footer
                <Footer />
            </div>
        </Show>
    }
}
