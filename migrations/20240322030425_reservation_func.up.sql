-- if user_id is null, find all reservations within during for the resource
-- if resource_id is null, find all reservations within during for the user
-- if both are null, find all reservations within during
-- if both set, find all reservations within during for the resource and user
CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during tstzrange)
    RETURNS TABLE(
        LIKE rsvp.reservations
    )
    AS $$
BEGIN
    IF uid IS NULL AND rid IS NULL THEN
        RETURN QUERY
        SELECT
            *
        FROM
            rsvp.reservations
        WHERE
            during @> rsvp.reservations.during;
    ELSIF uid IS NULL THEN
        RETURN QUERY
        SELECT
            *
        FROM
            rsvp.reservations
        WHERE
            rsvp.reservations.resource_id = rid
            AND during @> rsvp.reservations.during;
    ELSIF rid IS NULL THEN
        RETURN QUERY
        SELECT
            *
        FROM
            rsvp.reservations
        WHERE
            rsvp.reservations.user_id = uid
            AND during @> rsvp.reservations.during;
    ELSE
        RETURN QUERY
        SELECT
            *
        FROM
            rsvp.reservations
        WHERE
            rsvp.reservations.user_id = uid
            AND rsvp.reservations.resource_id = rid
            AND during @> rsvp.reservations.during;
    END IF;
END;
$$
LANGUAGE plpgsql;
