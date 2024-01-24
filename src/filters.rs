use chrono::{DateTime, Local, Timelike};
use num_rational::Rational32;


pub(crate) fn ratio2float(value: &Rational32, digits: usize) -> Result<String, askama::Error> {
    let num = *value.numer() as f64;
    let den = *value.denom() as f64;
    Ok(format!("{:.*}", digits, num / den))
}

pub(crate) fn ratio2float_owned(value: Rational32, digits: usize) -> Result<String, askama::Error> {
    ratio2float(&value, digits)
}

pub(crate) fn ratio2floatraw(value: &Rational32) -> Result<String, askama::Error> {
    let num = *value.numer() as f64;
    let den = *value.denom() as f64;
    Ok(format!("{}", num / den))
}

pub(crate) fn unix_timestamp_ms(timestamp: &DateTime<Local>) -> Result<i64, askama::Error> {
    Ok(timestamp.timestamp() * 1000)
}

pub(crate) fn time_of_day_ms(timestamp: &DateTime<Local>) -> Result<u32, askama::Error> {
    let time_of_day = timestamp.time();
    let ms = time_of_day.hour() * 60 * 60 * 1000
        + time_of_day.minute() * 60 * 1000
        + time_of_day.second() * 1000;
    Ok(ms)
}

pub(crate) fn time(timestamp: &DateTime<Local>) -> Result<String, askama::Error> {
    Ok(timestamp.format("%H:%M").to_string())
}
