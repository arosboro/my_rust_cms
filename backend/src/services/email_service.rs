use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use rand::Rng;
use std::env;

#[derive(Debug)]
pub enum EmailError {
    ConfigError(String),
    SendError(String),
    ParseError(String),
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::ConfigError(msg) => write!(f, "Email config error: {}", msg),
            EmailError::SendError(msg) => write!(f, "Email send error: {}", msg),
            EmailError::ParseError(msg) => write!(f, "Email parse error: {}", msg),
        }
    }
}

impl std::error::Error for EmailError {}

pub struct EmailService {
    mailer: SmtpTransport,
    from_email: String,
    from_name: String,
    base_url: String,
}

impl EmailService {
    pub fn new() -> Result<Self, EmailError> {
        tracing::info!("🔧 Initializing EmailService...");
        
        // Get SMTP configuration from environment variables
        let smtp_server = env::var("SMTP_SERVER")
            .map_err(|_| EmailError::ConfigError("SMTP_SERVER not set".to_string()))?;
        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .map_err(|_| EmailError::ConfigError("Invalid SMTP_PORT".to_string()))?;
        let smtp_username = env::var("SMTP_USERNAME")
            .map_err(|_| EmailError::ConfigError("SMTP_USERNAME not set".to_string()))?;
        let smtp_password = env::var("SMTP_PASSWORD")
            .map_err(|_| EmailError::ConfigError("SMTP_PASSWORD not set".to_string()))?;
        let from_email = env::var("FROM_EMAIL")
            .map_err(|_| EmailError::ConfigError("FROM_EMAIL not set".to_string()))?;
        let from_name = env::var("FROM_NAME").unwrap_or_else(|_| "CMS System".to_string());
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        tracing::info!("📧 Email configuration:");
        tracing::info!("   SMTP Server: {}", smtp_server);
        tracing::info!("   SMTP Port: {}", smtp_port);
        tracing::info!("   SMTP Username: {}", smtp_username);
        tracing::info!("   SMTP Password: [HIDDEN - {} chars]", smtp_password.len());
        tracing::info!("   From Email: {}", from_email);
        tracing::info!("   From Name: {}", from_name);
        tracing::info!("   Base URL: {}", base_url);

        let credentials = Credentials::new(smtp_username, smtp_password);

        tracing::info!("🔗 Creating SMTP transport...");
        let mailer = SmtpTransport::relay(&smtp_server)
            .map_err(|e| {
                let err = EmailError::ConfigError(format!("Failed to create SMTP relay: {}", e));
                tracing::error!("❌ SMTP relay creation failed: {}", err);
                err
            })?
            .port(smtp_port)
            .credentials(credentials)
            .timeout(Some(std::time::Duration::from_secs(30))) // 30-second timeout
            .build();

        tracing::info!("✅ EmailService initialized successfully");

        Ok(EmailService {
            mailer,
            from_email,
            from_name,
            base_url,
        })
    }

