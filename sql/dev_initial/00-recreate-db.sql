-- DEV ONLY - Brute Force DROP DB (for local dev and unit test)
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE
 usename = 'app_user_db_db' OR datname = 'sports_guide';
DROP DATABASE IF EXISTS sports_guide;
DROP USER IF EXISTS app_user_db;

-- DEV ONLY - Dev only password (for local dev and unit test).
CREATE USER app_user_db PASSWORD 'dev_only_pwd';
CREATE DATABASE sports_guide owner app_user_db ENCODING = 'UTF-8';