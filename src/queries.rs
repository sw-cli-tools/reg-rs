pub const CREATE_ORIGINAL_TEST_RESULTS_TABLE: &str = "
 CREATE TABLE IF NOT EXISTS original_test_results (
 name              TEXT NOT NULL,
 command           TEXT NOT NULL,
 time_created      TEXT NOT NULL,
 exit_code         INTEGER,
 stderr            TEXT NOT NULL,
 stdout            TEXT NOT NULL,
 lock              CHAR(1) NOT NULL DEFAULT 'L',
 CONSTRAINT id     PRIMARY KEY (lock),
 CONSTRAINT locked CHECK (lock='L')
)
";
pub const INSERT_ORIGINAL_TEST_RESULTS: &str = "
 INSERT INTO original_test_results (
 name, command, time_created, exit_code, stderr, stdout)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

pub const SELECT_ORIGINAL_TEST_RESULTS: &str = "
 SELECT name, command, time_created, exit_code, stderr, stdout 
 FROM original_test_results
";
