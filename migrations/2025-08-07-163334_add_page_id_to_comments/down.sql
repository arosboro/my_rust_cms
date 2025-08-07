-- Remove the constraint first
ALTER TABLE comments DROP CONSTRAINT IF EXISTS chk_comment_target;

-- Remove the index
DROP INDEX IF EXISTS idx_comments_page_id;

-- Remove the foreign key constraint
ALTER TABLE comments DROP CONSTRAINT IF EXISTS fk_comments_page_id;

-- Remove the page_id column
ALTER TABLE comments DROP COLUMN IF EXISTS page_id;