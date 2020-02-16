pub const CREATE_TABLE_TEST_RESULTS: &str = "
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

// this table has one row which points to the original test result
pub const CREATE_TABLE_ORIGINAL: &str = "
 CREATE TABLE original (
 Lock char(1) not null DEFAULT 'original',
 test_result_id INTEGER NOT NULL,
 constraint PK_ORIGINAL PRIMARY KEY (Lock),
 constraint CK_ORIGINAL_Locked CHECK (Lock='original')
)
";

// this table has at most one row which points to a test regression, if any
pub const CREATE_TABLE_REGRESSION: &str = "
 CREATE TABLE regression (
 Lock char(1) not null DEFAULT 'regression',
 test_result_id INTEGER NOT NULL,
 constraint PK_REGRESSION PRIMARY KEY (Lock),
 constraint CK_REGRESSION_Locked CHECK (Lock='regression')
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
