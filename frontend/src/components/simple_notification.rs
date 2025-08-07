use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SimpleNotificationProps {
    pub message: String,
    pub notification_type: String,
    pub on_close: Callback<()>,
}

#[function_component]
pub fn SimpleNotification(props: &SimpleNotificationProps) -> Html {
    let on_close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    html! {
        <div class={format!("notification {}", props.notification_type)}>
            <div class="notification-content">
                <span class="notification-message">{&props.message}</span>
                <button class="notification-close" onclick={on_close}>{"×"}</button>
            </div>
        </div>
    }
}
