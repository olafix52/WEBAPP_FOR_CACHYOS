use anyhow::{Context, Result};
use gtk::glib;
use gtk::{AlertDialog, Button, Entry, FileChooserNative, FileFilter, Orientation, ResponseType};
use gtk4 as gtk;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, ComboRow, HeaderBar, PreferencesGroup,
    PreferencesPage,
};
use std::fs;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

const APP_ID: &str = "org.cachyos.webappmanager";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("CachyOS Web App Manager")
        .default_width(600)
        .default_height(500)
        .build();

    let content = gtk::Box::new(Orientation::Vertical, 0);

    let header = HeaderBar::new();
    content.append(&header);

    let page = PreferencesPage::new();
    let group = PreferencesGroup::new();
    group.set_title("Web App Details");

    // Inputs
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("App Name (e.g., WhatsApp)"));
    let name_row = ActionRow::new();
    name_row.set_title("Name");
    name_row.add_suffix(&name_entry);
    group.add(&name_row);

    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("URL (e.g., https://web.whatsapp.com)"));
    let url_row = ActionRow::new();
    url_row.set_title("URL");
    url_row.add_suffix(&url_entry);
    group.add(&url_row);

    // Icon Picker
    let icon_entry = Entry::new();
    icon_entry.set_placeholder_text(Some("Path to icon"));
    let icon_btn = Button::with_label("Choose...");
    let download_icon_btn = Button::with_label("Download");
    let icon_row = ActionRow::new();
    icon_row.set_title("Icon");
    icon_row.add_suffix(&icon_entry);
    icon_row.add_suffix(&icon_btn);
    icon_row.add_suffix(&download_icon_btn);
    group.add(&icon_row);

    // Browser Selection
    let detected_browsers = detect_browsers();
    let browser_model = gtk::StringList::new(
        &detected_browsers
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
    );
    let browser_combo = ComboRow::new();
    browser_combo.set_title("Browser");
    browser_combo.set_model(Some(&browser_model));
    group.add(&browser_combo);

    // Category Selection
    let categories = vec![
        "Network",
        "Office",
        "Graphics",
        "AudioVideo",
        "Development",
        "Game",
        "Utility",
    ];
    let category_model = gtk::StringList::new(&categories);
    let category_combo = ComboRow::new();
    category_combo.set_title("Category");
    category_combo.set_model(Some(&category_model));
    group.add(&category_combo);

    page.add(&group);
    content.append(&page);

    // Create Button
    let create_btn = Button::with_label("Create Web App");
    create_btn.add_css_class("suggested-action");
    create_btn.set_margin_top(20);
    create_btn.set_margin_bottom(20);
    create_btn.set_halign(gtk::Align::Center);
    content.append(&create_btn);

    window.set_content(Some(&content));

    // Logic Handling
    let icon_entry_clone = icon_entry.clone();
    let window_clone = window.clone();

    icon_btn.connect_clicked(move |_| {
        let file_chooser = FileChooserNative::new(
            Some("Select Icon"),
            Some(&window_clone),
            gtk::FileChooserAction::Open,
            Some("Select"),
            Some("Cancel"),
        );

        let filter = FileFilter::new();
        filter.add_pixbuf_formats();
        filter.set_name(Some("Images"));
        file_chooser.add_filter(&filter);

        let entry = icon_entry_clone.clone();
        file_chooser.connect_response(move |dialog: &FileChooserNative, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        entry.set_text(&path.to_string_lossy());
                    }
                }
            }
            dialog.destroy();
        });

        file_chooser.show();
    });

    // Icon Download Logic
    let icon_entry_clone_2 = icon_entry.clone();
    let url_entry_clone = url_entry.clone();
    let window_clone_2 = window.clone();

    download_icon_btn.connect_clicked(move |_| {
        let url_text = url_entry_clone.text().to_string();
        if url_text.trim().is_empty() {
            show_alert(&window_clone_2, "Error", "Please enter a URL first.");
            return;
        }

        let entry = icon_entry_clone_2.clone();
        let win = window_clone_2.clone();

        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let result = download_best_icon(&url_text);
            let _ = sender.send(result);
        });

        glib::timeout_add_local(
            std::time::Duration::from_millis(100),
            move || match receiver.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(path) => entry.set_text(&path.to_string_lossy()),
                        Err(e) => show_alert(&win, "Download Failed", &e.to_string()),
                    }
                    glib::ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            },
        );
    });
    let window_clone = window.clone();
    create_btn.connect_clicked(move |_| {
        let name = name_entry.text().to_string();
        let url = url_entry.text().to_string();
        let icon = icon_entry.text().to_string();

        let browser_idx = browser_combo.selected();
        let browser = if !detected_browsers.is_empty() {
            detected_browsers[browser_idx as usize].clone()
        } else {
            "firefox".to_string() // Fallback
        };

        let category_idx = category_combo.selected();
        let category = categories[category_idx as usize];

        match create_web_app(&name, &url, &icon, &browser, category) {
            Ok(msg) => show_alert(&window_clone, "Success", &msg),
            Err(e) => show_alert(
                &window_clone,
                "Error",
                &format!("Failed to create web app: {:#}", e),
            ),
        }
    });

    window.present();
}

