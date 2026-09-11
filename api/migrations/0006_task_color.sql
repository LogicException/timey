ALTER TABLE tasks ADD COLUMN color TEXT NOT NULL DEFAULT '#5a8f6d'
    CHECK (color GLOB '#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]');

UPDATE tasks SET color = CASE id % 8
    WHEN 0 THEN '#e4b04a'
    WHEN 1 THEN '#3f9d6c'
    WHEN 2 THEN '#d45b49'
    WHEN 3 THEN '#6b8cae'
    WHEN 4 THEN '#c47a3a'
    WHEN 5 THEN '#7a9e5c'
    WHEN 6 THEN '#8b6bb0'
    ELSE '#4a9ea0'
END;
