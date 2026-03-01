use std::fs;
use std::path::Path;
use std::process::Command;

pub struct SslManager;

#[allow(dead_code)]
impl SslManager {
    /// Generate self-signed SSL certificate
    pub fn generate_self_signed(project_dir: &str) -> Result<(String, String), String> {
        let certs_dir = Path::new(project_dir).join("certs");
        fs::create_dir_all(&certs_dir).map_err(|e| format!("Failed to create certs dir: {}", e))?;

        let cert_path = certs_dir.join("server.crt");
        let key_path = certs_dir.join("server.key");

        // Use rcgen to generate self-signed cert
        match Self::generate_with_rcgen(&cert_path, &key_path) {
            Ok(_) => Ok((
                cert_path.to_string_lossy().to_string(),
                key_path.to_string_lossy().to_string(),
            )),
            Err(e) => {
                log::warn!("rcgen failed: {}, falling back to openssl", e);
                Self::generate_with_openssl(&cert_path, &key_path)
            }
        }
    }

    fn generate_with_rcgen(cert_path: &Path, key_path: &Path) -> Result<(), String> {
        use rcgen::{CertificateParams, KeyPair};

        let mut params =
            CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()])
                .map_err(|e| format!("Failed to create cert params: {}", e))?;
        params.distinguished_name.push(
            rcgen::DnType::CommonName,
            rcgen::DnValue::Utf8String("DockStack Dev Certificate".to_string()),
        );
        params.distinguished_name.push(
            rcgen::DnType::OrganizationName,
            rcgen::DnValue::Utf8String("DockStack".to_string()),
        );

        let key_pair =
            KeyPair::generate().map_err(|e| format!("Failed to generate key pair: {}", e))?;
        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| format!("Failed to self-sign: {}", e))?;

        fs::write(cert_path, cert.pem()).map_err(|e| format!("Failed to write cert: {}", e))?;
        fs::write(key_path, key_pair.serialize_pem())
            .map_err(|e| format!("Failed to write key: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(key_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = fs::set_permissions(key_path, perms);
            }
        }

        Ok(())
    }

    fn generate_with_openssl(
        cert_path: &Path,
        key_path: &Path,
    ) -> Result<(String, String), String> {
        let output = Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-keyout",
                &key_path.to_string_lossy(),
                "-out",
                &cert_path.to_string_lossy(),
                "-days",
                "365",
                "-nodes",
                "-subj",
                "/C=US/ST=Dev/L=Local/O=DockStack/CN=localhost",
            ])
            .output()
            .map_err(|e| format!("Failed to run openssl: {}", e))?;

        if output.status.success() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(key_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    let _ = fs::set_permissions(key_path, perms);
                }
            }
            Ok((
                cert_path.to_string_lossy().to_string(),
                key_path.to_string_lossy().to_string(),
            ))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("openssl failed: {}", stderr))
        }
    }

    /// Check if SSL certificates exist for a project
    pub fn certs_exist(project_dir: &str) -> bool {
        let certs_dir = Path::new(project_dir).join("certs");
        certs_dir.join("server.crt").exists() && certs_dir.join("server.key").exists()
    }

    /// Remove SSL certificates
    pub fn remove_certs(project_dir: &str) -> Result<(), String> {
        let certs_dir = Path::new(project_dir).join("certs");
        if certs_dir.exists() {
            fs::remove_dir_all(&certs_dir).map_err(|e| format!("Failed to remove certs: {}", e))?;
        }
        Ok(())
    }
}
