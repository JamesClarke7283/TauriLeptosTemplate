use futures::StreamExt;
use leptos::*;
use serde::{Deserialize, Serialize};
use tauri_sys::{event, tauri};

#[derive(Serialize)]
struct GreetCmdArgs {
    name: String,
}

#[derive(Serialize)]
struct EmitEventCmdArgs {
    num: u16,
}

#[derive(Debug, Deserialize)]
struct GreetEventRes {
    greeting: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct GenericEventRes {
    num: u16,
    message: String,
}

async fn greet(name: String) -> String {
    tauri::invoke("greet", &GreetCmdArgs { name })
        .await
        .unwrap()
}

async fn listen_on_greet_event() -> String {
    let event = event::once::<GreetEventRes>("greet-event").await.unwrap();
    log::debug!("Received greet-event {:#?}", event);
    event.payload.greeting
}

async fn emit_generic_event(num: u16) {
    tauri::invoke::<_, ()>("emit_event", &EmitEventCmdArgs { num })
        .await
        .unwrap();
}

async fn listen_on_generic_event(event_writer: WriteSignal<Vec<GenericEventRes>>) {
    let mut events = event::listen::<GenericEventRes>("generic-event")
        .await
        .unwrap();

    while let Some(event) = events.next().await {
        log::debug!("Received generic-event {:#?}", event);
        event_writer.update(|all_events| all_events.push(event.payload));
    }
}

#[component]
pub fn Counter(value: ReadSignal<i32>, set_value: WriteSignal<i32>) -> impl IntoView {
    view! {
        <div>
            <button on:click=move |_| set_value.set(0)>"Clear"</button>
            <button on:click=move |_| set_value.update(|value| *value -= 1)>"-1"</button>
            <span>"Value: " {move || value.get().to_string()} "!"</span>
            <button on:click=move |_| set_value.update(|value| *value += 1)>"+1"</button>
        </div>
    }
}

#[component]
pub fn Greeting(msg: ReadSignal<String>, greet_event_msg: ReadSignal<String>) -> impl IntoView {
    view! {
        <div>
            <p>{msg}</p>
            <p>{greet_event_msg}</p>
        </div>
    }
}

#[component]
pub fn GenericEvents(
    event_vec: ReadSignal<Vec<GenericEventRes>>,
    emit_event_action: Action<u16, ()>,
    event_counter: ReadSignal<u16>,
    set_event_counter: WriteSignal<u16>,
) -> impl IntoView {
    view! {
        <div>
            <button on:click=move |_| {
                emit_event_action.dispatch(event_counter.get());
                set_event_counter.set(event_counter.get() + 1);
            }>"Emit generic event"</button>

            <ul>
            <For each=move || event_vec.get().clone() key=|e| e.num  children=move |e: GenericEventRes| {
                view! {
                    <li>{e.message.clone()}</li>
                }
            } />

            </ul>
        </div>
    }
}

#[component]
pub fn SimpleCounter(name: String) -> impl IntoView {
    let (value, set_value) = create_signal(0);
    // Greet event, will clean-up once event is received.
    let (greet_event_msg, set_greet_event_msg) =
        create_signal("No `greet-event` from Tauri.".to_string());
    let greet_event_resource = create_local_resource(move || (), |_| listen_on_greet_event());
    let greet_event_msg_memo = create_memo(move |_| {
        set_greet_event_msg.set(
            greet_event_resource
                .get()
                .unwrap_or("Waiting for `greet-event` from Tauri.".to_string()),
        );
    });
    create_effect(move |_| greet_event_msg_memo);
    // Generic event, listening constantly.
    let (event_counter, set_event_counter) = create_signal(1u16);
    let (event_vec, set_event_vec) = create_signal::<Vec<GenericEventRes>>(vec![]);
    let emit_event_action = create_action(|num: &u16| emit_generic_event(*num));
    create_local_resource(move || set_event_vec, listen_on_generic_event);
    // Greet command response.
    let greet_resource = create_local_resource(move || name.to_owned(), greet);
    let (msg, set_msg) = create_signal("".to_string());
    create_effect(move |_| {
        set_msg.set(greet_resource.get().unwrap_or_else(|| "".to_string()));
    });

    view! {
        <div>
            <Counter value=value set_value=set_value />
            <Greeting msg=msg greet_event_msg=greet_event_msg />
            <GenericEvents event_vec=event_vec emit_event_action=emit_event_action event_counter=event_counter set_event_counter=set_event_counter />
        </div>
    }
}
