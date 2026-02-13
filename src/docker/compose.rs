use crate::config::ProjectConfig;
use serde_yaml;
use std::fs;
use std::path::Path;

type YamlMap = serde_yaml::Mapping;
type YamlVal = serde_yaml::Value;

pub fn generate_compose(project: &ProjectConfig) -> String {
    let mut root = YamlMap::new();
    let mut services = YamlMap::new();
    let mut volumes = YamlMap::new();
    let mut networks = YamlMap::new();

    let network_name = format!("dockstack_{}", project.id);

    for (name, svc) in &project.services {
        if !svc.enabled {
            continue;
        }
        match name.as_str() {
            "postgresql" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("postgres:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_postgres", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let mut env = YamlMap::new();
                for (k, v) in &svc.env_vars {
                    env.insert(y_str(k), y_str(v));
                }
                s.insert(y_str("environment"), YamlVal::Mapping(env));

                let ports = vec![YamlVal::String(format!("{}:5432", svc.port))];
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let vols = vec![YamlVal::String("postgres_data:/var/lib/postgresql/data".to_string())];
                s.insert(y_str("volumes"), YamlVal::Sequence(vols));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                s.insert(y_str("healthcheck"), healthcheck("pg_isready -U postgres", 10, 5, 5));

                services.insert(y_str("postgresql"), YamlVal::Mapping(s));
                volumes.insert(y_str("postgres_data"), YamlVal::Mapping(YamlMap::new()));
            }
            "mysql" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("mysql:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_mysql", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let mut env = YamlMap::new();
                for (k, v) in &svc.env_vars {
                    env.insert(y_str(k), y_str(v));
                }
                s.insert(y_str("environment"), YamlVal::Mapping(env));

                let ports = vec![YamlVal::String(format!("{}:3306", svc.port))];
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let vols = vec![YamlVal::String("mysql_data:/var/lib/mysql".to_string())];
                s.insert(y_str("volumes"), YamlVal::Sequence(vols));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                s.insert(y_str("healthcheck"), healthcheck("mysqladmin ping -h localhost", 10, 5, 5));

                services.insert(y_str("mysql"), YamlVal::Mapping(s));
                volumes.insert(y_str("mysql_data"), YamlVal::Mapping(YamlMap::new()));
            }
            "php" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("php:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_php", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let vols = vec![
                    YamlVal::String(format!("{}:/var/www/html", project.directory)),
                ];
                s.insert(y_str("volumes"), YamlVal::Sequence(vols));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                services.insert(y_str("php"), YamlVal::Mapping(s));
            }
            "apache" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("httpd:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_apache", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let ports = vec![YamlVal::String(format!("{}:80", svc.port))];
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let vols = vec![
                    YamlVal::String(format!("{}:/usr/local/apache2/htdocs/", project.directory)),
                ];
                s.insert(y_str("volumes"), YamlVal::Sequence(vols));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                services.insert(y_str("apache"), YamlVal::Mapping(s));
            }
            "nginx" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("nginx:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_nginx", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let mut ports = vec![YamlVal::String(format!("{}:80", svc.port))];
                if project.ssl_enabled {
                    ports.push(YamlVal::String("443:443".to_string()));
                }
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let mut vols = vec![
                    YamlVal::String(format!("{}:/usr/share/nginx/html", project.directory)),
                    YamlVal::String("./nginx/default.conf:/etc/nginx/conf.d/default.conf".to_string()),
                ];
                if project.ssl_enabled {
                    vols.push(YamlVal::String("./certs:/etc/nginx/certs:ro".to_string()));
                }
                s.insert(y_str("volumes"), YamlVal::Sequence(vols));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                services.insert(y_str("nginx"), YamlVal::Mapping(s));
            }
            "phpmyadmin" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("phpmyadmin:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_phpmyadmin", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let mut env = YamlMap::new();
                env.insert(y_str("PMA_HOST"), y_str("mysql"));
                env.insert(y_str("PMA_ARBITRARY"), y_str("1"));
                
                for (k, v) in &svc.env_vars {
                    env.insert(y_str(k), y_str(v));
                }
                
                s.insert(y_str("environment"), YamlVal::Mapping(env));

                let ports = vec![YamlVal::String(format!("{}:80", svc.port))];
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                let deps = vec![YamlVal::String("mysql".to_string())];
                if project.services.get("mysql").map_or(false, |s| s.enabled) {
                    s.insert(y_str("depends_on"), YamlVal::Sequence(deps));
                }

                services.insert(y_str("phpmyadmin"), YamlVal::Mapping(s));
            }
            "pgadmin" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("dpage/pgadmin4:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_pgadmin", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let mut env = YamlMap::new();
                for (k, v) in &svc.env_vars {
                    env.insert(y_str(k), y_str(v));
                }
                s.insert(y_str("environment"), YamlVal::Mapping(env));

                let ports = vec![YamlVal::String(format!("{}:80", svc.port))];
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let vols = vec![YamlVal::String("pgadmin_data:/var/lib/pgadmin".to_string())];
                s.insert(y_str("volumes"), YamlVal::Sequence(vols));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                if project.services.get("postgresql").map_or(false, |s| s.enabled) {
                    let deps = vec![YamlVal::String("postgresql".to_string())];
                    s.insert(y_str("depends_on"), YamlVal::Sequence(deps));
                }

                services.insert(y_str("pgadmin"), YamlVal::Mapping(s));
                volumes.insert(y_str("pgadmin_data"), YamlVal::Mapping(YamlMap::new()));
            }
            "redis" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("redis:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_redis", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let ports = vec![YamlVal::String(format!("{}:6379", svc.port))];
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let vols = vec![YamlVal::String("redis_data:/data".to_string())];
                s.insert(y_str("volumes"), YamlVal::Sequence(vols));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                s.insert(y_str("healthcheck"), healthcheck("redis-cli ping", 10, 5, 5));

                services.insert(y_str("redis"), YamlVal::Mapping(s));
                volumes.insert(y_str("redis_data"), YamlVal::Mapping(YamlMap::new()));
            }
            "adminer" => {
                let mut s = YamlMap::new();
                s.insert(y_str("image"), y_str(&format!("adminer:{}", svc.version)));
                s.insert(y_str("container_name"), y_str(&format!("dockstack_{}_adminer", project.id)));
                s.insert(y_str("restart"), y_str("unless-stopped"));

                let ports = vec![YamlVal::String(format!("{}:8080", svc.port))];
                s.insert(y_str("ports"), YamlVal::Sequence(ports));

                let nets = vec![YamlVal::String(network_name.clone())];
                s.insert(y_str("networks"), YamlVal::Sequence(nets));

                services.insert(y_str("adminer"), YamlVal::Mapping(s));
            }
            "ssl" => {
                // SSL is handled via nginx config, not as a separate service container.
                // The SSL toggle enables HTTPS on the nginx reverse proxy.
            }
            _ => {}
        }
    }

    // Network
    let mut net_conf = YamlMap::new();
    net_conf.insert(y_str("driver"), y_str("bridge"));
    networks.insert(y_str(&network_name), YamlVal::Mapping(net_conf));

    root.insert(y_str("services"), YamlVal::Mapping(services));
    if !volumes.is_empty() {
        root.insert(y_str("volumes"), YamlVal::Mapping(volumes));
    }
    root.insert(y_str("networks"), YamlVal::Mapping(networks));

    serde_yaml::to_string(&YamlVal::Mapping(root)).unwrap_or_default()
}

