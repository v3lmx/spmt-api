use leptonic::prelude::*;
use leptos::*;

fn main() {
    leptos::mount_to_body(|cx| view! { cx, <App/> })
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! {cx,
        <Root default_theme=LeptonicTheme::default()>
            <TopBar/>
        </Root>
    }
}

#[component]
pub fn TopBar(cx: Scope) -> impl IntoView {
    view! {cx,
        <AppBar height=Height::Em(3.5) style="z-index: 1; background: var(--brand-color); color: white;">
            <H3 style="margin-left: 1em; color: white;">"spmt"</H3>
            // <H3>"spmt"</H3>
        </AppBar>
    }
}
