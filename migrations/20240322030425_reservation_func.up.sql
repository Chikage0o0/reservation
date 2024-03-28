-- if user_id is null, find all reservations within during for the resource
-- if resource_id is null, find all reservations within during for the user
-- if both are null, find all reservations within during
-- if both set, find all reservations within during for the resource and user
CREATE OR REPLACE FUNCTION rsvp.query(
    uid text,
    rid text,
    during tstzrange,
    status rsvp.reservation_status DEFAULT 'unknown',
    page integer DEFAULT 1,
    is_desc bool DEFAULT FALSE,
    page_size integer DEFAULT 10
) RETURNS TABLE(LIKE rsvp.reservations) AS $$
DECLARE
    _sql text;
BEGIN
    IF page < 1 THEN
        page := 1;
    END IF;

    IF page_size < 1 OR page_size > 100 THEN
        page_size := 10;
    END IF;

    -- format the query
    _sql := format('SELECT * FROM rsvp.reservations WHERE %L @> timespan AND %s AND %s ORDER BY lower(timespan) %s LIMIT   %s  OFFSET  %s',
        during,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN
                'TRUE'
            WHEN uid IS NULL THEN
                'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN
                'user_id = ' || quote_literal(uid)
            ELSE
                'user_id = ' || quote_literal(uid) || ' AND resource_id = ' || quote_literal(rid)
        END,
        CASE
            WHEN status = 'unknown' THEN
                'TRUE'
            ELSE
                'status = ' || quote_literal(status)
        END,
        CASE
            WHEN is_desc THEN
                'DESC'
            ELSE
                'ASC'
        END,
        page,
        (page - 1) * page_size
    );

    RETURN QUERY EXECUTE _sql;
END;
$$
LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION rsvp.filter(
    uid text,
    rid text,
    status rsvp.reservation_status DEFAULT 'unknown',
    page integer DEFAULT 1,
    is_desc bool DEFAULT FALSE,
    page_size integer DEFAULT 10
) RETURNS TABLE(LIKE rsvp.reservations) AS $$
DECLARE
    _sql text;
BEGIN
    -- format the query
    _sql := format('SELECT * FROM rsvp.reservations WHERE %L @> timespan AND %s AND %s ORDER BY lower(timespan) %s LIMIT   %s  OFFSET  %s',
        during,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN
                'TRUE'
            WHEN uid IS NULL THEN
                'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN
                'user_id = ' || quote_literal(uid)
            ELSE
                'user_id = ' || quote_literal(uid) || ' AND resource_id = ' || quote_literal(rid)
        END,
        CASE
            WHEN status = 'unknown' THEN
                'TRUE'
            ELSE
                'status = ' || quote_literal(status)
        END,
        CASE
            WHEN is_desc THEN
                'DESC'
            ELSE
                'ASC'
        END,
        page,
        (page - 1) * page_size
    );

    RETURN QUERY EXECUTE _sql;
END;

$$
LANGUAGE plpgsql;
