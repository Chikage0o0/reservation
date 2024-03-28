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
        CASE WHEN page IS NULL THEN
            1
        ELSE
            page
        END,
        CASE WHEN page_size IS NULL AND page IS NULL THEN
            0
        WHEN page_size IS NULL THEN
            (page - 1) * 10
        ELSE
            (page - 1) * page_size
        END
    );

    RETURN QUERY EXECUTE _sql;
END;

$$
LANGUAGE plpgsql;
