use std::convert::TryInto;

use chrono::{DateTime, Local};
use num_rational::Rational32;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::numerism::{optional_max, optional_min, quasi_n_tile_index};


pub(crate) const SUGAR_MG_PER_DL_IN_MMOL_PER_L: i32 = 18;
pub(crate) const HBA1C_ADDITIVE_NUMER: i32 = 214;
pub(crate) const HBA1C_ADDITIVE_DENOM: i32 = 100;
pub(crate) const HBA1C_MULTIPLICATIVE_NUMER: i32 = 10_929;
pub(crate) const HBA1C_MULTIPLICATIVE_DENOM: i32 = 1_000;


#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct BloodPressureMeasurement {
    pub id: i64,
    #[serde(with = "crate::ser_de::serde_datetime_local")] pub timestamp: DateTime<Local>,
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
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.systolic_mmhg.max(other.systolic_mmhg),
            self.diastolic_mmhg.max(other.diastolic_mmhg),
            self.pulse_bpm.max(other.pulse_bpm),
            optional_max(self.spo2_percent, other.spo2_percent),
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.systolic_mmhg.min(other.systolic_mmhg),
            self.diastolic_mmhg.min(other.diastolic_mmhg),
            self.pulse_bpm.min(other.pulse_bpm),
            optional_min(self.spo2_percent, other.spo2_percent),
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

        let index = quasi_n_tile_index(measurements.len(), n_num, n_den);

        let mut systolics: Vec<i32> = measurements.iter().map(|m| m.systolic_mmhg).collect();
        systolics.sort_unstable();

        let mut diastolics: Vec<i32> = measurements.iter().map(|m| m.diastolic_mmhg).collect();
        diastolics.sort_unstable();

        let mut pulses: Vec<i32> = measurements.iter().map(|m| m.pulse_bpm).collect();
        pulses.sort_unstable();

        let mut spo2s: Vec<i32> = measurements.iter().filter_map(|m| m.spo2_percent).collect();
        let spo2_index = quasi_n_tile_index(spo2s.len(), n_num, n_den);
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


#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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


