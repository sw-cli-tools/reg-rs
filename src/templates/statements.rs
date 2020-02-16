pub const CREATE_TEST_RESULTS_TABLE_TEMPLATE: &str = "
 CREATE TABLE IF NOT EXISTS { table_name } (
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
pub const INSERT_TEST_RESULTS_TEMPLATE: &str = "
 INSERT INTO { table_name } (
 name, command, time_created, exit_code, stderr, stdout)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

pub const SELECT_TEST_RESULTS_TEMPLATE: &str = "
 SELECT name, command, time_created, exit_code, stderr, stdout 
 FROM { table_name }
";

pub const DROP_TABLE_TEMPLATE: &str = "
 DROP TABLE IF EXISTS { table_name }
";

pub const CREATE_DIFFERENCES_TABLE_TEMPLATE: &str = "
 CREATE TABLE IF NOT EXISTS { table_name } (
 id                INTEGER PRIMARY KEY,
 type              TEXT NOT NULL,
 chunk             TEXT
)
";

pub const INSERT_DIFFERENCE_TEMPLATE: &str = "
 INSERT INTO { table_name } (type, chunk)
 VALUES (?1, ?2)
";

pub const SELECT_DIFFERENCES_TEMPLATE: &str = "
 SELECT type, chunk
 FROM { table_name }
";
