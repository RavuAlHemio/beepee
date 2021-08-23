use chrono::{Duration, Local};
use log::error;
use num_rational::Rational32;
use tokio;
use tokio_postgres::{self, Client, NoTls};

use crate::config::CONFIG;
use crate::model::{BloodPressureMeasurement, BodyMassMeasurement};
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

pub(crate) async fn add_blood_pressure_measurement(measurement: &mut BloodPressureMeasurement) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = client
        .query_one(
            "INSERT INTO beepee.measurements (\"timestamp\", systolic, diastolic, pulse, spo2) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&measurement.timestamp, &measurement.systolic, &measurement.diastolic, &measurement.pulse, &measurement.spo2],
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
            "UPDATE beepee.measurements SET \"timestamp\"=$1, systolic=$2, diastolic=$3, pulse=$4, spo2=$5 WHERE id=$6",
            &[&measurement.timestamp, &measurement.systolic, &measurement.diastolic, &measurement.pulse, &measurement.spo2, &measurement.id],
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
            "SELECT id, \"timestamp\", systolic, diastolic, pulse, spo2 FROM beepee.measurements WHERE \"timestamp\" >= $1 ORDER BY \"timestamp\"",
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

pub(crate) async fn add_mass_measurement(measurement: &mut BodyMassMeasurement) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = client
        .query_one(
            "INSERT INTO beepee.mass_measurements (\"timestamp\", mass) VALUES ($1, ($2::int::numeric(6, 2) / $3::int::numeric(6, 2))) RETURNING id",
            &[&measurement.timestamp, &measurement.mass.numer(), &measurement.mass.denom()],
        )
        .await?;
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

    client
        .execute(
            "UPDATE beepee.mass_measurements SET \"timestamp\"=$1, mass=($2::int::numeric(6, 2) / $3::int::numeric(6, 2)) WHERE id=$4",
            &[&measurement.timestamp, &measurement.mass.numer(), &measurement.mass.denom(), &measurement.id],
        )
        .await?;

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
            "SELECT id, \"timestamp\", mass::character varying(128) FROM beepee.mass_measurements WHERE \"timestamp\" >= $1 ORDER BY \"timestamp\"",
            &[&start_time],
        )
        .await?;
    let mut ret = Vec::new();
    for row in rows {
        let mass_string: String = row.get(2);
        let mass: Rational32 = r32_from_decimal(&mass_string)
            .expect("parsing mass failed");
        let bmi: Option<Rational32> = square_height_m2.map(|sqh|
            mass / sqh
        );
        ret.push(BodyMassMeasurement::new(
            row.get(0),
            row.get(1),
            mass,
            bmi,
        ));
    }

    Ok(ret)
}
