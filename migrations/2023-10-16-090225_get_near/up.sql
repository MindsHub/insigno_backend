-- Your SQL goes here
CREATE OR REPLACE FUNCTION get_near(
	position_inp Geometry,
    user_id_inp BIGINT,
    include_resolved_inp BOOL
) RETURNS TABLE(
	id bigint,
    point geometry(Geometry,4326),
    creation_date timestamp with time zone,
    resolution_date timestamp with time zone,
    marker_types_id bigint,
    created_by bigint,
    solved_by bigint
) AS $$
	#variable_conflict use_column
	BEGIN
	RETURN QUERY
		SELECT *
		FROM markers
		WHERE ST_DWITHIN(markers.point, position_inp, 0.135)
		AND (resolution_date IS NULL OR include_resolved_inp)
		AND (SELECT COUNT (*) FROM marker_reports WHERE markers.id = reported_marker)<1
		AND (
			(SELECT COUNT (*) FROM marker_images WHERE markers.id = marker_images.refers_to AND (marker_images.verify_average>0.5 OR marker_images.approved) )>0
			OR ((user_id_inp IS NOT NULL) OR markers.created_by =user_id_inp) 
			);
	END;
$$ LANGUAGE plpgsql;