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

                let mut vols = vec![
                    YamlVal::String(format!("{}/www:/var/www/html", project.directory)),
                ];
                vols.push(YamlVal::String(format!("{}/php/php.ini:/usr/local/etc/php/conf.d/dockstack.ini", project.directory)));
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
                    YamlVal::String(format!("{}/www:/usr/local/apache2/htdocs/", project.directory)),
                    YamlVal::String("./apache/httpd.conf:/usr/local/apache2/conf/httpd.conf".to_string()),
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
                    YamlVal::String(format!("{}/www:/usr/share/nginx/html", project.directory)),
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

    // Write apache config if apache is enabled
    if project.services.get("apache").map_or(false, |s| s.enabled) {
        write_apache_config(project)?;
    }

    // Write default index.php if directory is empty
    write_default_index(project)?;

    // Write php config if php is enabled
    if project.services.get("php").map_or(false, |s| s.enabled) {
        write_php_config(project)?;
    }

    Ok(path.to_string_lossy().to_string())
}

fn write_php_config(project: &ProjectConfig) -> std::io::Result<()> {
    let php_dir = Path::new(&project.directory).join("php");
    fs::create_dir_all(&php_dir)?;
    
    let ini_path = php_dir.join("php.ini");
    let svc = project.services.get("php").unwrap();
    
    let mem_limit = svc.settings.get("memory_limit").cloned().unwrap_or_else(|| "256M".to_string());
    let extensions = svc.settings.get("extensions").cloned().unwrap_or_else(|| "".to_string());
    
    let mut content = format!("memory_limit = {}\n", mem_limit);
    content.push_str("upload_max_filesize = 100M\n");
    content.push_str("post_max_size = 100M\n");
    content.push_str("max_execution_time = 300\n");
    content.push_str("display_errors = On\n");
    content.push_str("error_reporting = E_ALL\n");
    
    // Note: Extensions in docker-php image usually need docker-php-ext-install but some basic ones can be loaded if they are shared.
    // However, for this to be 'Easy', we might need to use a richer image or dynamic installation.
    // For now, we setting up the INI for things that can be configured there.
    
    fs::write(ini_path, content)?;
    Ok(())
}

fn write_nginx_config(project: &ProjectConfig) -> std::io::Result<()> {
    let nginx_dir = Path::new(&project.directory).join("nginx");
    fs::create_dir_all(&nginx_dir)?;

    let config = if project.ssl_enabled {
        format!(r#"server {{
    listen 80;
    server_name {};
    return 301 https://$server_name$request_uri;
}}

server {{
    listen 443 ssl;
    server_name {};

    ssl_certificate /etc/nginx/certs/server.crt;
    ssl_certificate_key /etc/nginx/certs/server.key;

    root /usr/share/nginx/html;
    index index.php index.html;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location ~ \.php$ {{
        fastcgi_pass php:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }}
}}
"#, project.domain, project.domain)
    } else {
        format!(r#"server {{
    listen 80;
    server_name {};

    root /usr/share/nginx/html;
    index index.php index.html;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location ~ \.php$ {{
        fastcgi_pass php:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }}
}}
"#, project.domain)
    };

    let config_path = nginx_dir.join("default.conf");
    fs::write(config_path, config)?;
    Ok(())
}

fn write_apache_config(project: &ProjectConfig) -> std::io::Result<()> {
    let apache_dir = Path::new(&project.directory).join("apache");
    fs::create_dir_all(&apache_dir)?;

    let config_path = apache_dir.join("httpd.conf");

    // Basic Apache 2.4 config with DirectoryIndex and .htaccess enabled
    let mut config = format!(r#"
ServerRoot "/usr/local/apache2"
Listen 80
ServerName {}
"#, project.domain);
    config.push_str(r#"
LoadModule mpm_event_module modules/mod_mpm_event.so
LoadModule authz_core_module modules/mod_authz_core.so
LoadModule authz_host_module modules/mod_authz_host.so
LoadModule dir_module modules/mod_dir.so
LoadModule mime_module modules/mod_mime.so
LoadModule log_config_module modules/mod_log_config.so
LoadModule unixd_module modules/mod_unixd.so
LoadModule rewrite_module modules/mod_rewrite.so
LoadModule proxy_module modules/mod_proxy.so
LoadModule proxy_fcgi_module modules/mod_proxy_fcgi.so

User daemon
Group daemon

ServerAdmin you@example.com
DocumentRoot "/usr/local/apache2/htdocs"

<Directory />
    AllowOverride none
    Require all denied
</Directory>

<Directory "/usr/local/apache2/htdocs">
    Options Indexes FollowSymLinks
    AllowOverride All
    Require all granted
</Directory>

<IfModule dir_module>
    DirectoryIndex index.php index.html
</IfModule>

<IfModule log_config_module>
    LogFormat "%h %l %u %t \"%r\" %>s %b \"%{Referer}i\" \"%{User-Agent}i\"" combined
    CustomLog /proc/self/fd/1 combined
    ErrorLog /proc/self/fd/2
</IfModule>

<Files ".ht*">
    Require all denied
</Files>

<FilesMatch \.php$>
    SetHandler "proxy:fcgi://php:9000"
</FilesMatch>
"#);

    fs::write(config_path, config)?;
    Ok(())
}

fn write_default_index(project: &ProjectConfig) -> std::io::Result<()> {
    let www_dir = Path::new(&project.directory).join("www");
    fs::create_dir_all(&www_dir)?;

    let index_php = www_dir.join("index.php");
    let index_html = www_dir.join("index.html");

    if !index_php.exists() && !index_html.exists() {
        let content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>DockStack - {}</title>
    <style>
        body {{ font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #f0f2f5; }}
        .container {{ text-align: center; padding: 2rem; background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1 {{ color: #1a73e8; }}
        p {{ color: #5f6368; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>DockStack ⚡</h1>
        <p>Your service is up and running!</p>
        <p>Project: <strong>{}</strong></p>
        <p><small>PHP Version: <?php echo phpversion(); ?></small></p>
    </div>
</body>
</html>"#, project.name, project.name);
        
        fs::write(index_php, content)?;
    }
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
