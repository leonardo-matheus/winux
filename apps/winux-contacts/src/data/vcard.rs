// vCard Parser and Generator
// Supports vCard 3.0 and 4.0 formats

use crate::data::contact::*;
use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;

/// Parse vCard data and return a list of contacts
pub fn parse_vcard(data: &str) -> Result<Vec<Contact>> {
    let mut contacts = Vec::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut in_vcard = false;

    // Handle line folding (lines starting with space/tab are continuations)
    let unfolded = unfold_lines(data);

    for line in unfolded.lines() {
        let line = line.trim_end();

        if line.to_uppercase().starts_with("BEGIN:VCARD") {
            in_vcard = true;
            current_lines.clear();
        } else if line.to_uppercase().starts_with("END:VCARD") {
            if in_vcard {
                if let Ok(contact) = parse_single_vcard(&current_lines) {
                    contacts.push(contact);
                }
            }
            in_vcard = false;
        } else if in_vcard {
            current_lines.push(line.to_string());
        }
    }

    Ok(contacts)
}

fn unfold_lines(data: &str) -> String {
    let mut result = String::new();
    for line in data.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation line - append to previous
            result.push_str(line.trim_start());
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
    }
    result
}

fn parse_single_vcard(lines: &[String]) -> Result<Contact> {
    let mut contact = Contact::new();

    for line in lines {
        if let Some((property, params, value)) = parse_property_line(line) {
            match property.to_uppercase().as_str() {
                "N" => {
                    // N:Last;First;Middle;Prefix;Suffix
                    let parts: Vec<&str> = value.split(';').collect();
                    if parts.len() >= 2 {
                        contact.last_name = unescape_value(parts[0]);
                        contact.first_name = unescape_value(parts[1]);
                    }
                }
                "FN" => {
                    // Full name - use if N not set
                    if contact.first_name.is_empty() && contact.last_name.is_empty() {
                        let parts: Vec<&str> = value.splitn(2, ' ').collect();
                        if parts.len() == 2 {
                            contact.first_name = unescape_value(parts[0]);
                            contact.last_name = unescape_value(parts[1]);
                        } else {
                            contact.first_name = unescape_value(&value);
                        }
                    }
                }
                "NICKNAME" => {
                    contact.nickname = Some(unescape_value(&value));
                }
                "TEL" => {
                    let phone_type = extract_phone_type(&params);
                    let is_pref = params.contains_key("PREF") || params.get("TYPE").map(|t| t.contains("PREF")).unwrap_or(false);
                    contact.phones.push(PhoneNumber {
                        number: unescape_value(&value),
                        phone_type,
                        is_primary: is_pref,
                    });
                }
                "EMAIL" => {
                    let email_type = extract_email_type(&params);
                    let is_pref = params.contains_key("PREF") || params.get("TYPE").map(|t| t.contains("PREF")).unwrap_or(false);
                    contact.emails.push(EmailAddress {
                        email: unescape_value(&value),
                        email_type,
                        is_primary: is_pref,
                    });
                }
                "ADR" => {
                    // ADR:PO Box;Ext Addr;Street;City;State;Postal Code;Country
                    let parts: Vec<&str> = value.split(';').collect();
                    if parts.len() >= 7 {
                        let address = PostalAddress {
                            street: unescape_value(parts[2]),
                            city: unescape_value(parts[3]),
                            state: unescape_value(parts[4]),
                            postal_code: unescape_value(parts[5]),
                            country: unescape_value(parts[6]),
                            address_type: extract_address_type(&params),
                        };
                        if !address.is_empty() {
                            contact.addresses.push(address);
                        }
                    }
                }
                "ORG" => {
                    // ORG:Company;Department
                    let parts: Vec<&str> = value.split(';').collect();
                    contact.company = Some(unescape_value(parts[0]));
                    if parts.len() > 1 && !parts[1].is_empty() {
                        contact.department = Some(unescape_value(parts[1]));
                    }
                }
                "TITLE" => {
                    contact.job_title = Some(unescape_value(&value));
                }
                "BDAY" => {
                    if let Ok(date) = parse_vcard_date(&value) {
                        contact.birthday = Some(date);
                    }
                }
                "ANNIVERSARY" => {
                    if let Ok(date) = parse_vcard_date(&value) {
                        contact.anniversary = Some(date);
                    }
                }
                "NOTE" => {
                    contact.notes = Some(unescape_value(&value));
                }
                "PHOTO" => {
                    // Handle base64 encoded photo
                    if let Some(encoding) = params.get("ENCODING") {
                        if encoding.to_uppercase() == "B" || encoding.to_uppercase() == "BASE64" {
                            if let Ok(data) = base64::Engine::decode(
                                &base64::engine::general_purpose::STANDARD,
                                &value.replace(' ', ""),
                            ) {
                                contact.avatar_data = Some(data);
                            }
                        }
                    } else if value.starts_with("data:") || value.starts_with("http") {
                        contact.avatar_uri = Some(value);
                    }
                }
                "URL" => {
                    contact.website = Some(unescape_value(&value));
                }
                "CATEGORIES" => {
                    // Groups/tags
                    for group in value.split(',') {
                        contact.groups.push(unescape_value(group.trim()));
                    }
                }
                "UID" => {
                    contact.id = unescape_value(&value);
                }
                _ => {}
            }
        }
    }

    Ok(contact)
}

