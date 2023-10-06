CREATE TABLE IF NOT EXISTS user (
  id VARCHAR(18) NOT NULL PRIMARY KEY,
  org_id VARCHAR(18) NOT NULL,
  username VARCHAR NOT NULL,
  first_name VARCHAR NOT NULL,
  last_name VARCHAR NOT NULL,
  alias VARCHAR NOT NULL,
  base_url TEXT NOT NULL,
  access_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL
);