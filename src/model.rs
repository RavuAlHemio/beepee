use std::convert::TryInto;

use chrono::{DateTime, Local};
use num_rational::Rational32;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use crate::datetime::{milliseconds_since_epoch, milliseconds_since_midnight};


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BloodPressureMeasurement {
    pub id: i64,
    pub timestamp: DateTime<Local>,
    pub systolic: i32,
    pub diastolic: i32,
    pub pulse: i32,
    pub spo2: Option<i32>,
}
impl BloodPressureMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        systolic: i32,
        diastolic: i32,
        pulse: i32,
        spo2: Option<i32>,
    ) -> Self {
        Self {
            id,
            timestamp,
            systolic,
            diastolic,
            pulse,
            spo2,
        }
    }

    pub fn values_max(&self, other: &Self) -> Self {
        let spo2 = match (self.spo2, other.spo2) {
            (None, None) => None,
            (Some(sv), None) => Some(sv),
            (None, Some(ov)) => Some(ov),
            (Some(sv), Some(ov)) => Some(sv.max(ov)),
        };
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.systolic.max(other.systolic),
            self.diastolic.max(other.diastolic),
            self.pulse.max(other.pulse),
            spo2,
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        let spo2 = match (self.spo2, other.spo2) {
            (None, None) => None,
            (Some(sv), None) => Some(sv),
            (None, Some(ov)) => Some(ov),
            (Some(sv), Some(ov)) => Some(sv.min(ov)),
        };
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.systolic.min(other.systolic),
            self.diastolic.min(other.diastolic),
            self.pulse.min(other.pulse),
            spo2,
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();

        let systolic_sum: i32 = measurements.iter().map(|m| m.systolic).sum();
        let diastolic_sum: i32 = measurements.iter().map(|m| m.diastolic).sum();
        let pulse_sum: i32 = measurements.iter().map(|m| m.pulse).sum();

        let spo2s: Vec<i32> = measurements.iter().filter_map(|m| m.spo2).collect();
        let spo2s_sum: i32 = spo2s.iter().sum();
        let spo2s_len_i32: i32 = spo2s.len().try_into().unwrap();
        let spo2 = if spo2s_len_i32 > 0 {
            Some(spo2s_sum / spo2s_len_i32)
        } else {
            None
        };

        Self::new(
            -1,
            measurements[0].timestamp,
            systolic_sum / len_i32,
            diastolic_sum / len_i32,
            pulse_sum / len_i32,
            spo2,
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let index = (measurements.len() - 1) * n_num / n_den;

        let mut systolics: Vec<i32> = measurements.iter().map(|m| m.systolic).collect();
        systolics.sort_unstable();

        let mut diastolics: Vec<i32> = measurements.iter().map(|m| m.diastolic).collect();
        diastolics.sort_unstable();

        let mut pulses: Vec<i32> = measurements.iter().map(|m| m.pulse).collect();
        pulses.sort_unstable();

        let mut spo2s: Vec<i32> = measurements.iter().filter_map(|m| m.spo2).collect();
        let spo2_index = (spo2s.len() - 1) * n_num / n_den;
        spo2s.sort_unstable();

        Self::new(
            -1,
            measurements[0].timestamp,
            systolics[index],
            diastolics[index],
            pulses[index],
            if spo2s.len() > 0 { Some(spo2s[spo2_index]) } else { None },
        )
    }
}
impl Serialize for BloodPressureMeasurement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct(
            stringify!(BloodPressureMeasurement),
            10,
        )?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())?;
        state.serialize_field("zoned_timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S %z").to_string())?;
        state.serialize_field("unix_timestamp_ms", &milliseconds_since_epoch(&self.timestamp))?;
        state.serialize_field("time_of_day_ms", &milliseconds_since_midnight(&self.timestamp.time()))?;
        state.serialize_field("time", &self.timestamp.format("%H:%M").to_string())?;
        state.serialize_field("systolic", &self.systolic)?;
        state.serialize_field("diastolic", &self.diastolic)?;
        state.serialize_field("pulse", &self.pulse)?;
        state.serialize_field("spo2", &self.spo2)?;
        state.end()
    }
}


#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct DailyBloodPressureMeasurements {
    pub date_string: String,
    pub morning: Option<BloodPressureMeasurement>,
    pub midday: Option<BloodPressureMeasurement>,
    pub evening: Option<BloodPressureMeasurement>,
    pub other: Vec<BloodPressureMeasurement>,
}
impl DailyBloodPressureMeasurements {
    pub fn new(
        date_string: String,
        morning: Option<BloodPressureMeasurement>,
        midday: Option<BloodPressureMeasurement>,
        evening: Option<BloodPressureMeasurement>,
        other: Vec<BloodPressureMeasurement>,
    ) -> Self {
        Self {
            date_string,
            morning,
            midday,
            evening,
            other,
        }
    }

    pub fn new_empty(date_string: String) -> Self {
        Self::new(date_string, None, None, None, Vec::new())
    }
}


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BodyMassMeasurement {
    pub id: i64,
    pub timestamp: DateTime<Local>,
    pub mass: Rational32,
}
impl BodyMassMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        mass: Rational32,
    ) -> Self {
        Self {
            id,
            timestamp,
            mass,
        }
    }

    pub fn values_max(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.mass.max(other.mass),
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.mass.min(other.mass),
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();
        let len_r32: Rational32 = len_i32.into();

        let mass_sum: Rational32 = measurements.iter().map(|m| m.mass).sum();

        Self::new(
            -1,
            measurements[0].timestamp,
            mass_sum / len_r32,
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let index = (measurements.len() - 1) * n_num / n_den;

        let mut masses: Vec<Rational32> = measurements.iter().map(|m| m.mass).collect();
        masses.sort_unstable();

        Self::new(
            -1,
            measurements[0].timestamp,
            masses[index],
        )
    }
}
impl Serialize for BodyMassMeasurement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct(
            stringify!(BodyMassMeasurement),
            6,
        )?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())?;
        state.serialize_field("zoned_timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S %z").to_string())?;
        state.serialize_field("unix_timestamp_ms", &milliseconds_since_epoch(&self.timestamp))?;
        state.serialize_field("time_of_day_ms", &milliseconds_since_midnight(&self.timestamp.time()))?;
        state.serialize_field("mass", &self.mass.to_string())?;
        state.end()
    }
}
