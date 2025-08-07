-- Add page_id column to comments table to support commenting on pages
ALTER TABLE comments ADD COLUMN page_id INTEGER;

-- Add foreign key constraint to pages table
ALTER TABLE comments ADD CONSTRAINT fk_comments_page_id 
    FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE;

-- Create index for better query performance
CREATE INDEX idx_comments_page_id ON comments(page_id);

-- Add constraint to ensure either post_id or page_id is set, but not both
ALTER TABLE comments ADD CONSTRAINT chk_comment_target 
    CHECK ((post_id IS NOT NULL AND page_id IS NULL) OR (post_id IS NULL AND page_id IS NOT NULL));