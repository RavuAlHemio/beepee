use std::collections::HashMap;

use num_rational::Rational64;
use serde_json;
use tera;


pub(crate) struct RatioToFloat;
impl tera::Filter for RatioToFloat {
    fn filter(&self, value: &serde_json::Value, args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        let digits = match args.get("digits") {
            Some(serde_json::Value::Number(n)) => n.as_u64().map(|u| u as usize),
            Some(_) => return Err(tera::Error::msg("\"digits\" must be a number")),
            None => None,
        };
        let vstr = if let Some(vs) = value.as_str() {
            vs.to_owned()
        } else {
            value.to_string()
        };
        let ratio: Rational64 = vstr.parse()
            .map_err(|pre| tera::Error::msg(pre))?;
        let num = *ratio.numer() as f64;
        let den = *ratio.denom() as f64;
        let out_text = if let Some(d) = digits {
            format!("{:.*}", d, num / den)
        } else {
            format!("{}", num / den)
        };
        Ok(serde_json::Value::from(out_text))
    }
}
