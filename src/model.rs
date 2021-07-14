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
        state.serialize_field("time", &self.timestamp.format("%H:%M:%S").to_string())?;
        state.serialize_field("systolic", &self.systolic)?;
        state.serialize_field("diastolic", &self.diastolic)?;
        state.serialize_field("pulse", &self.pulse)?;
        state.serialize_field("spo2", &self.spo2)?;
        state.end()
    }
}


pub(crate) struct DailyMeasurements {
    pub morning: Option<Measurement>,
    pub midday: Option<Measurement>,
    pub evening: Option<Measurement>,
    pub other: Vec<Measurement>,
}
impl DailyMeasurements {
    pub fn new(
        morning: Option<Measurement>,
        midday: Option<Measurement>,
        evening: Option<Measurement>,
        other: Vec<Measurement>,
    ) -> DailyMeasurements {
        DailyMeasurements {
            morning,
            midday,
            evening,
            other,
        }
    }
}
impl Default for DailyMeasurements {
    fn default() -> Self {
        DailyMeasurements::new(None, None, None, Vec::new())
    }
}
impl Serialize for DailyMeasurements {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("DailyMeasurements", 4)?;
        state.serialize_field("morning", &self.morning)?;
        state.serialize_field("midday", &self.midday)?;
        state.serialize_field("evening", &self.evening)?;
        state.serialize_field("other", &self.other)?;
        state.end()
    }
}
