use chrono::{Duration, Local};
use log::error;
use num_rational::Rational32;
use tokio;
use tokio_postgres::{self, Client, NoTls};

use crate::config::CONFIG;
use crate::model::{
    BloodPressureMeasurement, BloodSugarMeasurement, BodyMassMeasurement, BodyTemperatureLocation,
    BodyTemperatureMeasurement,
};
use crate::numerism::r32_from_decimal;


async fn get_conn_string() -> String {
    CONFIG
        .get().expect("config not set")
        .read().await
        .db_conn_string
        .clone()
}

async fn connect() -> Result<Client, tokio_postgres::Error> {
    let conn_string = get_conn_string()
        .await;

    let (client, connection) = tokio_postgres::connect(&conn_string, NoTls)
        .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    Ok(client)
}

pub(crate) async fn add_blood_pressure_measurement(measurement: &BloodPressureMeasurement) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = client
        .query_one(
            "INSERT INTO beepee.measurements (\"timestamp\", systolic_mmhg, diastolic_mmhg, pulse_bpm, spo2_percent) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&measurement.timestamp, &measurement.systolic_mmhg, &measurement.diastolic_mmhg, &measurement.pulse_bpm, &measurement.spo2_percent],
        )
        .await?;
    let measurement_id: i64 = row.get(0);

    Ok(measurement_id)
}

pub(crate) async fn remove_blood_pressure_measurement(measurement_id: i64) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "DELETE FROM beepee.measurements WHERE id = $1",
            &[&measurement_id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn update_blood_pressure_measurement(measurement: &BloodPressureMeasurement) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "UPDATE beepee.measurements SET \"timestamp\"=$1, systolic_mmhg=$2, diastolic_mmhg=$3, pulse_bpm=$4, spo2_percent=$5 WHERE id=$6",
            &[&measurement.timestamp, &measurement.systolic_mmhg, &measurement.diastolic_mmhg, &measurement.pulse_bpm, &measurement.spo2_percent, &measurement.id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_recent_blood_pressure_measurements(ago: Duration) -> Result<Vec<BloodPressureMeasurement>, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let start_time = Local::now() - ago;

    let rows = client
        .query(
            "SELECT id, \"timestamp\", systolic_mmhg, diastolic_mmhg, pulse_bpm, spo2_percent FROM beepee.measurements WHERE \"timestamp\" >= $1 ORDER BY \"timestamp\"",
            &[&start_time],
        )
        .await?;
    let mut ret = Vec::new();
    for row in rows {
        ret.push(BloodPressureMeasurement::new(
            row.get(0),
            row.get(1),
            row.get(2),
            row.get(3),
            row.get(4),
            row.get(5),
        ));
    }

    Ok(ret)
}

pub(crate) async fn add_mass_measurement(measurement: &BodyMassMeasurement) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = if let Some(circum) = &measurement.waist_circum_cm {
        client
            .query_one(
                "INSERT INTO beepee.mass_measurements (\"timestamp\", mass_kg, waist_circum_cm) VALUES ($1, (CAST(CAST($2 AS int) AS numeric(6, 2)) / CAST(CAST($3 AS int) AS numeric(6, 2))), (CAST(CAST($4 AS int) AS numeric(6, 2)) / CAST(CAST($5 AS int) AS numeric(6, 2)))) RETURNING id",
                &[&measurement.timestamp, &measurement.mass_kg.numer(), &measurement.mass_kg.denom(), &circum.numer(), &circum.denom()],
            )
            .await?
    } else {
        client
            .query_one(
                "INSERT INTO beepee.mass_measurements (\"timestamp\", mass_kg, waist_circum_cm) VALUES ($1, (CAST(CAST($2 AS int) AS numeric(6, 2)) / CAST(CAST($3 AS int) AS numeric(6, 2))), NULL) RETURNING id",
                &[&measurement.timestamp, &measurement.mass_kg.numer(), &measurement.mass_kg.denom()],
            )
            .await?
    };
    let measurement_id: i64 = row.get(0);

    Ok(measurement_id)
}

