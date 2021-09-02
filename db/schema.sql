CREATE SEQUENCE beepee.measurements_id_seq AS bigint START WITH 1;

CREATE TABLE beepee.measurements
( id bigint NOT NULL DEFAULT nextval('beepee.measurements_id_seq')
, "timestamp" timestamp with time zone NOT NULL
, systolic_mmhg integer NOT NULL
, diastolic_mmhg integer NOT NULL
, pulse_bpm integer NOT NULL
, spo2_percent integer NULL DEFAULT NULL
, CONSTRAINT measurements_pkey PRIMARY KEY (id)
, CONSTRAINT measurements_check CHECK (systolic_mmhg >= 0 AND diastolic_mmhg >= 0 AND pulse_bpm >= 0 AND (spo2_percent IS NULL OR spo2_percent BETWEEN 0 AND 100))
);

CREATE SEQUENCE beepee.mass_measurements_id_seq AS bigint START WITH 1;

CREATE TABLE beepee.mass_measurements
( id bigint NOT NULL DEFAULT nextval('beepee.mass_measurements_id_seq')
, "timestamp" timestamp with time zone NOT NULL
, mass_kg numeric(6, 2) NOT NULL
, CONSTRAINT mass_measurements_pkey PRIMARY KEY (id)
, CONSTRAINT mass_measurements_check CHECK (mass_kg >= 0)
);
