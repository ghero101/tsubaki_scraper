use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudflareConfig {
    pub proxy: ProxyConfig,
    pub browser: BrowserConfig,
    pub fingerprint: FingerprintConfig,
    pub captcha: CaptchaConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub proxies: Vec<String>,
    pub rotation_interval_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub window_size: String,
    pub disable_webrtc: bool,
    pub disable_geolocation: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FingerprintConfig {
    pub enabled: bool,
    pub spoof_webgl: bool,
    pub spoof_canvas: bool,
    pub spoof_audio: bool,
    pub spoof_fonts: bool,
    pub random_user_agent: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaptchaConfig {
    pub enabled: bool,
    pub service: String,
    pub api_key: String,
    pub max_solve_time_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionConfig {
    pub enabled: bool,
    pub cookie_file: String,
    pub session_lifetime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub url: String,
    pub cookies: Vec<Cookie>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<u64>,
}

pub struct ProxyRotator {
    proxies: Vec<String>,
    current_index: usize,
    last_rotation: SystemTime,
    rotation_interval: Duration,
}

impl ProxyRotator {
    pub fn new(proxies: Vec<String>, rotation_interval_secs: u64) -> Self {
        Self {
            proxies,
            current_index: 0,
            last_rotation: SystemTime::now(),
            rotation_interval: Duration::from_secs(rotation_interval_secs),
        }
    }

    pub fn get_current(&mut self) -> Option<&str> {
        if self.proxies.is_empty() {
            return None;
        }

        // Rotate if interval has passed
        if self.last_rotation.elapsed().unwrap_or(Duration::ZERO) >= self.rotation_interval {
            self.current_index = (self.current_index + 1) % self.proxies.len();
            self.last_rotation = SystemTime::now();
            log::info!(
                "Rotated to proxy {}/{}",
                self.current_index + 1,
                self.proxies.len()
            );
        }

        Some(&self.proxies[self.current_index])
    }
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    config: SessionConfig,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Self {
        let sessions = if config.enabled {
            Self::load_sessions(&config.cookie_file).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            sessions: Arc::new(Mutex::new(sessions)),
            config,
        }
    }

    fn load_sessions(
        file_path: &str,
    ) -> Result<HashMap<String, SessionData>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let sessions: HashMap<String, SessionData> = serde_json::from_str(&content)?;
        Ok(sessions)
    }

    pub fn save_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Ok(());
        }

        let sessions = self.sessions.lock().unwrap();
        let content = serde_json::to_string_pretty(&*sessions)?;
        fs::write(&self.config.cookie_file, content)?;
        Ok(())
    }

    pub fn get_session(&self, url: &str) -> Option<SessionData> {
        if !self.config.enabled {
            return None;
        }

        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(url)?;

        // Check if session is still valid
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - session.created_at > self.config.session_lifetime_secs {
            return None;
        }

        Some(session.clone())
    }

    pub fn save_session(&self, url: String, cookies: Vec<Cookie>) {
        if !self.config.enabled {
            return;
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = SessionData {
            url: url.clone(),
            cookies,
            created_at: now,
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(url, session);
    }
}

impl CloudflareConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string("cloudflare_config.toml")?;
        let config: CloudflareConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                enabled: false,
                proxies: vec![],
                rotation_interval_secs: 300,
            },
            browser: BrowserConfig {
                headless: false,
                window_size: "1920,1080".to_string(),
                disable_webrtc: true,
                disable_geolocation: true,
            },
            fingerprint: FingerprintConfig {
                enabled: true,
                spoof_webgl: true,
                spoof_canvas: true,
                spoof_audio: true,
                spoof_fonts: true,
                random_user_agent: true,
            },
            captcha: CaptchaConfig {
                enabled: false,
                service: "2captcha".to_string(),
                api_key: String::new(),
                max_solve_time_secs: 120,
            },
            session: SessionConfig {
                enabled: true,
                cookie_file: "cloudflare_sessions.json".to_string(),
                session_lifetime_secs: 3600,
            },
        }
    }
}

