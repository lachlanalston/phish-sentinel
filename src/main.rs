use std::fs;
use mailparse::{parse_mail, MailHeaderMap};
use regex::Regex;

//TO DO
// - Add SPF, DMARC & DKIM checker functionality
// SPF is the easiest and first one to implement

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the .eml file bytes
    let eml_data = fs::read("examples/example1.eml")?;
    // Parse email
    let parsed = parse_mail(&eml_data)?;
    let headers = parsed.get_headers();

    let mut sections: Vec<[String; 2]> = Vec::new();

    // Extract From
    if let Some(from_val) = headers.get_first_value("From") {
        let (name, email) = parse_name_email(&from_val);
        sections.push(["from name".to_string(), name]);
        sections.push(["from email".to_string(), email]);
    }

    // Extract To
    if let Some(to_val) = headers.get_first_value("To") {
        let (name, email) = parse_name_email(&to_val);
        sections.push(["to name".to_string(), name]);
        sections.push(["to email".to_string(), email]);
    }

    // Extract Subject
    if let Some(subject) = headers.get_first_value("Subject") {
        sections.push(["subject".to_string(), subject.trim().to_string()]);
    }

    // Extract auth headers (SPF, DKIM, DMARC)
    let auth_headers = get_auth_headers(&headers);
    for (name, value) in auth_headers {
        sections.push([format!("auth header - {}", name), value]);
    }

    // Extract sender IPs from Received headers
    let sender_ips = extract_ips_from_received(&headers);
    for ip in sender_ips {
        sections.push(["sender ip".to_string(), ip]);
    }

    // Print all sections as [name][value]
    for (idx, pair) in sections.iter().enumerate() {
        println!("Section {}: [{}][{}]", idx + 1, pair[0], pair[1]);
    }

    Ok(())
}

// Parse "Name <email@example.com>" or just "email@example.com"
fn parse_name_email(s: &str) -> (String, String) {
    let s = s.trim();
    if let Some(start) = s.find('<') {
        if let Some(end) = s.find('>') {
            let name = s[..start].trim().trim_matches('"').to_string();
            let email = s[start + 1..end].trim().to_string();
            return (name, email);
        }
    }
    // Fallback: no name part, only email
    ("".to_string(), s.to_string())
}

fn get_auth_headers(headers: &dyn MailHeaderMap) -> Vec<(String, String)> {
    let auth_headers = [
        "Authentication-Results",
        "Received-SPF",
        "DKIM-Signature",
        "ARC-Authentication-Results",
        "DMARC-Filter",
    ];
    auth_headers
        .iter()
        .filter_map(|&h| headers.get_first_value(h).map(|v| (h.to_string(), v)))
        .collect()
}

fn extract_ips_from_received(headers: &dyn MailHeaderMap) -> Vec<String> {
    let mut ips = Vec::new();
    let ip_re = Regex::new(r"(\d{1,3}\.){3}\d{1,3}").unwrap();
    for received in headers.get_all_values("Received") {
        for cap in ip_re.captures_iter(&received) {
            ips.push(cap[0].to_string());
        }
    }
    ips
}