fn show_alert(window: &ApplicationWindow, title: &str, message: &str) {
    let alert = AlertDialog::builder()
        .modal(true)
        .detail(message)
        .message(title)
        .build();
    alert.show(Some(window));
}

fn detect_browsers() -> Vec<String> {
    let output = Command::new("pacman").arg("-Qq").output();

    let mut installed = Vec::new();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let packages: Vec<&str> = stdout.lines().collect();

        // Priority order
        let targets = vec![
            "firefox",
            "chromium",
            "brave-bin",
            "vivaldi",
            "google-chrome",
        ];

        for target in targets {
            if packages.contains(&target) {
                installed.push(target.to_string());
            }
        }
    }

    if installed.is_empty() {
        // Fallback for dev/testing if no pacman or no browsers found
        installed.push("firefox".to_string());
    }

    installed
}

fn create_web_app(
    name: &str,
    url: &str,
    icon: &str,
    browser: &str,
    category: &str,
) -> Result<String> {
    if name.trim().is_empty() || url.trim().is_empty() {
        anyhow::bail!("Name and URL are required.");
    }

    let safe_name = name
        .replace(" ", "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let home = dirs::home_dir().context("Could not find home directory")?;

    let apps_dir = home.join(".local/share/applications");
    let webapps_data_dir = home.join(".local/share/webapps").join(&safe_name);

    fs::create_dir_all(&apps_dir)?;

    // Prepare Exec command
    let exec_cmd: String = if browser.contains("firefox") {
        // Firefox Logic: Create profile if not exists
        let _ = Command::new("firefox")
            .arg("-CreateProfile")
            .arg(format!(
                "{} {}",
                safe_name,
                webapps_data_dir.to_string_lossy()
            ))
            .output();

        format!(
            "firefox --class \"WebApp-{}\" --name \"WebApp-{}\" --new-window {} -P {}",
            safe_name, safe_name, url, safe_name
        )
    } else {
        // Chromium/Brave/Vivaldi Logic
        fs::create_dir_all(&webapps_data_dir)?;
        let bin_name = match browser {
            "brave-bin" => "brave",
            "google-chrome" => "google-chrome-stable",
            _ => browser,
        };
        format!(
            "{} --app={} --user-data-dir={}",
            bin_name,
            url,
            webapps_data_dir.to_string_lossy()
        )
    };

    let desktop_content = format!(
        "[Desktop Entry]\n\
        Version=1.0\n\
        Type=Application\n\
        Name={}\n\
        Comment=Web App for {}\n\
        Exec={}\n\
        Icon={}\n\
        Terminal=false\n\
        Categories={};\n\
        StartupWMClass=WebApp-{}\n",
        name, url, exec_cmd, icon, category, safe_name
    );

    let desktop_file = apps_dir.join(format!("{}.desktop", safe_name));
    fs::write(&desktop_file, desktop_content)?;

    Ok(format!("Created {}", desktop_file.to_string_lossy()))
}

fn download_best_icon(url_str: &str) -> Result<std::path::PathBuf> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0")
        .build()?;

    // Fix URL scheme if missing
    let target_url = if !url_str.starts_with("http") {
        format!("https://{}", url_str)
    } else {
        url_str.to_string()
    };

    let resp = client.get(&target_url).send()?;
    let base_url = resp.url().clone();
    let body = resp.text()?;

    let document = scraper::Html::parse_document(&body);
    let icon_selector = scraper::Selector::parse("link[rel~='icon']").unwrap();
    let apple_selector = scraper::Selector::parse("link[rel~='apple-touch-icon']").unwrap();

    let mut icon_url_str = None;

    // Try apple touch icon first (usually higher res)
    if let Some(element) = document.select(&apple_selector).next() {
        if let Some(href) = element.value().attr("href") {
            icon_url_str = Some(href.to_string());
        }
    }

    // Try regular icon
    if icon_url_str.is_none() {
        if let Some(element) = document.select(&icon_selector).next() {
            if let Some(href) = element.value().attr("href") {
                icon_url_str = Some(href.to_string());
            }
        }
    }

    let final_icon_url = if let Some(href) = icon_url_str {
        base_url.join(&href)?
    } else {
        base_url.join("/favicon.ico")?
    };

    // Download
    let icon_resp = client.get(final_icon_url.clone()).send()?;
    if !icon_resp.status().is_success() {
        anyhow::bail!("Failed to download icon from {}", final_icon_url);
    }

    let bytes = icon_resp.bytes()?;

    // Save to cache
    let home = dirs::home_dir().context("No home dir")?;
    let cache_dir = home.join(".cache/cachyos_webapp_manager/icons");
    fs::create_dir_all(&cache_dir)?;

    // Derive filename from host
    let host = base_url.host_str().unwrap_or("webapp");
    // Simple verification if it's png or ico from headers or extension?
    // Just save as is, or try to guess extension.
    let extension = final_icon_url.path().split('.').last().unwrap_or("png");
    // Basic sanitization
    let safe_host = host.replace(|c: char| !c.is_alphanumeric() && c != '-', "");
    let filename = format!("{}.{}", safe_host, extension);
    let path = cache_dir.join(filename);

    fs::write(&path, bytes)?;

    Ok(path)
}
