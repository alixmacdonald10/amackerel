use topcoat::{
    icon::{icon, iconify::iconify_icon},
    runtime::{Event, Signal},
    view::{attributes, component, view},
    Result,
};

use crate::components::toggle::toggle;

#[component]
pub async fn dark_mode_toggle(dark_mode: &Signal<bool>) -> Result {
    view! {
        <div class="fixed top-3 right-4 z-50">
            toggle(
                attrs: attributes! {
                    name="dark"
                    :checked=$(dark_mode.get())
                    @click=$(|_e: Event| dark_mode.toggle())
                },
                icon(data: iconify_icon!("hugeicons:dark-mode"), label: "Dark")
            )
        </div>
    }
}
