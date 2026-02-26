#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub projects: Vec<ProjectConfig>,
    pub active_project_id: Option<String>,
    pub docker_path: String,
    pub compose_path: String,
    pub theme: ThemeConfig,
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub dark_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub minimize_to_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub services: HashMap<String, ServiceConfig>,
    pub ssl_enabled: bool,
    pub custom_ports: HashMap<String, u16>,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub enabled: bool,
    pub port: u16,
    pub version: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub image: Option<String>, // For custom services
    #[serde(default)]
    pub is_custom: bool,       // Flag for user-added services
    #[serde(default)]
    pub is_locked: bool,       // If true, DockStack won't regenerate its config files
    pub env_vars: HashMap<String, String>,
    pub settings: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            projects: vec![ProjectConfig::default()],
            active_project_id: Some("default".to_string()),
            docker_path: "docker".to_string(),
            compose_path: "docker compose".to_string(),
            theme: ThemeConfig { dark_mode: true },
            window: WindowConfig {
                width: 1280.0,
                height: 800.0,
                minimize_to_tray: true,
            },
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let mut services = HashMap::new();

        services.insert(
            "postgresql".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 5432,
                version: "16".to_string(),
                env_vars: {
                    let mut m = HashMap::new();
                    m.insert("POSTGRES_USER".to_string(), "postgres".to_string());
                    m.insert("POSTGRES_PASSWORD".to_string(), "postgres".to_string());
                    m.insert("POSTGRES_DB".to_string(), "devdb".to_string());
                    m
                },
                settings: HashMap::new(),
            },
        );

        services.insert(
            "mysql".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 3306,
                version: "8.0".to_string(),
                env_vars: {
                    let mut m = HashMap::new();
                    m.insert("MYSQL_ROOT_PASSWORD".to_string(), "root".to_string());
                    m.insert("MYSQL_DATABASE".to_string(), "devdb".to_string());
                    m
                },
                settings: HashMap::new(),
            },
        );

        services.insert(
            "php".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 9000,
                version: "8.3-fpm".to_string(),
                env_vars: HashMap::new(),
                settings: {
                    let mut m = HashMap::new();
                    m.insert("extensions".to_string(), "pdo_mysql,gd,zip,intl".to_string());
                    m.insert("memory_limit".to_string(), "256M".to_string());
                    m
                },
            },
        );

        services.insert(
            "apache".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 8080,
                version: "2.4".to_string(),
                env_vars: HashMap::new(),
                settings: HashMap::new(),
            },
        );

        services.insert(
            "nginx".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 80,
                version: "latest".to_string(),
                env_vars: HashMap::new(),
                settings: HashMap::new(),
            },
        );

        services.insert(
            "phpmyadmin".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 8081,
                version: "latest".to_string(),
                env_vars: {
                    let mut m = HashMap::new();
                    m.insert("PMA_USER".to_string(), "root".to_string());
                    m.insert("PMA_PASSWORD".to_string(), "root".to_string());
                    m
                },
                settings: HashMap::new(),
            },
        );

        services.insert(
            "pgadmin".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 8082,
                version: "latest".to_string(),
                env_vars: {
                    let mut m = HashMap::new();
                    m.insert(
                        "PGADMIN_DEFAULT_EMAIL".to_string(),
                        "admin@admin.com".to_string(),
                    );
                    m.insert(
                        "PGADMIN_DEFAULT_PASSWORD".to_string(),
                        "admin".to_string(),
                    );
                    m
                },
                settings: HashMap::new(),
            },
        );

        services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 6379,
                version: "7".to_string(),
                env_vars: HashMap::new(),
                settings: HashMap::new(),
            },
        );

        services.insert(
            "adminer".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 8083,
                version: "latest".to_string(),
                env_vars: HashMap::new(),
                settings: HashMap::new(),
            },
        );

        services.insert(
            "ssl".to_string(),
            ServiceConfig {
                enabled: false,
                is_custom: false,
                is_locked: false,
                display_name: None,
                image: None,
                port: 443,
                version: "latest".to_string(),
                env_vars: HashMap::new(),
                settings: HashMap::new(),
            },
        );

        Self {
            id: "default".to_string(),
            name: "Default Project".to_string(),
            directory: dirs::home_dir()
                .unwrap_or_default()
                .join("dockstack-projects")
                .join("default")
                .to_string_lossy()
                .to_string(),
            services,
            ssl_enabled: false,
            custom_ports: HashMap::new(),
            domain: "dockstack.test".to_string(),
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dockstack");
        fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        log::error!("Failed to parse config: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("Failed to read config: {}", e);
                }
            }
        }
        let config = Self::default();
        config.save();
        config
    }

    pub fn save(&self) {
        let path = Self::config_path();
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    log::error!("Failed to save config: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize config: {}", e);
            }
        }
    }

    pub fn active_project(&self) -> Option<&ProjectConfig> {
        let id = self.active_project_id.as_ref()?;
        self.projects.iter().find(|p| &p.id == id)
    }

    pub fn active_project_mut(&mut self) -> Option<&mut ProjectConfig> {
        let id = self.active_project_id.clone()?;
        self.projects.iter_mut().find(|p| p.id == id)
    }

    pub fn add_project(&mut self, name: String) -> String {
        let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        let dir = dirs::home_dir()
            .unwrap_or_default()
            .join("dockstack-projects")
            .join(&id);
        fs::create_dir_all(&dir).ok();
        let project = ProjectConfig {
            id: id.clone(),
            name,
            directory: dir.to_string_lossy().to_string(),
            ..ProjectConfig::default()
        };
        self.projects.push(project);
        self.active_project_id = Some(id.clone());
        self.save();
        id
    }

    pub fn remove_project(&mut self, id: &str) {
        self.projects.retain(|p| p.id != id);
        if self.active_project_id.as_deref() == Some(id) {
            self.active_project_id = self.projects.first().map(|p| p.id.clone());
        }
        self.save();
    }

    pub fn import_from_compose(&mut self, yaml_path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(yaml_path)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
        
        let project_dir = yaml_path.parent().unwrap_or(std::path::Path::new("."));
        let project_name = project_dir.file_name().unwrap_or_default().to_string_lossy().to_string();
        
        let mut services = HashMap::new();
        
        if let Some(yaml_services) = yaml.get("services").and_then(|v| v.as_mapping()) {
            for (name_val, svc_val) in yaml_services {
                let name = name_val.as_str().unwrap_or("unknown").to_string();
                let mut svc = ServiceConfig {
                    enabled: true,
                    is_custom: true,
                    is_locked: false,
                    display_name: Some(name.clone()),
                    image: None,
                    port: 0,
                    version: "latest".to_string(),
                    env_vars: HashMap::new(),
                    settings: HashMap::new(),
                };
                
                if let Some(img) = svc_val.get("image").and_then(|v| v.as_str()) {
                    if img.contains(':') {
                        let parts: Vec<&str> = img.split(':').collect();
                        svc.image = Some(parts[0].to_string());
                        svc.version = parts[1].to_string();
                    } else {
                        svc.image = Some(img.to_string());
                    }
                }
                
                if let Some(ports) = svc_val.get("ports").and_then(|v| v.as_sequence()) {
                    if let Some(p_str) = ports[0].as_str() {
                        if let Some(host_port) = p_str.split(':').next().and_then(|p| p.parse::<u16>().ok()) {
                            svc.port = host_port;
                        }
                    }
                }
                
                services.insert(name, svc);
            }
        }
        
        let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        let project = ProjectConfig {
            id: id.clone(),
            name: format!("Imported: {}", project_name),
            directory: project_dir.to_string_lossy().to_string(),
            services,
            ssl_enabled: false,
            custom_ports: HashMap::new(),
            domain: format!("{}.test", project_name.to_lowercase().replace(' ', "-")),
        };
        
        self.projects.push(project);
        self.active_project_id = Some(id.clone());
        self.save();
        Ok(id)
    }
}

impl ProjectConfig {
    pub fn enabled_services(&self) -> Vec<String> {
        self.services
            .iter()
            .filter(|(_, v)| v.enabled)
            .map(|(k, _)| k.clone())
            .collect()
    }
}