pub fn write_compose_file(project: &ProjectConfig) -> std::io::Result<String> {
    let dir = Path::new(&project.directory);
    fs::create_dir_all(dir)?;

    let compose = generate_compose(project);
    let path = dir.join("docker-compose.yml");
    fs::write(&path, &compose)?;

    // Write nginx config if nginx is enabled
    if project.services.get("nginx").map_or(false, |s| s.enabled) {
        write_nginx_config(project)?;
    }

    Ok(path.to_string_lossy().to_string())
}

fn write_nginx_config(project: &ProjectConfig) -> std::io::Result<()> {
    let nginx_dir = Path::new(&project.directory).join("nginx");
    fs::create_dir_all(&nginx_dir)?;

    let config = if project.ssl_enabled {
        r#"server {
    listen 80;
    server_name localhost;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl;
    server_name localhost;

    ssl_certificate /etc/nginx/certs/server.crt;
    ssl_certificate_key /etc/nginx/certs/server.key;

    root /usr/share/nginx/html;
    index index.php index.html;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass php:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }
}
"#
    } else {
        r#"server {
    listen 80;
    server_name localhost;

    root /usr/share/nginx/html;
    index index.php index.html;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass php:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }
}
"#
    };

    fs::write(nginx_dir.join("default.conf"), config)?;
    Ok(())
}

fn y_str(s: &str) -> YamlVal {
    YamlVal::String(s.to_string())
}

fn healthcheck(test: &str, interval: u32, timeout: u32, retries: u32) -> YamlVal {
    let mut hc = YamlMap::new();
    hc.insert(
        y_str("test"),
        YamlVal::Sequence(vec![
            y_str("CMD-SHELL"),
            y_str(test),
        ]),
    );
    hc.insert(y_str("interval"), y_str(&format!("{}s", interval)));
    hc.insert(y_str("timeout"), y_str(&format!("{}s", timeout)));
    hc.insert(y_str("retries"), YamlVal::Number(serde_yaml::Number::from(retries)));
    YamlVal::Mapping(hc)
}
