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
            "INSERT INTO beepee.measurements (timestamp, systolic, diastolic, pulse) VALUES ($1, $2, $3, $4) RETURNING id",
            &[&measurement.timestamp, &measurement.systolic, &measurement.diastolic, &measurement.pulse],
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
            "UPDATE beepee.measurements SET timestamp=$1, systolic=$2, diastolic=$3, pulse=$4 WHERE id=$5",
            &[&measurement.timestamp, &measurement.systolic, &measurement.diastolic, &measurement.pulse, &measurement.id],
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_recent_measurements(count: i64) -> Result<Vec<Measurement>, tokio_postgres::Error> {
    let client = connect()
        .await?;

    let rows = client
        .query(
            "SELECT id, timestamp, systolic, diastolic, pulse FROM beepee.measurements ORDER BY timestamp DESC LIMIT $1",
            &[&count],
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
        ));
    }

    Ok(ret)
}
