DROP TABLE IF EXISTS feedback_response;

CREATE TABLE feedback_response (
    id      SERIAL PRIMARY KEY,
    name    VARCHAR(100) NOT NULL,
    response TEXT        NOT NULL,
    ts      TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_feedback_response_ts ON feedback_response (ts);