#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct BodyMassMeasurement {
    pub id: i64,
    #[serde(with = "crate::ser_de::serde_datetime_local")] pub timestamp: DateTime<Local>,
    #[serde(with = "crate::ser_de::serde_rat32")] pub mass_kg: Rational32,
    #[serde(with = "crate::ser_de::serde_rat32_opt")] pub waist_circum_cm: Option<Rational32>,
    #[serde(with = "crate::ser_de::serde_rat32_opt")] pub bmi: Option<Rational32>,
}
impl BodyMassMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        mass_kg: Rational32,
        waist_circum_cm: Option<Rational32>,
        bmi: Option<Rational32>,
    ) -> Self {
        Self {
            id,
            timestamp,
            mass_kg,
            waist_circum_cm,
            bmi,
        }
    }

    pub fn values_max(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.mass_kg.max(other.mass_kg),
            optional_max(self.waist_circum_cm, other.waist_circum_cm),
            optional_max(self.bmi, other.bmi),
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.mass_kg.min(other.mass_kg),
            optional_min(self.waist_circum_cm, other.waist_circum_cm),
            optional_min(self.bmi, other.bmi),
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();
        let len_r32: Rational32 = len_i32.into();

        let circum_len_i32: i32 = measurements.iter().filter(|m| m.waist_circum_cm.is_some()).count().try_into().unwrap();
        let circum_len_r32: Rational32 = circum_len_i32.into();

        let bmi_len_i32: i32 = measurements.iter().filter(|m| m.bmi.is_some()).count().try_into().unwrap();
        let bmi_len_r32: Rational32 = bmi_len_i32.into();

        let mass_sum: Rational32 = measurements.iter().map(|m| m.mass_kg).sum();
        let bmi_sum: Rational32 = measurements.iter().filter_map(|m| m.bmi).sum();
        let circum_sum: Rational32 = measurements.iter().filter_map(|m| m.waist_circum_cm).sum();

        Self::new(
            -1,
            measurements[0].timestamp,
            mass_sum / len_r32,
            if circum_len_r32 != Rational32::zero() { Some(circum_sum / circum_len_r32) } else { None },
            if bmi_len_r32 != Rational32::zero() { Some(bmi_sum / bmi_len_r32) } else { None },
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let mut masses: Vec<Rational32> = measurements.iter().map(|m| m.mass_kg).collect();
        masses.sort_unstable();

        let mut bmis: Vec<Rational32> = measurements.iter().filter_map(|m| m.bmi).collect();
        bmis.sort_unstable();

        let mut circums: Vec<Rational32> = measurements.iter().filter_map(|m| m.waist_circum_cm).collect();
        circums.sort_unstable();

        let index = quasi_n_tile_index(measurements.len(), n_num, n_den);
        let bmi_index = quasi_n_tile_index(bmis.len(), n_num, n_den);
        let circum_index = quasi_n_tile_index(circums.len(), n_num, n_den);

        Self::new(
            -1,
            measurements[0].timestamp,
            masses[index],
            circums.get(circum_index).map(|c| c.clone()),
            bmis.get(bmi_index).map(|b| b.clone()),
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct BodyTemperatureMeasurement {
    pub id: i64,
    #[serde(with = "crate::ser_de::serde_datetime_local")] pub timestamp: DateTime<Local>,
    pub location_id: i64,
    #[serde(with = "crate::ser_de::serde_rat32")] pub temperature_celsius: Rational32,
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

        let index = quasi_n_tile_index(measurements.len(), n_num, n_den);

        Self::new(
            -1,
            measurements[0].timestamp,
            measurements[0].location_id,
            temperatures[index],
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct BloodSugarMeasurement {
    pub id: i64,
    #[serde(with = "crate::ser_de::serde_datetime_local")] pub timestamp: DateTime<Local>,
    #[serde(with = "crate::ser_de::serde_rat32")] pub sugar_mmol_per_l: Rational32,
}
impl BloodSugarMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        sugar_mmol_per_l: Rational32,
    ) -> Self {
        Self {
            id,
            timestamp,
            sugar_mmol_per_l,
        }
    }

    pub fn new_mg_per_dl(
        id: i64,
        timestamp: DateTime<Local>,
        sugar_mg_per_dl: Rational32,
    ) -> Self {
        let sugar_mmol_per_l = &sugar_mg_per_dl / SUGAR_MG_PER_DL_IN_MMOL_PER_L;
        Self::new(
            id,
            timestamp,
            sugar_mmol_per_l,
        )
    }

    pub fn sugar_mg_per_dl(&self) -> Rational32 {
        self.sugar_mmol_per_l * SUGAR_MG_PER_DL_IN_MMOL_PER_L
    }

    pub fn values_max(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.sugar_mmol_per_l.max(other.sugar_mmol_per_l),
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.sugar_mmol_per_l.min(other.sugar_mmol_per_l),
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();
        let len_r32: Rational32 = len_i32.into();

        let sugar_mmol_per_l_sum: Rational32 = measurements.iter().map(|m| m.sugar_mmol_per_l).sum();

        Self::new(
            -1,
            measurements[0].timestamp,
            sugar_mmol_per_l_sum / len_r32,
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let mut sugars_mmol_per_l: Vec<Rational32> = measurements.iter().map(|m| m.sugar_mmol_per_l).collect();
        sugars_mmol_per_l.sort_unstable();

        let index = quasi_n_tile_index(measurements.len(), n_num, n_den);

        Self::new(
            -1,
            measurements[0].timestamp,
            sugars_mmol_per_l[index],
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct LongTermBloodSugarMeasurement {
    pub id: i64,
    #[serde(with = "crate::ser_de::serde_datetime_local")] pub timestamp: DateTime<Local>,
    #[serde(with = "crate::ser_de::serde_rat32")] pub hba1c_mmol_per_mol: Rational32,
}
impl LongTermBloodSugarMeasurement {
    pub fn new(
        id: i64,
        timestamp: DateTime<Local>,
        hba1c_mmol_per_mol: Rational32,
    ) -> Self {
        Self {
            id,
            timestamp,
            hba1c_mmol_per_mol,
        }
    }

    pub fn new_dcct_percent(
        id: i64,
        timestamp: DateTime<Local>,
        hba1c_dcct_percent: Rational32,
    ) -> Self {
        let additive_factor = Rational32::new(HBA1C_ADDITIVE_NUMER, HBA1C_ADDITIVE_DENOM);
        let multiplicative_factor = Rational32::new(HBA1C_MULTIPLICATIVE_NUMER, HBA1C_MULTIPLICATIVE_DENOM);
        let hba1c_mmol_per_mol = (hba1c_dcct_percent - additive_factor) * multiplicative_factor;

        Self::new(
            id,
            timestamp,
            hba1c_mmol_per_mol,
        )
    }

    pub fn hba1c_dcct_percent(&self) -> Rational32 {
        let additive_factor = Rational32::new(HBA1C_ADDITIVE_NUMER, HBA1C_ADDITIVE_DENOM);
        let multiplicative_factor = Rational32::new(HBA1C_MULTIPLICATIVE_NUMER, HBA1C_MULTIPLICATIVE_DENOM);

        (self.hba1c_mmol_per_mol / multiplicative_factor) + additive_factor
    }

    pub fn values_max(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.max(other.timestamp),
            self.hba1c_mmol_per_mol.max(other.hba1c_mmol_per_mol),
        )
    }

    pub fn values_min(&self, other: &Self) -> Self {
        Self::new(
            -1,
            self.timestamp.min(other.timestamp),
            self.hba1c_mmol_per_mol.min(other.hba1c_mmol_per_mol),
        )
    }

    pub fn average(measurements: &[Self]) -> Self {
        assert_ne!(measurements.len(), 0);
        let len_i32: i32 = measurements.len().try_into().unwrap();
        let len_r32: Rational32 = len_i32.into();

        let sugar_mmol_per_l_sum: Rational32 = measurements.iter().map(|m| m.hba1c_mmol_per_mol).sum();

        Self::new(
            -1,
            measurements[0].timestamp,
            sugar_mmol_per_l_sum / len_r32,
        )
    }

    pub fn quasi_n_tile(measurements: &[Self], n_num: usize, n_den: usize) -> Self {
        assert_ne!(measurements.len(), 0);

        let mut sugars_mmol_per_l: Vec<Rational32> = measurements.iter().map(|m| m.hba1c_mmol_per_mol).collect();
        sugars_mmol_per_l.sort_unstable();

        let index = quasi_n_tile_index(measurements.len(), n_num, n_den);

        Self::new(
            -1,
            measurements[0].timestamp,
            sugars_mmol_per_l[index],
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct MeasurementStatistics<T> {
    pub maximum: T,
    pub quasi_q3: T,
    pub average: T,
    pub quasi_q2: T,
    pub quasi_q1: T,
    pub minimum: T,
}