fn parse_property_line(line: &str) -> Option<(String, HashMap<String, String>, String)> {
    // Find the colon separating property from value
    let colon_pos = line.find(':')?;
    let (prop_part, value) = line.split_at(colon_pos);
    let value = &value[1..]; // Skip the colon

    // Parse property name and parameters
    let mut parts = prop_part.split(';');
    let property = parts.next()?.to_string();

    let mut params = HashMap::new();
    for param in parts {
        if let Some(eq_pos) = param.find('=') {
            let key = param[..eq_pos].to_uppercase();
            let val = param[eq_pos + 1..].to_string();
            params.insert(key, val);
        } else {
            // vCard 2.1 style parameter without =
            params.insert("TYPE".to_string(), param.to_uppercase());
        }
    }

    Some((property, params, value.to_string()))
}

fn extract_phone_type(params: &HashMap<String, String>) -> PhoneType {
    let type_str = params.get("TYPE").map(|s| s.to_uppercase()).unwrap_or_default();

    if type_str.contains("CELL") || type_str.contains("MOBILE") {
        PhoneType::Mobile
    } else if type_str.contains("WORK") && type_str.contains("FAX") {
        PhoneType::WorkFax
    } else if type_str.contains("HOME") && type_str.contains("FAX") {
        PhoneType::HomeFax
    } else if type_str.contains("FAX") {
        PhoneType::WorkFax
    } else if type_str.contains("WORK") {
        PhoneType::Work
    } else if type_str.contains("HOME") {
        PhoneType::Home
    } else if type_str.contains("MAIN") {
        PhoneType::Main
    } else if type_str.contains("PAGER") {
        PhoneType::Pager
    } else {
        PhoneType::Other
    }
}

fn extract_email_type(params: &HashMap<String, String>) -> EmailType {
    let type_str = params.get("TYPE").map(|s| s.to_uppercase()).unwrap_or_default();

    if type_str.contains("WORK") {
        EmailType::Work
    } else if type_str.contains("HOME") || type_str.contains("PERSONAL") {
        EmailType::Personal
    } else {
        EmailType::Other
    }
}

fn extract_address_type(params: &HashMap<String, String>) -> AddressType {
    let type_str = params.get("TYPE").map(|s| s.to_uppercase()).unwrap_or_default();

    if type_str.contains("WORK") {
        AddressType::Work
    } else if type_str.contains("HOME") {
        AddressType::Home
    } else {
        AddressType::Other
    }
}

