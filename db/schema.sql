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
, waist_circum_cm numeric(6, 2) NULL DEFAULT NULL
, CONSTRAINT mass_measurements_pkey PRIMARY KEY (id)
, CONSTRAINT mass_measurements_check CHECK (mass_kg >= 0 AND (waist_circum_cm IS NULL OR waist_circum_cm >= 0))
);

CREATE SEQUENCE beepee.body_temperature_locations_id_seq AS bigint START WITH 1;

CREATE TABLE beepee.body_temperature_locations
( id bigint NOT NULL DEFAULT nextval('beepee.body_temperature_locations_id_seq')
, "name" varchar(256) NOT NULL
, CONSTRAINT body_temperature_locations_pkey PRIMARY KEY (id)
);

CREATE SEQUENCE beepee.body_temperature_measurements_id_seq AS bigint START WITH 1;

CREATE TABLE beepee.body_temperature_measurements
( id bigint NOT NULL DEFAULT nextval('beepee.body_temperature_measurements_id_seq')
, "timestamp" timestamp with time zone NOT NULL
, location_id bigint NOT NULL
, temperature_celsius numeric(6, 2) NOT NULL
, CONSTRAINT body_temperature_measurements_pkey PRIMARY KEY (id)
, CONSTRAINT body_temperature_measurements_check CHECK (temperature_celsius >= -273.15)
, CONSTRAINT body_temperature_measurements_location_id_fkey FOREIGN KEY (location_id) REFERENCES beepee.body_temperature_locations (id)
);

CREATE SEQUENCE beepee.blood_sugar_measurements_id_seq AS bigint START WITH 1;

CREATE TABLE beepee.blood_sugar_measurements
( id bigint NOT NULL DEFAULT nextval('beepee.blood_sugar_measurements_id_seq')
, "timestamp" timestamp with time zone NOT NULL
, sugar_mmol_per_l numeric(6, 2) NOT NULL
, CONSTRAINT blood_sugar_measurements_pkey PRIMARY KEY (id)
);

CREATE SEQUENCE beepee.long_term_blood_sugar_measurements_id_seq AS bigint START WITH 1;

CREATE TABLE beepee.long_term_blood_sugar_measurements
( id bigint NOT NULL DEFAULT nextval('beepee.long_term_blood_sugar_measurements_id_seq')
, "timestamp" timestamp with time zone NOT NULL
, hba1c_mmol_per_mol numeric(6, 2) NOT NULL
, CONSTRAINT long_term_blood_sugar_measurements_pkey PRIMARY KEY (id)
);
