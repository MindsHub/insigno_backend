-- Your SQL goes here
CREATE OR REPLACE FUNCTION special_scoreboard(
) RETURNS TABLE(
	id BIGINT,
    name character varying(254),
	points DOUBLE PRECISION
) AS $$
	DECLARE start_date TIMESTAMP WITH TIME ZONE;
	DECLARE center_position GEOMETRY;
	BEGIN
	SELECT ( CAST ((CURRENT_TIMESTAMP::date) AS TIMESTAMP WITH TIME ZONE) - interval '2h') INTO start_date;
	SELECT ST_GeomFromText('POINT(12.311324 41.806265)', 4326) INTO center_position;
	RETURN QUERY
		SELECT users.id, users.name, CAST(SUM(tbl.points) AS DOUBLE PRECISION) AS points
            FROM (
                SELECT created_by AS user_id, 1 AS points
                FROM markers
                WHERE ST_DWITHIN(point, center_position, 31000, FALSE)
                    AND creation_date IS NOT NULL
                    AND creation_date >= start_date

                UNION ALL

                SELECT solved_by AS user_id, 10 AS points
                FROM markers
                WHERE ST_DWITHIN(point, center_position, 31000, FALSE)
                    AND solved_by IS NOT NULL
                    AND resolution_date IS NOT NULL
                    AND resolution_date >= start_date
            ) AS tbl
            JOIN users ON tbl.user_id = users.id
            GROUP BY users.id
            ORDER BY SUM(tbl.points) DESC
            LIMIT 100;
	END;
$$ LANGUAGE plpgsql;
--SELECT * FROM special_scoreboard()