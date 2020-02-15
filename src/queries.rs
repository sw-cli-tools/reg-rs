pub const CREATE_TABLE: &str = "
 CREATE TABLE IF NOT EXISTS test_results (
 id              INTEGER PRIMARY KEY,
 name            TEXT NOT NULL,
 command         TEXT NOT NULL,
 time_created    TEXT NOT NULL,
 exit_code       INTEGER,
 stderr          TEXT NOT NULL,
 stdout          TEXT NOT NULL
 )
";

pub const INSERT_TEST_RESULTS: &str = "
 INSERT INTO test_results (
 name, command, time_created, exit_code, stderr, stdout)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

pub const SELECT_TEST_RESULTS: &str = "
 SELECT id, name, command, time_created, exit_code, stderr, stdout 
 FROM test_results
 ORDER BY time_created DESC
";
