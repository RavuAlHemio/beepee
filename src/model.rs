use chrono::{DateTime, Local};
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Measurement {
    pub id: i64,
    pub timestamp: DateTime<Local>,
    pub systolic: i32,
    pub diastolic: i32,
    pub pulse: i32,
    pub spo2: Option<i32>,
}
impl Measurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
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
        let mut state = serializer.serialize_struct("Measurement", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())?;
        state.serialize_field("time", &self.timestamp.format("%H:%M").to_string())?;
        state.serialize_field("systolic", &self.systolic)?;
        state.serialize_field("diastolic", &self.diastolic)?;
        state.serialize_field("pulse", &self.pulse)?;
        state.serialize_field("spo2", &self.spo2)?;
        state.end()
    }
}


#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct DailyMeasurements {
    pub date_string: String,
    pub morning: Option<Measurement>,
    pub midday: Option<Measurement>,
    pub evening: Option<Measurement>,
    pub other: Vec<Measurement>,
}
impl DailyMeasurements {
    pub fn new(
        date_string: String,
        morning: Option<Measurement>,
        midday: Option<Measurement>,
        evening: Option<Measurement>,
        other: Vec<Measurement>,
    ) -> DailyMeasurements {
        DailyMeasurements {
            date_string,
            morning,
            midday,
            evening,
            other,
        }
    }

    pub fn new_empty(date_string: String) -> DailyMeasurements {
        DailyMeasurements::new(date_string, None, None, None, Vec::new())
    }
}
