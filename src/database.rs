use chrono::{Duration, Local};
use log::error;
use tokio;
use tokio_postgres::{self, Client, NoTls};

use crate::config::CONFIG;
use crate::model::Measurement;


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

pub(crate) async fn add_measurement(measurement: &mut Measurement) -> Result<i64, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let row = client
        .query_one(
            "INSERT INTO beepee.measurements (timestamp, systolic, diastolic, pulse, spo2) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&measurement.timestamp, &measurement.systolic, &measurement.diastolic, &measurement.pulse, &measurement.spo2],
        )
        .await?;
    let measurement_id: i64 = row.get(0);

    Ok(measurement_id)
}

pub(crate) async fn remove_measurement(measurement_id: i64) -> Result<(), tokio_postgres::Error> {
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

pub(crate) async fn update_measurement(measurement: &Measurement) -> Result<(), tokio_postgres::Error> {
    let client = connect()
        .await?;

    client
        .execute(
            "UPDATE beepee.measurements SET timestamp=$1, systolic=$2, diastolic=$3, pulse=$4, spo2=$5 WHERE id=$6",
            &[&measurement.timestamp, &measurement.systolic, &measurement.diastolic, &measurement.pulse, &measurement.spo2, &measurement.id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_recent_measurements(ago: Duration) -> Result<Vec<Measurement>, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let start_time = Local::now() - ago;

    let rows = client
        .query(
            "SELECT id, timestamp, systolic, diastolic, pulse, spo2 FROM beepee.measurements WHERE timestamp >= $1 ORDER BY timestamp",
            &[&start_time],
        )
        .await?;
    let mut ret = Vec::new();
    for row in rows {
        ret.push(Measurement::new(
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
