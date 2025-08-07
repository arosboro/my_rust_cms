-- Add email verification fields to users table
ALTER TABLE users ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN email_verification_token VARCHAR;
ALTER TABLE users ADD COLUMN email_verification_expires_at TIMESTAMP;

-- Make email required for new signups by updating the constraint
-- Note: We'll handle this in the application logic rather than database constraint
-- to maintain backward compatibility with existing users