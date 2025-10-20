use std::rc::Rc;

use gloo_timers::callback::Timeout;
use uuid::Uuid;
use yew::{
    function_component, html, use_effect_with, use_reducer, use_state, Children, ContextProvider,
    Html, Properties, Reducible, UseReducerHandle, UseStateHandle,
};

#[derive(PartialEq, Clone)]
pub enum AlertType {
    Success,
    Alert,
    Error,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    id: Uuid,
    pub alert_type: AlertType,
    pub message: String,
}

#[function_component]
pub fn Notification(props: &Props) -> Html {
    let visible = use_state(|| true);
    {
        let visible = visible.clone();
        use_effect_with((), move |_| {
            let timeout = Timeout::new(5000, move || {
                visible.set(false);
            });
            // Keep timer alive until effect is dropped
            move || drop(timeout.cancel())
        });
    }

    if !*visible {
        return html! {}; // hide completely
    }
    let alert_icon = match props.alert_type {
        AlertType::Success => html! {
        <svg fill="green" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640">
            <path
                d="M320 576C178.6 576 64 461.4 64 320C64 178.6 178.6 64 320 64C461.4 64 576 178.6 576 320C576 461.4 461.4 576 320 576zM438 209.7C427.3 201.9 412.3 204.3 404.5 215L285.1 379.2L233 327.1C223.6 317.7 208.4 317.7 199.1 327.1C189.8 336.5 189.7 351.7 199.1 361L271.1 433C276.1 438 282.9 440.5 289.9 440C296.9 439.5 303.3 435.9 307.4 430.2L443.3 243.2C451.1 232.5 448.7 217.5 438 209.7z" />
        </svg>
        },
        AlertType::Alert => html! {
        <svg fill="orange" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640">
            <path
                d="M320 576C178.6 576 64 461.4 64 320C64 178.6 178.6 64 320 64C461.4 64 576 178.6 576 320C576 461.4 461.4 576 320 576zM320 384C302.3 384 288 398.3 288 416C288 433.7 302.3 448 320 448C337.7 448 352 433.7 352 416C352 398.3 337.7 384 320 384zM320 192C301.8 192 287.3 207.5 288.6 225.7L296 329.7C296.9 342.3 307.4 352 319.9 352C332.5 352 342.9 342.3 343.8 329.7L351.2 225.7C352.5 207.5 338.1 192 319.8 192z" />
        </svg>

        },
        AlertType::Error => html! {
        <svg fill="red" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640">
            <path
                d="M320 576C461.4 576 576 461.4 576 320C576 178.6 461.4 64 320 64C178.6 64 64 178.6 64 320C64 461.4 178.6 576 320 576zM231 231C240.4 221.6 255.6 221.6 264.9 231L319.9 286L374.9 231C384.3 221.6 399.5 221.6 408.8 231C418.1 240.4 418.2 255.6 408.8 264.9L353.8 319.9L408.8 374.9C418.2 384.3 418.2 399.5 408.8 408.8C399.4 418.1 384.2 418.2 374.9 408.8L319.9 353.8L264.9 408.8C255.5 418.2 240.3 418.2 231 408.8C221.7 399.4 221.6 384.2 231 374.9L286 319.9L231 264.9C221.6 255.5 221.6 240.3 231 231z" />
        </svg>
        },
    };

    html! {
    <div class={format!("toast {}", match props.alert_type{AlertType::Success=>"", AlertType::Alert=>"alert",
        AlertType::Error=>"error"})}>
        {alert_icon}{&props.message}
    </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct AuthProviderProps {
    pub children: Children,
}

#[derive(PartialEq, Clone)]
pub struct AlertMessage {
    pub id: Uuid,
    pub message: String,
    pub alert_type: AlertType,
}
impl AlertMessage {
    pub fn new(message: &str, alert_type: AlertType) -> Self {
        Self {
            id: Uuid::new_v4(),
            message: message.to_string(),
            alert_type,
        }
    }
}

#[derive(PartialEq)]
pub struct AlertMessages {
    pub messages: Vec<AlertMessage>,
}

pub enum AlertAction {
    Add(AlertMessage),
    Close(Uuid),
}

impl Reducible for AlertMessages {
    type Action = AlertAction;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut messages = self.messages.clone();
        match action {
            AlertAction::Add(alert) => {
                messages.push(alert);
                Rc::new(AlertMessages { messages })
            }
            AlertAction::Close(index) => {
                let messages = self
                    .messages
                    .clone()
                    .into_iter()
                    .filter(|n| n.id != index)
                    .collect();
                Rc::new(AlertMessages { messages })
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct AlertManager {
    pub alerts: UseReducerHandle<AlertMessages>,
    pub db_state: UseStateHandle<bool>,
    pub is_loading: UseStateHandle<bool>,
}

impl AlertManager {
    pub fn set(&self, msg: AlertMessage) {
        self.alerts.dispatch(AlertAction::Add(msg));
    }
}

#[function_component]
pub fn AlertProvider(props: &AuthProviderProps) -> Html {
    let alerts = use_reducer(|| AlertMessages {
        messages: Vec::new(),
    });
    let is_loading = use_state(|| false);
    let manager = AlertManager {
        alerts: alerts.clone(),
        db_state: use_state(|| false),
        is_loading: is_loading.clone(),
    };
    html! {
    <ContextProvider<AlertManager> context={manager}>

    {if *is_loading{html!{
        <div class="loader">
            <svg class="bike" viewBox="0 0 48 30" width="48px" height="30px">
        <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1">
            <g transform="translate(9.5,19)">
                <circle class="bike__tire" r="9" stroke-dasharray="56.549 56.549" />
                <g class="bike__spokes-spin" stroke-dasharray="31.416 31.416" stroke-dashoffset="-23.562">
                    <circle class="bike__spokes" r="5" />
                    <circle class="bike__spokes" r="5" transform="rotate(180,0,0)" />
                </g>
            </g>
            <g transform="translate(24,19)">
                <g class="bike__pedals-spin" stroke-dasharray="25.133 25.133" stroke-dashoffset="-21.991" transform="rotate(67.5,0,0)">
                    <circle class="bike__pedals" r="4" />
                    <circle class="bike__pedals" r="4" transform="rotate(180,0,0)" />
                </g>
            </g>
            <g transform="translate(38.5,19)">
                <circle class="bike__tire" r="9" stroke-dasharray="56.549 56.549" />
                <g class="bike__spokes-spin" stroke-dasharray="31.416 31.416" stroke-dashoffset="-23.562">
                    <circle class="bike__spokes" r="5" />
                    <circle class="bike__spokes" r="5" transform="rotate(180,0,0)" />
                </g>
            </g>
            <polyline class="bike__seat" points="14 3,18 3" stroke-dasharray="5 5" />
            <polyline class="bike__body" points="16 3,24 19,9.5 19,18 8,34 7,24 19" stroke-dasharray="79 79" />
            <path class="bike__handlebars" d="m30,2h6s1,0,1,1-1,1-1,1" stroke-dasharray="10 10" />
            <polyline class="bike__front" points="32.5 2,38.5 19" stroke-dasharray="19 19" />
        </g>
    </svg>
            </div>
        }}else{html!{}}}
        <div id="toastBox">
            { for alerts.messages.iter().map(|el|html!{
            <Notification key={el.id.to_string()} id={el.id} alert_type={el.alert_type.clone()}
                message={el.message.clone()} />})}
        </div>
        { for props.children.iter() }
    </ContextProvider<AlertManager>>

    }
}
