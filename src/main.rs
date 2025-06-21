use std::fs;
use mailparse::{parse_mail, MailHeaderMap};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to your .eml file
    let eml_data = fs::read("examples/example1.eml")?;

    // Parse the email
    let parsed = parse_mail(&eml_data)?;

    // Print all headers
    println!("== Headers ==");
    for header in &parsed.headers {
        println!("{}: {}", header.get_key(), header.get_value());
    }

    // Print specific headers
    if let Some(subject) = parsed.get_headers().get_first_value("Subject") {
        println!("\nSubject: {}", subject);
    }
    if let Some(from) = parsed.get_headers().get_first_value("From") {
        println!("From: {}", from);
    }

    // Print MIME body structure
    println!("\n== MIME Parts ==");
    print_body_parts(&parsed, 0)?;

    Ok(())
}

fn print_body_parts(part: &mailparse::ParsedMail, depth: usize) -> Result<(), Box<dyn std::error::Error>> {
    let indent = "  ".repeat(depth);

    if part.subparts.is_empty() {
        let content_type = part.get_headers()
            .get_first_value("Content-Type")
            .unwrap_or_else(|| "unknown".to_string());

        let content_disposition = part.get_headers()
            .get_first_value("Content-Disposition")
            .unwrap_or_else(|| "inline".to_string());

        println!("{indent}- Content-Type: {content_type}");
        println!("{indent}  Content-Disposition: {content_disposition}");

        let body = part.get_body()?;
        println!("{indent}  Body size: {} bytes", body.len());

        // Optional: Save attachment if it's not inline
        if content_disposition.contains("attachment") {
            if let Some(filename) = content_disposition.split("filename=").nth(1) {
                let filename = filename.trim_matches('"').to_string();
                std::fs::write(filename.clone(), body)?;
                println!("{indent}  Saved attachment: {}", filename);
            }
        }

    } else {
        println!("{indent}- Multipart: {}", part.ctype.mimetype);
        for subpart in &part.subparts {
            print_body_parts(subpart, depth + 1)?;
        }
    }

    Ok(())
}
