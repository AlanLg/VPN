DROP SCHEMA IF EXISTS testing CASCADE;
CREATE SCHEMA testing;

CREATE TABLE testing.users (
	id  BIGSERIAL PRIMARY KEY,
	email       VARCHAR(200) NOT NULL,
	username    VARCHAR(50) UNIQUE NOT NULL,
    role VARCHAR(50) NOT NULL,
    public_key VARCHAR(50) NULL,
    private_key VARCHAR(50) NULL,
	UNIQUE (username)
);
