CREATE FUNCTION update_key_times() RETURNS INTEGER as $$
BEGIN
    DECLARE ret INTEGER := 0
END;

SELECT "Time_Hour" AS hour,
       "Time_Minute" AS min,
       "Time_TimeOfDay" AS tod,
       "Time_TimeType" AS kind
INTO kt
FROM "KeyTimes" AS kt
WHERE kt."Date" = CURRENT_DATE;

PREPARE update_time (kind INTEGER, hour INTEGER, minute INTEGER) as
    UPDATE "Flips"
    SET "Time_Hour" = hour,
    SET "Time_Minute" = minute
    WHERE "Time_TimeType" = kind;

EXECUTE update_time(1, (select hour from kt where kind = 1 LIMIT 1), (select min from kt where kind = 1 LIMIT 1));

UPDATE "Flips"
SET "Time_Hour" = (select hour from kt where kind = 1 LIMIT 1)
SET "Time_Minute" = (select min from kt where kind = 1 LIMIT 1)
WHERE "Time_TimeKind" = 1;

UPDATE "Flips"
SET "Time_Hour" = (select hour from kt where kind = 2 LIMIT 1)
SET "Time_Minute" = (select min from kt where kind = 2 LIMIT 1)
WHERE "Time_TimeKind" = 2;

UPDATE "Flips"
SET "Time_Hour" = (select hour from kt where kind = 3 LIMIT 1)
SET "Time_Minute" = (select min from kt where kind = 3 LIMIT 1)
WHERE "Time_TimeKind" = 3;

UPDATE "Flips"
