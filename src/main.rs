use std::fs;
use mailparse::{parse_mail, MailHeaderMap};
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the .eml file bytes
    let eml_data = fs::read("examples/example1.eml")?;

    // Parse email
    let parsed = parse_mail(&eml_data)?;

    // Store each top-level header as separate entry in a vector (for analysis)
    let mut sections: Vec<String> = Vec::new();
    for header in &parsed.headers {
        let line = format!("Header - {}: {}", header.get_key(), header.get_value());
        sections.push(line);
    }

    // Extract auth headers (SPF, DKIM, DMARC)
    let auth_headers = get_auth_headers(&parsed.get_headers());
    for (name, value) in &auth_headers {
        sections.push(format!("Auth Header - {}: {}", name, value));
    }

    // Extract sender IPs from Received headers
    let sender_ips = extract_ips_from_received(&parsed.get_headers());
    for ip in &sender_ips {
        sections.push(format!("Sender IP: {}", ip));
    }

    // Check Reply-To spoofing
    let (from, reply_to) = get_header_addresses(&parsed.get_headers());
    if let (Some(f), Some(r)) = (&from, &reply_to) {
        if f != r {
            sections.push(format!("Warning: Reply-To spoofing detected. From='{}' Reply-To='{}'", f, r));
        }
    }

    // Process MIME parts recursively, collecting info about each part
    process_mime_parts(&parsed, &mut sections, 0)?;

    // Print all collected sections with numbering
    for (idx, section) in sections.iter().enumerate() {
        println!("Section {}: {}", idx + 1, section);
    }

    Ok(())
}

fn process_mime_parts(part: &mailparse::ParsedMail, sections: &mut Vec<String>, depth: usize) -> Result<(), Box<dyn std::error::Error>> {
    let indent = "  ".repeat(depth);

    if part.subparts.is_empty() {
        let content_type = part.get_headers()
            .get_first_value("Content-Type")
            .unwrap_or_else(|| "unknown".to_string());

        let content_disp = part.get_headers()
            .get_first_value("Content-Disposition")
            .unwrap_or_else(|| "inline".to_string());

        // Add MIME part info
        sections.push(format!("{}MIME Part - {} ({} bytes)", indent, content_type, part.get_body()?.len()));

        // Add each header in this part
        for header in &part.headers {
            sections.push(format!("{}  Header - {}: {}", indent, header.get_key(), header.get_value()));
        }

        // Extract URLs from body
        let body = part.get_body()?;
        let urls = extract_urls(&body);
        if !urls.is_empty() {
            sections.push(format!("{}  URLs found: {:?}", indent, urls));
        }

        // Check for suspicious attachment
        if content_disp.contains("attachment") {
            if let Some(filename) = extract_filename(&content_disp) {
                sections.push(format!("{}  Attachment filename: {}", indent, filename));
                if is_suspicious_attachment(&filename) {
                    sections.push(format!("{}  Warning: Suspicious attachment filetype detected!", indent));
                }
            }
        }

    } else {
        // Multipart
        let mime_type = &part.ctype.mimetype;
        sections.push(format!("{}Multipart MIME Part: {}", indent, mime_type));
        for subpart in &part.subparts {
            process_mime_parts(subpart, sections, depth + 1)?;
        }
    }

    Ok(())
}

// Extract URLs using regex from a string body
fn extract_urls(body: &str) -> Vec<String> {
    let url_re = Regex::new(r#"https?://[^\s"'>]+"#).unwrap();
    url_re.find_iter(body).map(|m| m.as_str().to_string()).collect()
}

// Extract auth headers like SPF, DKIM, DMARC
fn get_auth_headers(headers: &dyn mailparse::MailHeaderMap) -> Vec<(String, String)> {
    let auth_headers = [
        "Authentication-Results",
        "Received-SPF",
        "DKIM-Signature",
        "ARC-Authentication-Results",
        "DMARC-Filter"
    ];
    auth_headers.iter()
        .filter_map(|&h| headers.get_first_value(h).map(|v| (h.to_string(), v)))
        .collect()
}

// Extract IP addresses from Received headers
fn extract_ips_from_received(headers: &dyn mailparse::MailHeaderMap) -> Vec<String> {
    let mut ips = Vec::new();
    let ip_re = Regex::new(r"(\d{1,3}\.){3}\d{1,3}").unwrap();
    for received in headers.get_all_values("Received") {
        for cap in ip_re.captures_iter(&received) {
            ips.push(cap[0].to_string());
        }
    }
    ips
}

// Get From and Reply-To header values
fn get_header_addresses(headers: &dyn mailparse::MailHeaderMap) -> (Option<String>, Option<String>) {
    (headers.get_first_value("From"), headers.get_first_value("Reply-To"))
}

// Check suspicious file extensions in attachments
fn is_suspicious_attachment(filename: &str) -> bool {
    let suspicious_exts = ["exe", "scr", "js", "bat", "cmd", "vbs", "docm", "xlsm", "pptm", "zip", "rar"];
    suspicious_exts.iter().any(|ext| filename.to_lowercase().ends_with(ext))
}

fn extract_filename(content_disp: &str) -> Option<String> {
    let filename_re = Regex::new(r#"filename="?([^";]+)"#).unwrap();
    filename_re.captures(content_disp).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_string())
}
