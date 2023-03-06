CREATE OR REPLACE FUNCTION _marker_update(marker_id BIGINT, user_id BIGINT) RETURNS FLOAT AS $$
		DECLARE ret BIGINT;
		DECLARE points DOUBLE PRECISION;
	BEGIN
		SELECT count (*)
		FROM markers
		WHERE id=marker_id
		INTO ret;

		IF(ret !=1) THEN
			RAISE EXCEPTION 'marker non trovato';
		END IF;

		UPDATE markers
		SET resolution_date=now(), solved_by=user_id
		WHERE id=marker_id AND resolution_date is NULL;
		get diagnostics ret = row_count;

		IF(ret !=1) THEN
			--RAISE NOTICE 'update row number %', ret;
			RAISE EXCEPTION 'marker risolto';
		END IF;
		
		SELECT marker_types.points
		INTO points
		FROM markers, marker_types
		WHERE markers.marker_types_id=marker_types.id;

		RETURN points;
	END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION assign_point(inp_user_id BIGINT, pt FLOAT) RETURNS VOID AS $$
		DECLARE ret BIGINT;
	BEGIN
		
		UPDATE users
		SET points=points+pt
		WHERE id=inp_user_id;
		get diagnostics ret = row_count;

		IF(ret !=1) THEN
		RAISE NOTICE 'update row number %', ret;
			RAISE EXCEPTION 'user id not found';
		END IF;
		
		--aggiornare tutti i gruppi

		WITH RECURSIVE to_update AS(
			-- non-recursive term
			SELECT groups.id
			FROM groups, users_groups_join
			WHERE users_groups_join.group_id=groups.id AND users_groups_join.user_id=inp_user_id
		UNION
			-- recursive term
			SELECT groups_groups_join.group_parent_id
			FROM to_update, groups_groups_join
			WHERE groups_groups_join.group_son_id = to_update.id
		)UPDATE groups
		SET points=points+pt
		FROM to_update
		WHERE to_update.id=groups.id;
	END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION resolve_marker(marker_id BIGINT, user_id BIGINT) RETURNS VOID AS $$
	DECLARE points DOUBLE PRECISION;
	BEGIN
		--mark marker as resolved and get points value
		SELECT _marker_update(marker_id, user_id)
		INTO points;
		
		--update all the points 
		PERFORM assign_point(user_id, points);
		
	END;
$$ LANGUAGE plpgsql;