pub(crate) async fn remove_mass_measurement(measurement_id: i64) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "DELETE FROM beepee.mass_measurements WHERE id = $1",
            &[&measurement_id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn update_mass_measurement(measurement: &BodyMassMeasurement) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    if let Some(circum) = &measurement.waist_circum_cm {
        client
            .execute(
                "UPDATE beepee.mass_measurements SET \"timestamp\"=$1, mass_kg=(CAST(CAST($2 AS int) AS numeric(6, 2)) / CAST(CAST($3 AS int) AS numeric(6, 2))), waist_circum_cm=(CAST(CAST($4 AS int) AS numeric(6, 2)) / CAST(CAST($5 AS int) AS numeric(6, 2))) WHERE id=$6",
                &[&measurement.timestamp, &measurement.mass_kg.numer(), &measurement.mass_kg.denom(), &circum.numer(), &circum.denom(), &measurement.id],
            )
            .await?
    } else {
        client
            .execute(
                "UPDATE beepee.mass_measurements SET \"timestamp\"=$1, mass_kg=(CAST(CAST($2 AS int) AS numeric(6, 2)) / CAST(CAST($3 AS int) AS numeric(6, 2))), waist_circum_cm=NULL WHERE id=$4",
                &[&measurement.timestamp, &measurement.mass_kg.numer(), &measurement.mass_kg.denom(), &measurement.id],
            )
            .await?
    };

    Ok(())
}

pub(crate) async fn get_recent_mass_measurements(ago: Duration) -> Result<Vec<BodyMassMeasurement>, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let height_cm: Option<i32> = {
        let config_guard = CONFIG
            .get().expect("initial config not set")
            .read().await;
        config_guard.height_cm
    };
    let height_m = height_cm
        .map(|h| Rational32::new(h, 100));
    let square_height_m2 = height_m
        .map(|h| h * h);

    let start_time = Local::now() - ago;

    let rows = client
        .query(
            "SELECT id, \"timestamp\", CAST(mass_kg AS character varying(128)) mass_kg, CAST(waist_circum_cm AS character varying(128)) waist_circum_cm FROM beepee.mass_measurements WHERE \"timestamp\" >= $1 ORDER BY \"timestamp\"",
            &[&start_time],
        )
        .await?;
    let mut ret = Vec::new();
    for row in rows {
        let mass_string: String = row.get(2);
        let mass_kg: Rational32 = r32_from_decimal(&mass_string)
            .expect("parsing mass failed");
        let circum_string: Option<String> = row.get(3);
        let circum_cm: Option<Rational32> = circum_string.map(|s|
            r32_from_decimal(&s)
                .expect("parsing circumference failed")
        );
        let bmi: Option<Rational32> = square_height_m2.map(|sqh|
            mass_kg / sqh
        );
        ret.push(BodyMassMeasurement::new(
            row.get(0),
            row.get(1),
            mass_kg,
            circum_cm,
            bmi,
        ));
    }

    Ok(ret)
}

pub(crate) async fn add_temperature_location(loc: &BodyTemperatureLocation) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = client
        .query_one(
            "INSERT INTO beepee.body_temperature_locations (\"name\") VALUES ($1) RETURNING id",
            &[&loc.name],
        )
        .await?;
    let loc_id: i64 = row.get(0);

    Ok(loc_id)
}

pub(crate) async fn remove_temperature_location(loc_id: i64) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "DELETE FROM beepee.body_temperature_locations WHERE id = $1",
            &[&loc_id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn update_temperature_location(loc: &BodyTemperatureLocation) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "UPDATE beepee.body_temperature_locations SET \"name\"=$1 WHERE id=$2",
            &[&loc.name, &loc.id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_temperature_locations() -> Result<Vec<BodyTemperatureLocation>, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let rows = client
        .query(
            "SELECT id, \"name\" FROM beepee.body_temperature_locations ORDER BY \"name\"",
            &[],
        )
        .await?;
    let mut ret = Vec::new();
    for row in rows {
        ret.push(BodyTemperatureLocation::new(
            row.get(0),
            row.get(1),
        ));
    }

    Ok(ret)
}

pub(crate) async fn add_temperature_measurement(measurement: &BodyTemperatureMeasurement) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = client
        .query_one(
            "INSERT INTO beepee.body_temperature_measurements (\"timestamp\", location_id, temperature_celsius) VALUES ($1, $2, (CAST(CAST($3 AS int) AS numeric(6, 2)) / CAST(CAST($4 AS int) AS numeric(6, 2)))) RETURNING id",
            &[&measurement.timestamp, &measurement.location_id, &measurement.temperature_celsius.numer(), &measurement.temperature_celsius.denom()],
        )
        .await?;
    let measurement_id: i64 = row.get(0);

    Ok(measurement_id)
}

