pub(crate) mod serde_datetime_local {
    use chrono::{DateTime, Local, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::Error as _;

    const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.fZ";

    pub fn serialize<S: Serializer>(value: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error> {
        let utc = value.with_timezone(&Utc);
        let string = utc.format(TIME_FORMAT).to_string();
        string.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Local>, D::Error> {
        let string = String::deserialize(deserializer)?;
        let naive_utc = NaiveDateTime::parse_from_str(&string, TIME_FORMAT)
            .map_err(|e| D::Error::custom(e))?;
        let utc = naive_utc.and_utc();
        Ok(utc.with_timezone(&Local))
    }
}

fn rat32_to_string(value: &num_rational::Rational32) -> String {
    format!("{}/{}", value.numer(), value.denom())
}

fn string_to_rat32(s: &str) -> Result<num_rational::Rational32, String> {
    let (num, denom) = if let Some((num_str, denom_str)) = s.split_once('/') {
        let num = num_str.parse()
            .map_err(|e| format!("failed to parse numerator {:?}: {}", num_str, e))?;
        let denom = denom_str.parse()
            .map_err(|e| format!("failed to parse denominator {:?}: {}", denom_str, e))?;
        (num, denom)
    } else {
        // assume numerator only
        let num = s.parse()
            .map_err(|e| format!("failed to parse lone numerator {:?}: {}", s, e))?;
        (num, 1)
    };
    if denom == 0 {
        return Err("denominator must not be zero".to_owned());
    }
    let rat = num_rational::Rational32::new(num, denom);
    Ok(rat)
}

pub(crate) mod serde_rat32 {
    use num_rational::Rational32;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::Error as _;

    pub fn serialize<S: Serializer>(value: &Rational32, serializer: S) -> Result<S::Ok, S::Error> {
        let string = super::rat32_to_string(value);
        string.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Rational32, D::Error> {
        let string = String::deserialize(deserializer)?;
        let rat = super::string_to_rat32(&string)
            .map_err(|e| D::Error::custom(e))?;
        Ok(rat)
    }
}

pub(crate) mod serde_rat32_opt {
    use num_rational::Rational32;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::Error as _;

    pub fn serialize<S: Serializer>(value: &Option<Rational32>, serializer: S) -> Result<S::Ok, S::Error> {
        let string = value.as_ref().map(|rat| super::rat32_to_string(rat));
        string.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Rational32>, D::Error> {
        let string: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = string {
            let rat = super::string_to_rat32(&s)
                .map_err(|e| D::Error::custom(e))?;
            Ok(Some(rat))
        } else {
            Ok(None)
        }
    }
}

