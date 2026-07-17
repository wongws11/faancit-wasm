use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{Event, HtmlElement, HtmlInputElement, Performance};

use crate::{get_faancit, get_jyutping};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document is unavailable"))?;
    let performance = window
        .performance()
        .ok_or_else(|| JsValue::from_str("performance timer is unavailable"))?;
    let input = document
        .get_element_by_id("input")
        .ok_or_else(|| JsValue::from_str("missing #input"))?
        .dyn_into::<HtmlInputElement>()?;
    let output = document
        .get_element_by_id("output")
        .ok_or_else(|| JsValue::from_str("missing #output"))?
        .dyn_into::<HtmlElement>()?;
    let pronunciations = document
        .get_element_by_id("pronunciations")
        .ok_or_else(|| JsValue::from_str("missing #pronunciations"))?
        .dyn_into::<HtmlElement>()?;

    let input_for_listener = input.clone();
    let listener = Closure::<dyn FnMut(Event)>::new(move |_| {
        render(
            &input_for_listener.value(),
            &output,
            &pronunciations,
            &performance,
        );
    });
    input.add_event_listener_with_callback("input", listener.as_ref().unchecked_ref())?;
    listener.forget();

    Ok(())
}

fn render(
    value: &str,
    output: &HtmlElement,
    pronunciations: &HtmlElement,
    performance: &Performance,
) {
    let mut characters = value.chars();
    let Some(upper) = characters.next() else {
        clear(output, pronunciations);
        return;
    };
    let Some(lower) = characters.next() else {
        clear(output, pronunciations);
        return;
    };
    if characters.next().is_some() {
        clear(output, pronunciations);
        return;
    }

    let started_at = performance.now();
    let (Some(upper), Some(lower)) = (
        get_jyutping(&upper.to_string()),
        get_jyutping(&lower.to_string()),
    ) else {
        clear(output, pronunciations);
        return;
    };
    let faancit = get_faancit(&upper, &lower);
    let elapsed = performance.now() - started_at;
    let timing = if elapsed < 1.0 {
        format!("Faancit lookup: {:.3} μs", elapsed * 1_000.0)
    } else {
        format!("Faancit lookup: {elapsed:.3} ms")
    };
    web_sys::console::log_1(&timing.into());

    output.set_inner_text(&faancit.pronunciation.to_string());
    let mut homophones = String::with_capacity(faancit.homophones.len() * 4);
    for character in faancit.homophones {
        homophones.push(character);
        homophones.push(' ');
    }
    pronunciations.set_inner_text(&homophones);

    if !faancit.diagnostics.is_empty() {
        web_sys::console::log_1(&faancit.diagnostics.into());
    }
}

fn clear(output: &HtmlElement, pronunciations: &HtmlElement) {
    output.set_inner_text("");
    pronunciations.set_inner_text("");
}
