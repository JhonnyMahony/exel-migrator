use serde::Serialize;
use serde_wasm_bindgen::to_value;
use web_sys::{FormData, HtmlFormElement};
use yew::{
    function_component, html, use_context, use_effect_with, use_node_ref, Callback, Html,
    Properties, UseStateHandle,
};
use yew_hooks::use_async;

use crate::app::{
    notification::{AlertManager, AlertMessage, AlertType},
    tauri_invoke, Config,
};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub config: UseStateHandle<Config>,
}

#[derive(Serialize)]
struct Args {
    config: Config,
}

#[function_component]
pub fn Settings(props: &Props) -> Html {
    let context: AlertManager = use_context::<AlertManager>().expect("No AppContext found!");
    let settings_form = use_node_ref();
    let update_settings = {
        let settings_form = settings_form.clone();
        let context = context.clone();
        let config = props.config.clone();
        use_async(async move {
            let form = settings_form.cast::<HtmlFormElement>().unwrap();
            let form_data = FormData::new_with_form(&form).unwrap();
            let new_config = Config {
                ip_address: form_data.get("ip_address").as_string().unwrap(),
                db_name: form_data.get("db_name").as_string().unwrap(),
                db_username: form_data.get("db_username").as_string().unwrap(),
                db_password: form_data.get("db_password").as_string().unwrap(),
                mt_username: form_data.get("mt_username").as_string().unwrap(),
                mt_password: form_data.get("db_password").as_string().unwrap(),
                is_initialized: config.is_initialized,
            };

            let new_config = tauri_invoke(
                "update_config",
                to_value(&Args {
                    config: new_config.clone(),
                })
                .unwrap(),
                context.clone(),
            )
            .await?;
            context.set(AlertMessage::new(
                "Дані успішно оновлено",
                AlertType::Success,
            ));
            config.set(new_config);

            Ok::<(), ()>(())
        })
    };

    let on_submit = {
        let update_settings = update_settings.clone();
        Callback::from(move |e: yew::SubmitEvent| {
            e.prevent_default();
            update_settings.run()
        })
    };

    {
        let context = context.clone();
        use_effect_with(update_settings.loading, move |l1| {
            if *l1 {
                context.is_loading.set(true);
            } else {
                context.is_loading.set(false);
            }
        })
    }

    html! {
        <form ref={settings_form} onsubmit={on_submit} class="db-section">
            <label class="form-group-label">{"Сервер"}</label>
                <div class="form-group">
                    <label>{"IP-адреса серверу"}</label>
                    <input value={props.config.ip_address.clone()} name="ip_address" type="text" />
                </div>
            <label class="form-group-label">{"База данних"}</label>
                <div class="form-group">
                    <label>{"Назва БД"}</label>
                    <input value={props.config.db_name.clone()} name="db_name" type="text" />
                </div>
                <div class="form-group">
                    <label>{"Ім'я користувача БД"}</label>
                    <input value={props.config.db_username.clone()} name="db_username" type="text"  />
                </div>
                <div class="form-group">
                    <label>{"Пароль користувача БД"}</label>
                    <input value={props.config.db_password.clone()} name="db_password" type="text"  />
                </div>
            <label class="form-group-label">{"Metabase"}</label>
                <div class="form-group">
                    <label>{"Ім'я користувача MT"}</label>
                    <input value={props.config.mt_username.clone()} name="mt_username" type="text"  />
                </div>
                <div class="form-group">
                    <label>{"Пароль користувача МТ"}</label>
                    <input value={props.config.mt_password.clone()} name="mt_password" type="text"  />
                </div>
                <button type="submit" class="convert-btn">{"Оновити дані"}</button>
        </form>

    }
}