pub(crate) async fn remove_temperature_measurement(measurement_id: i64) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "DELETE FROM beepee.body_temperature_measurements WHERE id = $1",
            &[&measurement_id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn update_temperature_measurement(measurement: &BodyTemperatureMeasurement) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "UPDATE beepee.body_temperature_measurements SET \"timestamp\"=$1, location_id=$2, temperature_celsius=(CAST(CAST($3 AS int) AS numeric(6, 2)) / CAST(CAST($4 AS int) AS numeric(6, 2))) WHERE id=$5",
            &[&measurement.timestamp, &measurement.location_id, &measurement.temperature_celsius.numer(), &measurement.temperature_celsius.denom(), &measurement.id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_recent_temperature_measurements(ago: Duration) -> Result<Vec<BodyTemperatureMeasurement>, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let start_time = Local::now() - ago;

    let rows = client
        .query(
            "SELECT id, \"timestamp\", location_id, CAST(temperature_celsius AS character varying(128)) temperature_celsius FROM beepee.body_temperature_measurements WHERE \"timestamp\" >= $1 ORDER BY \"timestamp\"",
            &[&start_time],
        )
        .await?;
    let mut ret = Vec::new();
    for row in rows {
        let temperature_string: String = row.get(3);
        let temperature_celsius: Rational32 = r32_from_decimal(&temperature_string)
            .expect("parsing temperature failed");
        ret.push(BodyTemperatureMeasurement::new(
            row.get(0),
            row.get(1),
            row.get(2),
            temperature_celsius,
        ));
    }

    Ok(ret)
}

pub(crate) async fn add_blood_sugar_measurement(measurement: &BloodSugarMeasurement) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = client
        .query_one(
            "INSERT INTO beepee.blood_sugar_measurements (\"timestamp\", sugar_mmol_per_l) VALUES ($1, (CAST(CAST($2 AS int) AS numeric(6, 2)) / CAST(CAST($3 AS int) AS numeric(6, 2)))) RETURNING id",
            &[&measurement.timestamp, &measurement.sugar_mmol_per_l.numer(), &measurement.sugar_mmol_per_l.denom()],
        )
        .await?;
    let measurement_id: i64 = row.get(0);

    Ok(measurement_id)
}

pub(crate) async fn remove_blood_sugar_measurement(measurement_id: i64) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "DELETE FROM beepee.blood_sugar_measurements WHERE id = $1",
            &[&measurement_id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn update_blood_sugar_measurement(measurement: &BloodSugarMeasurement) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "UPDATE beepee.blood_sugar_measurements SET \"timestamp\"=$1, sugar_mmol_per_l=(CASE(CASE($2 AS int) AS numeric(6, 2)) / CASE(CASE($3 AS int) AS numeric(6, 2))) WHERE id=$4",
            &[&measurement.timestamp, &measurement.sugar_mmol_per_l.numer(), &measurement.sugar_mmol_per_l.denom(), &measurement.id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_recent_blood_sugar_measurements(ago: Duration) -> Result<Vec<BloodSugarMeasurement>, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let start_time = Local::now() - ago;

    let rows = client
        .query(
            "SELECT id, \"timestamp\", CASE(sugar_mmol_per_l AS character varying(128)) sugar_mmol_per_l FROM beepee.blood_sugar_measurements WHERE \"timestamp\" >= $1 ORDER BY \"timestamp\"",
            &[&start_time],
        )
        .await?;
    let mut ret = Vec::new();
    for row in rows {
        let temperature_string: String = row.get(2);
        let temperature_celsius: Rational32 = r32_from_decimal(&temperature_string)
            .expect("parsing temperature failed");
        ret.push(BloodSugarMeasurement::new(
            row.get(0),
            row.get(1),
            temperature_celsius,
        ));
    }

    Ok(ret)
}
