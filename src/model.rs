use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Measurement {
    pub id: i64,
    pub timestamp: DateTime<FixedOffset>,
    pub systolic: i32,
    pub diastolic: i32,
    pub pulse: i32,
    pub spo2: Option<i32>,
}
impl Measurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<FixedOffset>,
        systolic: i32,
        diastolic: i32,
        pulse: i32,
        spo2: Option<i32>,
    ) -> Measurement {
        Measurement {
            id,
            timestamp,
            systolic,
            diastolic,
            pulse,
            spo2,
        }
    }
}
impl Serialize for Measurement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Measurement", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())?;
        state.serialize_field("systolic", &self.systolic)?;
        state.serialize_field("diastolic", &self.diastolic)?;
        state.serialize_field("pulse", &self.pulse)?;
        state.serialize_field("spo2", &self.spo2)?;
        state.end()
    }
}