fn parse_vcard_date(value: &str) -> Result<NaiveDate> {
    // Try various date formats
    let formats = [
        "%Y-%m-%d",
        "%Y%m%d",
        "%Y-%m",
        "%Y",
    ];

    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(value, fmt) {
            return Ok(date);
        }
    }

    // Try with dashes removed
    let clean = value.replace('-', "");
    if clean.len() >= 8 {
        if let Ok(date) = NaiveDate::parse_from_str(&clean[..8], "%Y%m%d") {
            return Ok(date);
        }
    }

    Err(anyhow!("Unable to parse date: {}", value))
}

fn unescape_value(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\N", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

fn escape_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace(',', "\\,")
        .replace(';', "\\;")
}

/// Generate vCard 3.0 format from a contact
pub fn generate_vcard(contact: &Contact) -> String {
    let mut lines = Vec::new();

    lines.push("BEGIN:VCARD".to_string());
    lines.push("VERSION:3.0".to_string());

    // UID
    lines.push(format!("UID:{}", contact.id));

    // Name
    lines.push(format!(
        "N:{};{};;;",
        escape_value(&contact.last_name),
        escape_value(&contact.first_name)
    ));
    lines.push(format!("FN:{}", escape_value(&contact.display_name())));

    // Nickname
    if let Some(nickname) = &contact.nickname {
        lines.push(format!("NICKNAME:{}", escape_value(nickname)));
    }

    // Phones
    for phone in &contact.phones {
        let type_param = match phone.phone_type {
            PhoneType::Mobile => "CELL",
            PhoneType::Home => "HOME,VOICE",
            PhoneType::Work => "WORK,VOICE",
            PhoneType::Main => "MAIN",
            PhoneType::HomeFax => "HOME,FAX",
            PhoneType::WorkFax => "WORK,FAX",
            PhoneType::Pager => "PAGER",
            PhoneType::Other => "OTHER",
        };
        let pref = if phone.is_primary { ",PREF" } else { "" };
        lines.push(format!("TEL;TYPE={}{}:{}", type_param, pref, phone.number));
    }

    // Emails
    for email in &contact.emails {
        let type_param = match email.email_type {
            EmailType::Personal => "HOME",
            EmailType::Work => "WORK",
            EmailType::Other => "OTHER",
        };
        let pref = if email.is_primary { ",PREF" } else { "" };
        lines.push(format!("EMAIL;TYPE={}{}:{}", type_param, pref, email.email));
    }

    // Addresses
    for address in &contact.addresses {
        let type_param = match address.address_type {
            AddressType::Home => "HOME",
            AddressType::Work => "WORK",
            AddressType::Other => "OTHER",
        };
        lines.push(format!(
            "ADR;TYPE={}:;;{};{};{};{};{}",
            type_param,
            escape_value(&address.street),
            escape_value(&address.city),
            escape_value(&address.state),
            escape_value(&address.postal_code),
            escape_value(&address.country)
        ));
    }

    // Organization
    if let Some(company) = &contact.company {
        let dept = contact.department.as_deref().unwrap_or("");
        lines.push(format!("ORG:{};{}", escape_value(company), escape_value(dept)));
    }

    // Title
    if let Some(title) = &contact.job_title {
        lines.push(format!("TITLE:{}", escape_value(title)));
    }

    // Birthday
    if let Some(birthday) = &contact.birthday {
        lines.push(format!("BDAY:{}", birthday.format("%Y-%m-%d")));
    }

    // Anniversary
    if let Some(anniversary) = &contact.anniversary {
        lines.push(format!("ANNIVERSARY:{}", anniversary.format("%Y-%m-%d")));
    }

    // Notes
    if let Some(notes) = &contact.notes {
        lines.push(format!("NOTE:{}", escape_value(notes)));
    }

    // Website
    if let Some(website) = &contact.website {
        lines.push(format!("URL:{}", website));
    }

    // Photo (base64)
    if let Some(data) = &contact.avatar_data {
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        lines.push(format!("PHOTO;ENCODING=b;TYPE=JPEG:{}", encoded));
    } else if let Some(uri) = &contact.avatar_uri {
        lines.push(format!("PHOTO:{}", uri));
    }

    // Categories/Groups
    if !contact.groups.is_empty() {
        let groups: Vec<String> = contact.groups.iter().map(|g| escape_value(g)).collect();
        lines.push(format!("CATEGORIES:{}", groups.join(",")));
    }

    // Timestamps
    lines.push(format!("REV:{}", contact.updated_at.format("%Y%m%dT%H%M%SZ")));

    lines.push("END:VCARD".to_string());

    lines.join("\r\n")
}

