//! Private sample submissions from CLI setup or an explicit request.

use vastlint_core::ValidationResult;

const CONTRIBUTE_ENDPOINT: &str = "https://vastlint.org/api/samples";
const MAX_SAMPLE_BYTES: usize = 512 * 1024;

pub fn body(xml: &str, result: &ValidationResult) -> Option<String> {
    if xml.len() > MAX_SAMPLE_BYTES {
        return None;
    }
    let version = result
        .version
        .best()
        .map(|v| format!("\"{}\"", v.as_str()))
        .unwrap_or_else(|| "null".to_owned());
    let body = format!(
        "{{\"xml\":\"{}\",\"source\":\"cli\",\"version\":{},\"summary\":{{\"errors\":{},\"warnings\":{},\"infos\":{}}}}}",
        crate::json_escape(xml), version, result.summary.errors, result.summary.warnings, result.summary.infos
    );
    (body.len() <= MAX_SAMPLE_BYTES).then_some(body)
}

pub fn submit(body: &str) {
    let _ = ureq::post(CONTRIBUTE_ENDPOINT)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(2))
        .send_string(body);
}
