use rusty_v8 as v8;
use v8::{
    Context, ContextScope, CreateParams, FunctionCallbackArguments, FunctionTemplate, HandleScope,
    NewStringType, ReturnValue, Script,
};

pub struct ReactSSR {
    _isolate: v8::OwnedIsolate,
    _source: String,
}

// this method will be called from javascript, will be passed rendered string using ReactDOMServer
fn set_html(scope: &mut HandleScope, args: FunctionCallbackArguments, _rv: ReturnValue) {
    assert!(args.length() > 0);

    let html = args.get(0);
    assert!(html.is_string());
    let html = html.to_string(scope).unwrap();

    let global = scope.get_current_context().global(scope);
    let html_key = v8::String::new(scope, "html").unwrap();

    global.set(scope, html_key.into(), html.into());
}

impl ReactSSR {
    pub fn new(source: String) -> Self {
        // V8 must be initialized.
        v8::V8::assert_initialized();

        let isolate = v8::Isolate::new(CreateParams::default());

        Self {
            _isolate: isolate,
            _source: source,
        }
    }
}

impl ReactSSR {
    // TODO: add props as well

    // TODO: Add exception handling.
    pub fn render_to_string(&mut self) -> Option<String> {
        let isolate = &mut self._isolate;

        // Create new Handle scope.
        let scope = &mut HandleScope::new(isolate);
        let context = Context::new(scope);

        let global = context.global(scope);

        let scope = &mut ContextScope::new(scope, context);

        // Create Proxy
        let proxy_key = v8::String::new(scope, "proxy").unwrap();
        let proxy_val = v8::Object::new(scope);
        global.set(scope, proxy_key.into(), proxy_val.into());

        // add setHtml method on proxy that can be called from javascript and passed the html string rendered from
        let set_html_key = v8::String::new(scope, "setHtml").unwrap();
        let set_html_template = FunctionTemplate::new(scope, set_html);
        let set_html_val = set_html_template.get_function(scope).unwrap();
        proxy_val.set(scope, set_html_key.into(), set_html_val.into());

        let source =
            v8::String::new_from_utf8(scope, self._source.as_bytes(), NewStringType::Normal)
                .unwrap();

        let script = Script::compile(scope, source, None).unwrap();

        script.run(scope).and_then(|_| {
            // TODO: remove unwraps and handle errors.
            let html_key = v8::String::new(scope, "html").unwrap();
            let html_val = global.get(scope, html_key.into()).unwrap();
            let html_string = html_val.to_string(scope).unwrap();

            let html = html_string.to_rust_string_lossy(scope);
            Some(html)
        })
    }
}

// It initizes the V8 for the specific platform.
pub fn initialize() {
    // v8::V8::initialize();
    // TODO: Check if we should use some configured platform.
    let mut platform = v8::new_default_platform();

    // Initialize the platform.
    v8::V8::initialize_platform(platform.take().unwrap());

    // initialize the V8.
    v8::V8::initialize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_works() {
        initialize()
    }
}