/// Generate vCard for multiple contacts
pub fn generate_vcards(contacts: &[Contact]) -> String {
    contacts
        .iter()
        .map(generate_vcard)
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// Export contacts to CSV format
pub fn export_to_csv(contacts: &[Contact]) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(Vec::new());

    // Header
    wtr.write_record(&[
        "First Name",
        "Last Name",
        "Nickname",
        "Phone 1",
        "Phone 1 Type",
        "Phone 2",
        "Phone 2 Type",
        "Email 1",
        "Email 1 Type",
        "Email 2",
        "Email 2 Type",
        "Street",
        "City",
        "State",
        "Postal Code",
        "Country",
        "Company",
        "Job Title",
        "Birthday",
        "Notes",
    ])?;

    for contact in contacts {
        let phone1 = contact.phones.get(0);
        let phone2 = contact.phones.get(1);
        let email1 = contact.emails.get(0);
        let email2 = contact.emails.get(1);
        let addr = contact.addresses.first();

        wtr.write_record(&[
            &contact.first_name,
            &contact.last_name,
            contact.nickname.as_deref().unwrap_or(""),
            phone1.map(|p| p.number.as_str()).unwrap_or(""),
            phone1.map(|p| p.phone_type.as_str()).unwrap_or(""),
            phone2.map(|p| p.number.as_str()).unwrap_or(""),
            phone2.map(|p| p.phone_type.as_str()).unwrap_or(""),
            email1.map(|e| e.email.as_str()).unwrap_or(""),
            email1.map(|e| e.email_type.as_str()).unwrap_or(""),
            email2.map(|e| e.email.as_str()).unwrap_or(""),
            email2.map(|e| e.email_type.as_str()).unwrap_or(""),
            addr.map(|a| a.street.as_str()).unwrap_or(""),
            addr.map(|a| a.city.as_str()).unwrap_or(""),
            addr.map(|a| a.state.as_str()).unwrap_or(""),
            addr.map(|a| a.postal_code.as_str()).unwrap_or(""),
            addr.map(|a| a.country.as_str()).unwrap_or(""),
            contact.company.as_deref().unwrap_or(""),
            contact.job_title.as_deref().unwrap_or(""),
            &contact.birthday.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            contact.notes.as_deref().unwrap_or(""),
        ])?;
    }

    let data = wtr.into_inner()?;
    Ok(String::from_utf8(data)?)
}