/// Generate fingerprint spoofing JavaScript
pub fn get_fingerprint_spoofing_script(config: &FingerprintConfig) -> String {
    let mut scripts = Vec::new();

    if config.spoof_webgl {
        scripts.push(
            r#"
            // Spoof WebGL
            const getParameter = WebGLRenderingContext.prototype.getParameter;
            WebGLRenderingContext.prototype.getParameter = function(parameter) {
                if (parameter === 37445) {
                    return 'Intel Inc.';
                }
                if (parameter === 37446) {
                    return 'Intel(R) Iris(TM) Plus Graphics 640';
                }
                return getParameter.call(this, parameter);
            };
        "#,
        );
    }

    if config.spoof_canvas {
        scripts.push(
            r#"
            // Spoof Canvas fingerprint
            const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
            HTMLCanvasElement.prototype.toDataURL = function() {
                const context = this.getContext('2d');
                if (context) {
                    const imageData = context.getImageData(0, 0, this.width, this.height);
                    for (let i = 0; i < imageData.data.length; i += 4) {
                        imageData.data[i] = imageData.data[i] ^ 0x1;
                    }
                    context.putImageData(imageData, 0, 0);
                }
                return originalToDataURL.apply(this, arguments);
            };
        "#,
        );
    }

    if config.spoof_audio {
        scripts.push(
            r#"
            // Spoof Audio Context
            const AudioContext = window.AudioContext || window.webkitAudioContext;
            if (AudioContext) {
                const originalCreateOscillator = AudioContext.prototype.createOscillator;
                AudioContext.prototype.createOscillator = function() {
                    const oscillator = originalCreateOscillator.call(this);
                    const originalStart = oscillator.start;
                    oscillator.start = function() {
                        originalStart.apply(this, arguments);
                    };
                    return oscillator;
                };
            }
        "#,
        );
    }

    if config.spoof_fonts {
        scripts.push(
            r#"
            // Spoof Font fingerprinting
            Object.defineProperty(navigator, 'fonts', {
                get: () => ({
                    ready: Promise.resolve(),
                    check: () => true,
                    load: () => Promise.resolve([]),
                    clear: () => {},
                    delete: () => true,
                    entries: () => [][Symbol.iterator](),
                    forEach: () => {},
                    has: () => true,
                    keys: () => [][Symbol.iterator](),
                    values: () => [][Symbol.iterator](),
                    size: 0
                })
            });
        "#,
        );
    }

    // Remove navigator.webdriver
    scripts.push(
        r#"
        Object.defineProperty(navigator, 'webdriver', {
            get: () => undefined
        });

        // Remove chrome automation flags
        window.chrome = {
            runtime: {}
        };

        // Overwrite the `plugins` property to use a custom getter
        Object.defineProperty(navigator, 'plugins', {
            get: () => [1, 2, 3, 4, 5]
        });

        // Overwrite the `languages` property to use a custom getter
        Object.defineProperty(navigator, 'languages', {
            get: () => ['en-US', 'en']
        });

        // Remove Selenium indicators
        delete window.cdc_adoQpoasnfa76pfcZLmcfl_Array;
        delete window.cdc_adoQpoasnfa76pfcZLmcfl_Promise;
        delete window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol;
    "#,
    );

    scripts.join("\n")
}

/// Generate random realistic user agents
pub fn get_random_user_agent() -> &'static str {
    let user_agents = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15",
    ];

    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    user_agents[(now as usize) % user_agents.len()]
}

/// Solve CAPTCHA using configured service
pub async fn solve_captcha(
    config: &CaptchaConfig,
    site_key: &str,
    page_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    if !config.enabled {
        return Err("CAPTCHA solving is not enabled".into());
    }

    match config.service.as_str() {
        "2captcha" => solve_2captcha(&config.api_key, site_key, page_url).await,
        "anticaptcha" => solve_anticaptcha(&config.api_key, site_key, page_url).await,
        "capsolver" => solve_capsolver(&config.api_key, site_key, page_url).await,
        _ => Err(format!("Unsupported CAPTCHA service: {}", config.service).into()),
    }
}

async fn solve_2captcha(
    api_key: &str,
    site_key: &str,
    page_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Submit CAPTCHA
    let submit_url = format!(
        "https://2captcha.com/in.php?key={}&method=userrecaptcha&googlekey={}&pageurl={}",
        api_key, site_key, page_url
    );

    let response = client.get(&submit_url).send().await?.text().await?;

    if !response.starts_with("OK|") {
        return Err(format!("2Captcha submit error: {}", response).into());
    }

    let captcha_id = response.strip_prefix("OK|").unwrap();

    // Poll for result
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let result_url = format!(
            "https://2captcha.com/res.php?key={}&action=get&id={}",
            api_key, captcha_id
        );

        let result = client.get(&result_url).send().await?.text().await?;

        if result.starts_with("OK|") {
            return Ok(result.strip_prefix("OK|").unwrap().to_string());
        } else if result != "CAPCHA_NOT_READY" {
            return Err(format!("2Captcha solve error: {}", result).into());
        }
    }

    Err("2Captcha timeout".into())
}

async fn solve_anticaptcha(
    api_key: &str,
    site_key: &str,
    page_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Similar implementation for Anti-Captcha
    Err("Anti-Captcha not implemented yet".into())
}

async fn solve_capsolver(
    api_key: &str,
    site_key: &str,
    page_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Similar implementation for CapSolver
    Err("CapSolver not implemented yet".into())
}
