-- PostgreSQL Enterprise Extension Definition for Hadamard-DB Link
-- Copyright 2026 Yağız Yağlı. Apache-2.0 License.

CREATE OR REPLACE FUNCTION pg_hadamard_sync_table(
    collection_name TEXT,
    json_table_payload TEXT
)
RETURNS INT
AS 'hadamard_postgres_extension.so', 'pg_hadamard_sync_table'
LANGUAGE C STRICT;

CREATE OR REPLACE FUNCTION pg_hadamard_quantum_search(
    sql_statement TEXT
)
RETURNS TEXT
AS 'hadamard_postgres_extension.so', 'pg_hadamard_quantum_search'
LANGUAGE C STRICT;

-- =========================================================================
-- PRACTICAL USAGE TESTCASE INSIDE POSTGRESQL ENVIRONMENT
-- =========================================================================
-- 1. Create a massive standard transactional audit table
-- CREATE TABLE audit_logs (id SERIAL PRIMARY KEY, ip_address TEXT, action_status TEXT);
--
-- 2. Synchronize Postgres snapshots directly into Quantum QRAM memory pools:
-- SELECT pg_hadamard_sync_table('postgres_audit_pool', json_agg(t)) FROM (SELECT ip_address, action_status FROM audit_logs) t;
--
-- 3. Execute accelerated Quantum search bypass loops from within traditional SQL script:
-- SELECT pg_hadamard_quantum_search('SELECT * FROM postgres_audit_pool WHERE action_status = ''MALICIOUS''');
