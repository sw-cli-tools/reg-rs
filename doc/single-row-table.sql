-- https://stackoverflow.com/questions/3967372/sql-server-how-to-constrain-a-table-to-contain-a-single-row
CREATE TABLE ORIGINAL (
    Lock char(1) not null DEFAULT 'X',
    test_result_id INTEGER NOT NULL,
    constraint PK_T1 PRIMARY KEY (Lock),
    constraint CK_T1_Locked CHECK (Lock='X')
);

