-- Remove email verification fields from users table
ALTER TABLE users DROP COLUMN email_verification_expires_at;
ALTER TABLE users DROP COLUMN email_verification_token;
ALTER TABLE users DROP COLUMN email_verified;