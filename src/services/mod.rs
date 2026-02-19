#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub default_port: u16,
    pub category: ServiceCategory,
    pub icon: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceCategory {
    Database,
    WebServer,
    Runtime,
    Admin,
    Cache,
    Security,
    Custom,
}

impl ServiceCategory {
    pub fn label(&self) -> &str {
        match self {
            Self::Database => "📦 Database",
            Self::WebServer => "🌐 Web Server",
            Self::Runtime => "⚙️ Runtime",
            Self::Admin => "🔧 Admin Tools",
            Self::Cache => "💾 Cache",
            Self::Security => "🔒 Security",
            Self::Custom => "🧩 Custom Services",
        }
    }
}

pub fn get_service_registry() -> Vec<ServiceInfo> {
    vec![
        ServiceInfo {
            name: "postgresql".to_string(),
            display_name: "PostgreSQL".to_string(),
            description: "Advanced open source relational database".to_string(),
            default_port: 5432,
            category: ServiceCategory::Database,
            icon: "🐘",
        },
        ServiceInfo {
            name: "mysql".to_string(),
            display_name: "MySQL".to_string(),
            description: "Popular open source relational database".to_string(),
            default_port: 3306,
            category: ServiceCategory::Database,
            icon: "🐬",
        },
        ServiceInfo {
            name: "redis".to_string(),
            display_name: "Redis".to_string(),
            description: "In-memory data structure store".to_string(),
            default_port: 6379,
            category: ServiceCategory::Cache,
            icon: "⚡",
        },
        ServiceInfo {
            name: "nginx".to_string(),
            display_name: "Nginx".to_string(),
            description: "High performance web server & reverse proxy".to_string(),
            default_port: 80,
            category: ServiceCategory::WebServer,
            icon: "🌐",
        },
        ServiceInfo {
            name: "apache".to_string(),
            display_name: "Apache".to_string(),
            description: "The most widely used web server".to_string(),
            default_port: 8080,
            category: ServiceCategory::WebServer,
            icon: "🎯",
        },
        ServiceInfo {
            name: "php".to_string(),
            display_name: "PHP-FPM".to_string(),
            description: "PHP FastCGI Process Manager".to_string(),
            default_port: 9000,
            category: ServiceCategory::Runtime,
            icon: "🐘",
        },
        ServiceInfo {
            name: "phpmyadmin".to_string(),
            display_name: "phpMyAdmin".to_string(),
            description: "Web interface for MySQL administration".to_string(),
            default_port: 8081,
            category: ServiceCategory::Admin,
            icon: "🔧",
        },
        ServiceInfo {
            name: "pgadmin".to_string(),
            display_name: "pgAdmin".to_string(),
            description: "Web interface for PostgreSQL administration".to_string(),
            default_port: 8082,
            category: ServiceCategory::Admin,
            icon: "🔧",
        },
        ServiceInfo {
            name: "adminer".to_string(),
            display_name: "Adminer".to_string(),
            description: "Universal database management in single PHP file".to_string(),
            default_port: 8083,
            category: ServiceCategory::Admin,
            icon: "🗄️",
        },
        ServiceInfo {
            name: "ssl".to_string(),
            display_name: "SSL/HTTPS".to_string(),
            description: "Self-signed HTTPS reverse proxy".to_string(),
            default_port: 443,
            category: ServiceCategory::Security,
            icon: "🔐",
        },
    ]
}

pub fn get_service_info(name: &str) -> Option<ServiceInfo> {
    get_service_registry().into_iter().find(|s| s.name == name)
}
