use leptos::prelude::*;

#[component]
pub fn ReportingPage() -> impl IntoView {
    let categories = vec![
        ("Documentation", "All documents, contracts, and reports", "📁"),
        ("Payslips", "Employee pay records and history", "💰"),
        ("Working Hours", "Timesheets and hour logs", "🕒"),
        ("Deeds of Ownership", "Title deeds and ownership certificates", "📜"),
        ("Registration", "Registration documents and renewals", "📝"),
        ("Delivery Notices", "Delivery dockets and receipts", "🚚"),
    ];

    view! {
        <div class="reporting-page">
            <div class="reporting-header">
                <h2 class="reporting-title">"Reporting"</h2>
                <div class="reporting-subtitle">"Fast access to all documents, payslips, working hours, deeds, registration, and delivery notices."</div>
            </div>
            <div class="reporting-grid">
                {categories.into_iter().map(|(title, desc, icon)| view! {
                    <div class="reporting-card">
                        <div class="reporting-card-icon">{icon}</div>
                        <div class="reporting-card-title">{title}</div>
                        <div class="reporting-card-desc">{desc}</div>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
