DROP TABLE IF EXISTS feedback_response;

CREATE TABLE feedback_response (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    response TEXT,
    ts TIMESTAMP WITH TIME ZONE
);

CREATE TABLE user (
    id SERIAL PRIMARY KEY
);