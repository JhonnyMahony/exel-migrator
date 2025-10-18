use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use web_sys::HtmlSelectElement;
use web_sys::{FormData, HtmlFormElement};
use yew::prelude::*;
use yew_hooks::use_async;

use crate::app::notification::{AlertMessage, AlertType, Context};
use crate::app::{tauri_invoke, Action, FileData};

#[derive(Serialize)]
struct ActionData {
    action: Action,
    file_path: String,
    sheet: String,
    range: String,
    table_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Args {
    action_data: ActionData,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub db_tables: UseStateHandle<Vec<String>>,
    pub action_state: UseStateHandle<Action>,
    pub file_data: UseStateHandle<FileData>,
}

#[function_component]
pub fn Migrator(props: &Props) -> Html {
    let context: Context = use_context::<Context>().expect("No AppContext found!");
    let form_ref = use_node_ref();

    let action_ref = use_node_ref();
    let on_change_action = {
        let action_state = props.action_state.clone();
        let action_ref = action_ref.clone();
        Callback::from(move |_| {
            let select_element = action_ref.cast::<HtmlSelectElement>().unwrap();
            action_state.set(select_element.value().as_str().into());
        })
    };
    let get_db_tables = use_async({
        let db_tables = props.db_tables.clone();
        let context = context.clone();
        async move {
            let data = tauri_invoke("get_tables", JsValue::NULL, context).await?;
            db_tables.set(data);
            Ok::<(), ()>(())
        }
    });

    let execute_action = use_async({
        let form_ref = form_ref.clone();
        let file_data = props.file_data.clone();
        let action_state = props.action_state.clone();
        let context = context.clone();
        async move {
            let form = form_ref.cast::<HtmlFormElement>().unwrap();
            let form_data = FormData::new_with_form(&form).unwrap();
            let action_data = ActionData {
                action: (*action_state).clone(),
                file_path: file_data.file_path.clone(),
                sheet: form_data.get("sheet").as_string().unwrap_or("".to_string()),
                range: form_data.get("range").as_string().unwrap_or("".to_string()),
                table_name: form_data.get("table_name").as_string().unwrap(),
            };

            tauri_invoke::<()>(
                "execute_action",
                to_value(&Args { action_data }).unwrap(),
                context.clone(),
            )
            .await?;

            get_db_tables.run();

            context.set(AlertMessage::new(
                "Дія успішно виконана",
                AlertType::Success,
            ));

            Ok::<(), ()>(())
        }
    });

    let on_submit_form = {
        let execute_action = execute_action.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            execute_action.run();
        })
    };

    // choose file
    let select_file = use_async({
        let file_data = props.file_data.clone();
        let context = context.clone();
        async move {
            let data = tauri_invoke("choose_file", JsValue::NULL, context.clone()).await?;
            file_data.set(data);
            context.set(AlertMessage::new("Файл успішно додано", AlertType::Success));

            Ok::<(), ()>(())
        }
    });
    let on_choose_file = Callback::from(move |_| {
        select_file.run();
    });

    html! {
            <form class="db-section"  ref={form_ref} onsubmit={on_submit_form}>
                <div class="created-by">

                    <p>{"@Розробник: Віталій Ковінський"}</p>
                </div>
                <div class="form-group">
                    <label>{"Дія"}</label>
                    <select ref={action_ref} onchange={on_change_action}
                        name="action">
                        <option selected={*props.action_state == Action::Create} value={Action::Create.to_string()}>{"Додати
                            таблицю"}</option>
                        <option selected={*props.action_state == Action::Rewrite} value={Action::Rewrite.to_string()}>
                            {"Перезаписати таблицю"}</option>
                        <option selected={*props.action_state == Action::Append} value={Action::Append.to_string()}>{"Доповнити
                            таблицю"}</option>
                        <option selected={*props.action_state == Action::Delete} value={Action::Delete.to_string()}>{"Видалити
                            таблицю"}</option>
                    </select>
                    <p>{"Яку дію ви хочете виконати?"}</p>
                </div>
                { if *props.action_state != Action::Delete{ html!{
            <>
                <div class="form-group">
                    <label>{format!("Оберіть файл {}",
                        props.file_data.file_path)}</label>
                    <button type="button"
                        onclick={on_choose_file}>{"Обрати"}</button>
                </div>
                <div class="form-group">
                    <label>{"Оберіть аркуш"}</label>
                    <select name="sheet">
                        {for props.file_data.sheets.iter().map(|sheet|{
                        html!{
                        <option>{sheet}</option>
                        }
                        })}
                    </select>
                    <p>{"На якому аркушы знаходиться таблиця для завантаження?"}
                    </p>
                </div>
                <div class="form-group">
                    <label>{"Діапазанон"}</label>
                    <input name="range" type="text" placeholder="A1:P256" />
                    <p>{"Де саме на аркуші знахядяться данні?"}</p>
                </div>
            </>
            }}else{html!{}}}

                {if *props.action_state == Action::Create{
            html!{

                <div class="form-group">
                    <label>{"Назва таблиці"}</label>
                    <input name="table_name" type="text" value="my_table" />
                    <p>{"Як назвати таблицю в базі данних?"} </p>
                </div>
            }
                }else{html!{
                                <div class="form-group">
                    <label>{"Оберіть таблицю в БД"}</label>
                    <select name="table_name">
                        {for props.db_tables.iter().map(|table|{
                        html!{
                        <option>{table}</option>
                        }
                        })}
                    </select>
                    <p>{"Над якою таблицею в базі данних виконивати дію?"}
                    </p>
                </div>


            }}}
                <button type="submit" class="convert-btn">{ match *props.action_state {
            Action::Create => "Додати таблицю",
            Action::Rewrite => "Перезаписати таблицю",
            Action::Append => "Доповнити таблицю",
            Action::Delete => "Видалити таблицю",
                }}</button>
            </form>
    }
}
