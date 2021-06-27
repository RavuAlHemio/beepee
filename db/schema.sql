CREATE SEQUENCE beepee.measurements_id_seq AS bigint START WITH 1;

CREATE TABLE beepee.measurements
( id bigint NOT NULL DEFAULT nextval('beepee.measurements_id_seq')
, "timestamp" timestamp with time zone NOT NULL
, systolic integer NOT NULL
, diastolic integer NOT NULL
, pulse integer NOT NULL
, spo2 integer NULL DEFAULT NULL
, CONSTRAINT measurements_pkey PRIMARY KEY (id)
, CONSTRAINT measurements_check CHECK (systolic >= 0 AND diastolic >= 0 AND pulse >= 0 AND (spo2 IS NULL OR spo2 BETWEEN 0 AND 100))
);
