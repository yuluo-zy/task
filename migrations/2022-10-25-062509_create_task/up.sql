-- Your SQL goes here
CREATE TABLE fang_tasks (
                            id VARCHAR(50) PRIMARY KEY not null ,
                            metadata JSON NOT NULL,
                            error_message TEXT,
                            state  CHAR(64) NOT NULL default 'new',
                            task_type VARCHAR(256) DEFAULT 'common' NOT NULL,
                            uniq_hash CHAR(64),
                            retries INTEGER DEFAULT 0 NOT NULL,
                            scheduled_at Timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL ,
                            created_at Timestamp  DEFAULT CURRENT_TIMESTAMP NOT NULL ,
                            updated_at Timestamp  DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX fang_tasks_type_index ON fang_tasks(task_type);
CREATE INDEX fang_tasks_scheduled_at_index ON fang_tasks(scheduled_at);
CREATE INDEX fang_tasks_uniq_hash ON fang_tasks(uniq_hash);