    pub fn send_verification_email(
        &self,
        to_email: &str,
        username: &str,
        verification_token: &str,
    ) -> Result<(), EmailError> {
        tracing::info!("🚀 Starting email verification for: {}", to_email);
        
        let verification_url = format!(
            "{}/verify-email?token={}",
            self.base_url, verification_token
        );

        tracing::info!("📧 Verification URL: {}", verification_url);

        let subject = "Verify your email address";
        let body = format!(
            r#"
Hello {username},

Thank you for signing up! Please verify your email address by clicking the link below:

{verification_url}

This link will expire in 24 hours.

If you didn't create this account, you can safely ignore this email.

Best regards,
The CMS Team
            "#,
            username = username,
            verification_url = verification_url
        );

        tracing::info!("📝 Building email message...");

        let email = Message::builder()
            .from(
                format!("{} <{}>", self.from_name, self.from_email)
                    .parse()
                    .map_err(|e| {
                        let err = EmailError::ParseError(format!("Invalid from address: {}", e));
                        tracing::error!("❌ From address error: {}", err);
                        err
                    })?,
            )
            .to(to_email
                .parse()
                .map_err(|e| {
                    let err = EmailError::ParseError(format!("Invalid to address: {}", e));
                    tracing::error!("❌ To address error: {}", err);
                    err
                })?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|e| {
                let err = EmailError::ParseError(format!("Failed to build email: {}", e));
                tracing::error!("❌ Email build error: {}", err);
                err
            })?;

        tracing::info!("📤 Sending email via SMTP...");
        tracing::info!("   From: {} <{}>", self.from_name, self.from_email);
        tracing::info!("   To: {}", to_email);

        match self.mailer.send(&email) {
            Ok(response) => {
                tracing::info!("✅ Email sent successfully! Response: {:?}", response);
                Ok(())
            }
            Err(e) => {
                let err = EmailError::SendError(format!("SMTP failed: {}", e));
                tracing::error!("❌ Email send failed: {}", err);
                tracing::error!("   Full error: {:?}", e);
                Err(err)
            }
        }
    }

    pub fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        reset_token: &str,
    ) -> Result<(), EmailError> {
        let reset_url = format!("{}/reset-password?token={}", self.base_url, reset_token);

        let subject = "Reset your password";
        let body = format!(
            r#"
Hello {username},

You requested a password reset. Click the link below to reset your password:

{reset_url}

This link will expire in 1 hour.

If you didn't request this password reset, you can safely ignore this email.

Best regards,
The CMS Team
            "#,
            username = username,
            reset_url = reset_url
        );

        let email = Message::builder()
            .from(
                format!("{} <{}>", self.from_name, self.from_email)
                    .parse()
                    .map_err(|e| EmailError::ParseError(format!("Invalid from address: {}", e)))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| EmailError::ParseError(format!("Invalid to address: {}", e)))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|e| EmailError::ParseError(format!("Failed to build email: {}", e)))?;

        self.mailer
            .send(&email)
            .map_err(|e| EmailError::SendError(format!("Failed to send email: {}", e)))?;

        Ok(())
    }
}

/// Generate a secure random verification token
pub fn generate_verification_token() -> String {
    let mut rng = rand::thread_rng();
    let token: [u8; 32] = rng.gen();
    hex::encode(token)
}

/// Create a mock email service for development/testing
#[cfg(debug_assertions)]
pub struct MockEmailService {
    base_url: String,
}

#[cfg(debug_assertions)]
impl MockEmailService {
    pub fn new() -> Self {
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        MockEmailService { base_url }
    }

    pub fn send_verification_email(
        &self,
        to_email: &str,
        username: &str,
        verification_token: &str,
    ) -> Result<(), EmailError> {
        let verification_url = format!(
            "{}/verify-email?token={}",
            self.base_url, verification_token
        );

        println!("=== MOCK EMAIL SERVICE ===");
        println!("To: {}", to_email);
        println!("Subject: Verify your email address");
        println!("Body:");
        println!(
            r#"
Hello {username},

Thank you for signing up! Please verify your email address by clicking the link below:

{verification_url}

This link will expire in 24 hours.

If you didn't create this account, you can safely ignore this email.

Best regards,
The CMS Team
            "#,
            username = username,
            verification_url = verification_url
        );
        println!("=========================");

        Ok(())
    }

    pub fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        reset_token: &str,
    ) -> Result<(), EmailError> {
        let reset_url = format!("{}/reset-password?token={}", self.base_url, reset_token);

        println!("=== MOCK EMAIL SERVICE ===");
        println!("To: {}", to_email);
        println!("Subject: Reset your password");
        println!("Body:");
        println!(
            r#"
Hello {username},

You requested a password reset. Click the link below to reset your password:

{reset_url}

This link will expire in 1 hour.

If you didn't request this password reset, you can safely ignore this email.

Best regards,
The CMS Team
            "#,
            username = username,
            reset_url = reset_url
        );
        println!("=========================");

        Ok(())
    }
}