/// Import contacts from CSV
pub fn import_from_csv(data: &str) -> Result<Vec<Contact>> {
    let mut rdr = csv::Reader::from_reader(data.as_bytes());
    let mut contacts = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let mut contact = Contact::new();

        if let Some(val) = record.get(0) {
            contact.first_name = val.to_string();
        }
        if let Some(val) = record.get(1) {
            contact.last_name = val.to_string();
        }
        if let Some(val) = record.get(2) {
            if !val.is_empty() {
                contact.nickname = Some(val.to_string());
            }
        }

        // Phone 1
        if let Some(val) = record.get(3) {
            if !val.is_empty() {
                let phone_type = record.get(4)
                    .map(parse_phone_type_from_str)
                    .unwrap_or(PhoneType::Mobile);
                contact.phones.push(PhoneNumber {
                    number: val.to_string(),
                    phone_type,
                    is_primary: true,
                });
            }
        }

        // Phone 2
        if let Some(val) = record.get(5) {
            if !val.is_empty() {
                let phone_type = record.get(6)
                    .map(parse_phone_type_from_str)
                    .unwrap_or(PhoneType::Other);
                contact.phones.push(PhoneNumber {
                    number: val.to_string(),
                    phone_type,
                    is_primary: false,
                });
            }
        }

        // Email 1
        if let Some(val) = record.get(7) {
            if !val.is_empty() {
                let email_type = record.get(8)
                    .map(parse_email_type_from_str)
                    .unwrap_or(EmailType::Personal);
                contact.emails.push(EmailAddress {
                    email: val.to_string(),
                    email_type,
                    is_primary: true,
                });
            }
        }

        // Email 2
        if let Some(val) = record.get(9) {
            if !val.is_empty() {
                let email_type = record.get(10)
                    .map(parse_email_type_from_str)
                    .unwrap_or(EmailType::Other);
                contact.emails.push(EmailAddress {
                    email: val.to_string(),
                    email_type,
                    is_primary: false,
                });
            }
        }

        // Address
        let street = record.get(11).unwrap_or("").to_string();
        let city = record.get(12).unwrap_or("").to_string();
        let state = record.get(13).unwrap_or("").to_string();
        let postal_code = record.get(14).unwrap_or("").to_string();
        let country = record.get(15).unwrap_or("").to_string();

        if !street.is_empty() || !city.is_empty() {
            contact.addresses.push(PostalAddress {
                street,
                city,
                state,
                postal_code,
                country,
                address_type: AddressType::Home,
            });
        }

        // Company & Title
        if let Some(val) = record.get(16) {
            if !val.is_empty() {
                contact.company = Some(val.to_string());
            }
        }
        if let Some(val) = record.get(17) {
            if !val.is_empty() {
                contact.job_title = Some(val.to_string());
            }
        }

        // Birthday
        if let Some(val) = record.get(18) {
            if let Ok(date) = NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                contact.birthday = Some(date);
            }
        }

        // Notes
        if let Some(val) = record.get(19) {
            if !val.is_empty() {
                contact.notes = Some(val.to_string());
            }
        }

        contacts.push(contact);
    }

    Ok(contacts)
}

fn parse_phone_type_from_str(s: &str) -> PhoneType {
    match s.to_lowercase().as_str() {
        "mobile" | "cell" => PhoneType::Mobile,
        "home" => PhoneType::Home,
        "work" => PhoneType::Work,
        "main" => PhoneType::Main,
        "fax" | "work fax" => PhoneType::WorkFax,
        "home fax" => PhoneType::HomeFax,
        "pager" => PhoneType::Pager,
        _ => PhoneType::Other,
    }
}

fn parse_email_type_from_str(s: &str) -> EmailType {
    match s.to_lowercase().as_str() {
        "personal" | "home" => EmailType::Personal,
        "work" => EmailType::Work,
        _ => EmailType::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vcard() {
        let vcard = r#"BEGIN:VCARD
VERSION:3.0
N:Doe;John;;;
FN:John Doe
TEL;TYPE=CELL:+1-555-1234
EMAIL;TYPE=WORK:john@example.com
END:VCARD"#;

        let contacts = parse_vcard(vcard).unwrap();
        assert_eq!(contacts.len(), 1);

        let contact = &contacts[0];
        assert_eq!(contact.first_name, "John");
        assert_eq!(contact.last_name, "Doe");
        assert_eq!(contact.phones.len(), 1);
        assert_eq!(contact.phones[0].number, "+1-555-1234");
    }

    #[test]
    fn test_generate_vcard() {
        let mut contact = Contact::new();
        contact.first_name = "Jane".to_string();
        contact.last_name = "Smith".to_string();
        contact.phones.push(PhoneNumber {
            number: "+1-555-5678".to_string(),
            phone_type: PhoneType::Mobile,
            is_primary: true,
        });

        let vcard = generate_vcard(&contact);
        assert!(vcard.contains("BEGIN:VCARD"));
        assert!(vcard.contains("N:Smith;Jane;;;"));
        assert!(vcard.contains("TEL;TYPE=CELL,PREF:+1-555-5678"));
    }
}
