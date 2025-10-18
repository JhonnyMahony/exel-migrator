use std::fmt::Display;

use crate::app::notification::{AlertMessage, AlertProvider, Context};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{console::log_1, window, HtmlElement};
use yew::{function_component, html, use_context, use_state, Callback, Html};
use yew_hooks::{use_async, use_async_with_options, UseAsyncOptions};

use crate::app::{migrator::Migrator, notification::AlertType, settings::Settings};

mod migrator;
mod notification;
mod settings;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

async fn tauri_invoke<T: DeserializeOwned>(
    cmd: &str,
    args: JsValue,
    message: Context,
) -> Result<T, ()> {
    let invoke_result = invoke(cmd, args).await;

    match invoke_result {
        Ok(val) => Ok(from_value::<T>(val).unwrap()),
        Err(err_js) => {
            let msg = from_value::<String>(err_js).unwrap();
            message.set(AlertMessage::new(&msg, AlertType::Error));
            Err(())
        }
    }
}

enum Tabs {
    Application,
    Settings,
}

#[derive(Default, PartialEq, Deserialize, Serialize, Clone)]
struct Config {
    ip_address: String,
    db_name: String,
    db_username: String,
    db_password: String,
    mt_username: String,
    mt_password: String,
    is_initialized: bool,
}

mod connection_args {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Args {
        pub server_ip: String,
    }
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
struct FileData {
    file_path: String,
    sheets: Vec<String>,
}

#[derive(Serialize, PartialEq, Clone)]
enum Action {
    Create,
    Rewrite,
    Append,
    Delete,
}

impl From<&str> for Action {
    fn from(value: &str) -> Self {
        match value {
            "Create" => Action::Create,
            "Rewrite" => Action::Rewrite,
            "Append" => Action::Append,
            "Delete" => Action::Delete,
            _ => unreachable!(),
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Create => write!(f, "{}", "Create"),
            Action::Rewrite => write!(f, "{}", "Rewrite"),
            Action::Append => write!(f, "{}", "Append"),
            Action::Delete => write!(f, "{}", "Delete"),
        }
    }
}

#[function_component]
pub fn Main() -> Html {
    let context: Context = use_context::<Context>().expect("No AppContext found!");
    // vars
    let config_state = use_state(|| Config::default());
    let file_data = use_state(|| FileData::default());
    let action_state = use_state(|| Action::Create);
    let db_tables = use_state(|| Vec::new());
    let db_status = use_state(|| false);

    // tabs
    let current_tab = use_state(|| Tabs::Application);
    let on_application = {
        let current_tab = current_tab.clone();
        Callback::from(move |_| current_tab.set(Tabs::Application))
    };
    let on_settings = {
        let current_tab = current_tab.clone();
        Callback::from(move |_| {
            current_tab.set(Tabs::Settings);
        })
    };
    //db connection
    let connect_db = use_async({
        let config_state = config_state.clone();
        let db_tables = db_tables.clone();
        let db_status = db_status.clone();
        let context = context.clone();
        async move {
            let data = tauri_invoke(
                "connect_to_db",
                to_value(&connection_args::Args {
                    server_ip: config_state.ip_address.clone(),
                })
                .unwrap(),
                context.clone(),
            )
            .await?;
            let element = window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("overlay")
                .unwrap()
                .unchecked_into::<HtmlElement>();
            let _ = element.set_class_name("hidden");
            db_tables.set(data);
            context.set(AlertMessage::new(
                "База данних підключена",
                AlertType::Success,
            ));
            db_status.set(true);
            Ok::<(), ()>(())
        }
    });

    //config
    let _get_config = {
        let config_state = config_state.clone();
        let connect_db = connect_db.clone();
        use_async_with_options(
            async move {
                let config = tauri_invoke::<Config>("get_config", JsValue::NULL, context).await?;
                if config.is_initialized {
                    let element = window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("overlay")
                        .unwrap()
                        .unchecked_into::<HtmlElement>();
                    let _ = element.set_class_name("hidden");
                    connect_db.run();
                }
                config_state.set(config);
                Ok::<(), ()>(())
            },
            UseAsyncOptions::enable_auto(),
        )
    };
    let on_click_reconect = Callback::from(move |_| connect_db.run());

    html! {
        <body>
        <div>
        <div class={format!("db-status {}",if *db_status {"db-connected"}else{"db-disconnected"} )}>
            {if *db_status{"База данних підключена"}else{"Немає з'єднання з базою данних"}}
                    <svg onclick={on_click_reconect} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640"><path d="M544.1 256L552 256C565.3 256 576 245.3 576 232L576 88C576 78.3 570.2 69.5 561.2 65.8C552.2 62.1 541.9 64.2 535 71L483.3 122.8C439 86.1 382 64 320 64C191 64 84.3 159.4 66.6 283.5C64.1 301 76.2 317.2 93.7 319.7C111.2 322.2 127.4 310 129.9 292.6C143.2 199.5 223.3 128 320 128C364.4 128 405.2 143 437.7 168.3L391 215C384.1 221.9 382.1 232.2 385.8 241.2C389.5 250.2 398.3 256 408 256L544.1 256zM573.5 356.5C576 339 563.8 322.8 546.4 320.3C529 317.8 512.7 330 510.2 347.4C496.9 440.4 416.8 511.9 320.1 511.9C275.7 511.9 234.9 496.9 202.4 471.6L249 425C255.9 418.1 257.9 407.8 254.2 398.8C250.5 389.8 241.7 384 232 384L88 384C74.7 384 64 394.7 64 408L64 552C64 561.7 69.8 570.5 78.8 574.2C87.8 577.9 98.1 575.8 105 569L156.8 517.2C201 553.9 258 576 320 576C449 576 555.7 480.6 573.4 356.5z"/></svg>

        </div>
         <div class="tab-buttons">
            <button id="application-button" onclick={on_application}
        class={format!("tab-button {}", match *current_tab {Tabs::Application=>{"active"}, Tabs::Settings => {""}})}>{"Додаток"}
            </button>
            <button id="settings-button" onclick={on_settings}
        class={format!("tab-button {}", match *current_tab {Tabs::Application=>{""}, Tabs::Settings => {"active"}})}>{"Налаштування"}
            </button>
        </div>
        {match *current_tab {
            Tabs::Application=>html!{
                <Migrator db_tables={db_tables} file_data={file_data} action_state={action_state} />
            },
            Tabs::Settings=>html!{
                <Settings config={config_state.clone()} />
            }
        }}
        </div>
        </body>
    }
}
#[function_component]
pub fn App() -> Html {
    html! {
        <AlertProvider>
        <Main />
        </AlertProvider>
    }
}
