CREATE TABLE requests (
	ip         TEXT NOT NULL,
	user       TEXT NOT NULL,
	ts         TEXT NOT NULL,
	method     TEXT NOT NULL,
	path       TEXT NOT NULL,
	protocol   TEXT NOT NULL,
	resp_code  INTEGER NOT NULL,
	resp_size  INTEGER NOT NULL,
	referer    TEXT NOT NULL,
	user_agent TEXT NOT NULL
);

CREATE INDEX requests_ip ON requests (ip);
CREATE INDEX requests_ts ON requests (ts);
CREATE INDEX requests_path ON requests (path);
CREATE INDEX requests_resp_code ON requests (resp_code);
CREATE UNIQUE INDEX requests_ip_ts_path ON requests (ip, ts, path);

CREATE TABLE downloads (
	ts       TEXT NOT NULL,
	version  TEXT NOT NULL,
	country  TEXT,
	city     TEXT,
	domain   TEXT,
	user_agent_hash TEXT NOT NULL,
	request_id INTEGER
);

CREATE INDEX downloads_ts ON downloads (ts);
CREATE INDEX downloads_version ON downloads (version);
CREATE INDEX downloads_country ON downloads (country);
CREATE INDEX downloads_city ON downloads (city);
CREATE INDEX downloads_domain ON downloads (domain);
CREATE INDEX downloads_user_agent_hash ON downloads (user_agent_hash);
CREATE INDEX downloads_request_id ON downloads (request_id);
