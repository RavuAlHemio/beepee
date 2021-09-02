use std::convert::TryInto;

use chrono::{DateTime, Local};
use num_rational::Rational32;
use num_traits::Zero;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use crate::datetime::{milliseconds_since_epoch, milliseconds_since_midnight};


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BloodPressureMeasurement {
    pub id: i64,
    pub timestamp: DateTime<Local>,
    pub systolic_mmhg: i32,
    pub diastolic_mmhg: i32,
    pub pulse_bpm: i32,
    pub spo2_percent: Option<i32>,
}
impl BloodPressureMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        systolic_mmhg: i32,
        diastolic_mmhg: i32,
        pulse_bpm: i32,
        spo2_percent: Option<i32>,
    ) -> Self {
        Self {
            id,
            timestamp,
            systolic_mmhg,
            diastolic_mmhg,
            pulse_bpm,
            spo2_percent,
        }
    }

    pub fn values_max(&self, other: &Self) -> Self {
        let spo2_percent = match (self.spo2_percent, other.spo2_percent) {
            (None, None) => None,
            (Some(sv), None) => Some(sv),
            (None, Some(ov)) => Some(ov),
            (Some(sv), Some(ov)) => Some(sv.max(ov)),
        };
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.systolic_mmhg.max(other.systolic_mmhg),
            self.diastolic_mmhg.max(other.diastolic_mmhg),
            self.pulse_bpm.max(other.pulse_bpm),
            spo2_percent,
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        let spo2_percent = match (self.spo2_percent, other.spo2_percent) {
            (None, None) => None,
            (Some(sv), None) => Some(sv),
            (None, Some(ov)) => Some(ov),
            (Some(sv), Some(ov)) => Some(sv.min(ov)),
        };
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.systolic_mmhg.min(other.systolic_mmhg),
            self.diastolic_mmhg.min(other.diastolic_mmhg),
            self.pulse_bpm.min(other.pulse_bpm),
            spo2_percent,
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();

        let systolic_sum: i32 = measurements.iter().map(|m| m.systolic_mmhg).sum();
        let diastolic_sum: i32 = measurements.iter().map(|m| m.diastolic_mmhg).sum();
        let pulse_sum: i32 = measurements.iter().map(|m| m.pulse_bpm).sum();

        let spo2s: Vec<i32> = measurements.iter().filter_map(|m| m.spo2_percent).collect();
        let spo2s_sum: i32 = spo2s.iter().sum();
        let spo2s_len_i32: i32 = spo2s.len().try_into().unwrap();
        let spo2_percent = if spo2s_len_i32 > 0 {
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
            spo2_percent,
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let index = (measurements.len() - 1) * n_num / n_den;

        let mut systolics: Vec<i32> = measurements.iter().map(|m| m.systolic_mmhg).collect();
        systolics.sort_unstable();

        let mut diastolics: Vec<i32> = measurements.iter().map(|m| m.diastolic_mmhg).collect();
        diastolics.sort_unstable();

        let mut pulses: Vec<i32> = measurements.iter().map(|m| m.pulse_bpm).collect();
        pulses.sort_unstable();

        let mut spo2s: Vec<i32> = measurements.iter().filter_map(|m| m.spo2_percent).collect();
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
        state.serialize_field("systolic_mmhg", &self.systolic_mmhg)?;
        state.serialize_field("diastolic_mmhg", &self.diastolic_mmhg)?;
        state.serialize_field("pulse_bpm", &self.pulse_bpm)?;
        state.serialize_field("spo2_percent", &self.spo2_percent)?;
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
    pub mass_kg: Rational32,
    pub bmi: Option<Rational32>,
}
impl BodyMassMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        mass_kg: Rational32,
        bmi: Option<Rational32>,
    ) -> Self {
        Self {
            id,
            timestamp,
            mass_kg,
            bmi,
        }
    }

    pub fn values_max(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.mass_kg.max(other.mass_kg),
            self.bmi.max(other.bmi),
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.mass_kg.min(other.mass_kg),
            self.bmi.max(other.bmi),
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();
        let len_r32: Rational32 = len_i32.into();

        let bmi_len_i32: i32 = measurements.iter().filter(|m| m.bmi.is_some()).count().try_into().unwrap();
        let bmi_len_r32: Rational32 = bmi_len_i32.into();

        let mass_sum: Rational32 = measurements.iter().map(|m| m.mass_kg).sum();
        let bmi_sum: Rational32 = measurements.iter().filter_map(|m| m.bmi).sum();

        Self::new(
            -1,
            measurements[0].timestamp,
            mass_sum / len_r32,
            if bmi_len_r32 != Rational32::zero() { Some(bmi_sum / bmi_len_r32) } else { None },
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let mut masses: Vec<Rational32> = measurements.iter().map(|m| m.mass_kg).collect();
        masses.sort_unstable();

        let mut bmis: Vec<Rational32> = measurements.iter().filter_map(|m| m.bmi).collect();
        bmis.sort_unstable();

        let index = (measurements.len() - 1) * n_num / n_den;
        let bmi_index = if bmis.len() == 0 {
            0
        } else {
            (bmis.len() - 1) * n_num / n_den
        };

        Self::new(
            -1,
            measurements[0].timestamp,
            masses[index],
            bmis.get(bmi_index).map(|b| b.clone()),
        )
    }
}
impl Serialize for BodyMassMeasurement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct(
            stringify!(BodyMassMeasurement),
            7,
        )?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())?;
        state.serialize_field("zoned_timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S %z").to_string())?;
        state.serialize_field("unix_timestamp_ms", &milliseconds_since_epoch(&self.timestamp))?;
        state.serialize_field("time_of_day_ms", &milliseconds_since_midnight(&self.timestamp.time()))?;
        state.serialize_field("mass_kg", &self.mass_kg.to_string())?;
        state.serialize_field("bmi", &self.bmi.map(|b| b.to_string()))?;
        state.end()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct BodyTemperatureLocation {
    pub id: i64,
    pub name: String,
}
impl BodyTemperatureLocation {
    pub fn new(
        id: i64,
        name: String,
    ) -> Self {
        Self {
            id,
            name,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BodyTemperatureMeasurement {
    pub id: i64,
    pub timestamp: DateTime<Local>,
    pub location_id: i64,
    pub temperature_celsius: Rational32,
}
impl BodyTemperatureMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        location_id: i64,
        temperature_celsius: Rational32,
    ) -> Self {
        Self {
            id,
            timestamp,
            location_id,
            temperature_celsius,
        }
    }

    pub fn values_max(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.location_id.max(other.location_id),
            self.temperature_celsius.max(other.temperature_celsius),
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.location_id.min(other.location_id),
            self.temperature_celsius.max(other.temperature_celsius),
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();
        let len_r32: Rational32 = len_i32.into();

        let temperature_celsius_sum: Rational32 = measurements.iter().map(|m| m.temperature_celsius).sum();

        Self::new(
            -1,
            measurements[0].timestamp,
            measurements[0].location_id,
            temperature_celsius_sum / len_r32,
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let mut temperatures: Vec<Rational32> = measurements.iter().map(|m| m.temperature_celsius).collect();
        temperatures.sort_unstable();

        let index = (measurements.len() - 1) * n_num / n_den;

        Self::new(
            -1,
            measurements[0].timestamp,
            measurements[0].location_id,
            temperatures[index],
        )
    }
}
impl Serialize for BodyTemperatureMeasurement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct(
            stringify!(BodyTemperatureMeasurement),
            7,
        )?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())?;
        state.serialize_field("zoned_timestamp", &self.timestamp.format("%Y-%m-%d %H:%M:%S %z").to_string())?;
        state.serialize_field("unix_timestamp_ms", &milliseconds_since_epoch(&self.timestamp))?;
        state.serialize_field("time_of_day_ms", &milliseconds_since_midnight(&self.timestamp.time()))?;
        state.serialize_field("location_id", &self.location_id.to_string())?;
        state.serialize_field("temperature_celsius", &self.temperature_celsius.to_string())?;
        state.end()
    }
}
