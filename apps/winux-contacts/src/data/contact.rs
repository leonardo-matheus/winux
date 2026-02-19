// Contact data structure

use chrono::{NaiveDate, Utc, DateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Phone number with type label
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhoneNumber {
    pub number: String,
    pub phone_type: PhoneType,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PhoneType {
    #[default]
    Mobile,
    Home,
    Work,
    Main,
    HomeFax,
    WorkFax,
    Pager,
    Other,
}

impl PhoneType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PhoneType::Mobile => "Mobile",
            PhoneType::Home => "Home",
            PhoneType::Work => "Work",
            PhoneType::Main => "Main",
            PhoneType::HomeFax => "Home Fax",
            PhoneType::WorkFax => "Work Fax",
            PhoneType::Pager => "Pager",
            PhoneType::Other => "Other",
        }
    }

    pub fn all() -> Vec<PhoneType> {
        vec![
            PhoneType::Mobile,
            PhoneType::Home,
            PhoneType::Work,
            PhoneType::Main,
            PhoneType::HomeFax,
            PhoneType::WorkFax,
            PhoneType::Pager,
            PhoneType::Other,
        ]
    }
}

/// Email address with type label
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailAddress {
    pub email: String,
    pub email_type: EmailType,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EmailType {
    #[default]
    Personal,
    Work,
    Other,
}

impl EmailType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailType::Personal => "Personal",
            EmailType::Work => "Work",
            EmailType::Other => "Other",
        }
    }

    pub fn all() -> Vec<EmailType> {
        vec![EmailType::Personal, EmailType::Work, EmailType::Other]
    }
}

/// Postal address
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PostalAddress {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub address_type: AddressType,
}

impl PostalAddress {
    pub fn is_empty(&self) -> bool {
        self.street.is_empty()
            && self.city.is_empty()
            && self.state.is_empty()
            && self.postal_code.is_empty()
            && self.country.is_empty()
    }

    pub fn formatted(&self) -> String {
        let mut parts = Vec::new();
        if !self.street.is_empty() {
            parts.push(self.street.clone());
        }
        let mut city_line = Vec::new();
        if !self.city.is_empty() {
            city_line.push(self.city.clone());
        }
        if !self.state.is_empty() {
            city_line.push(self.state.clone());
        }
        if !self.postal_code.is_empty() {
            city_line.push(self.postal_code.clone());
        }
        if !city_line.is_empty() {
            parts.push(city_line.join(", "));
        }
        if !self.country.is_empty() {
            parts.push(self.country.clone());
        }
        parts.join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum AddressType {
    #[default]
    Home,
    Work,
    Other,
}

impl AddressType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AddressType::Home => "Home",
            AddressType::Work => "Work",
            AddressType::Other => "Other",
        }
    }

    pub fn all() -> Vec<AddressType> {
        vec![AddressType::Home, AddressType::Work, AddressType::Other]
    }
}

/// Social media profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialProfile {
    pub platform: SocialPlatform,
    pub username: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SocialPlatform {
    Twitter,
    Facebook,
    LinkedIn,
    Instagram,
    GitHub,
    Website,
    Other(String),
}

impl SocialPlatform {
    pub fn as_str(&self) -> &str {
        match self {
            SocialPlatform::Twitter => "Twitter",
            SocialPlatform::Facebook => "Facebook",
            SocialPlatform::LinkedIn => "LinkedIn",
            SocialPlatform::Instagram => "Instagram",
            SocialPlatform::GitHub => "GitHub",
            SocialPlatform::Website => "Website",
            SocialPlatform::Other(name) => name,
        }
    }
}

/// Contact group/label
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactGroup {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

impl ContactGroup {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            color: None,
        }
    }
}

/// Main contact structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>,
    pub phones: Vec<PhoneNumber>,
    pub emails: Vec<EmailAddress>,
    pub addresses: Vec<PostalAddress>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub anniversary: Option<NaiveDate>,
    pub notes: Option<String>,
    pub avatar_data: Option<Vec<u8>>,
    pub avatar_uri: Option<String>,
    pub groups: Vec<String>,
    pub is_favorite: bool,
    pub social_profiles: Vec<SocialProfile>,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub etag: Option<String>,
    pub carddav_href: Option<String>,
}

impl Contact {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            first_name: String::new(),
            last_name: String::new(),
            nickname: None,
            phones: Vec::new(),
            emails: Vec::new(),
            addresses: Vec::new(),
            company: None,
            job_title: None,
            department: None,
            birthday: None,
            anniversary: None,
            notes: None,
            avatar_data: None,
            avatar_uri: None,
            groups: Vec::new(),
            is_favorite: false,
            social_profiles: Vec::new(),
            website: None,
            created_at: now,
            updated_at: now,
            etag: None,
            carddav_href: None,
        }
    }

    pub fn display_name(&self) -> String {
        let full_name = format!("{} {}", self.first_name, self.last_name).trim().to_string();
        if full_name.is_empty() {
            if let Some(nickname) = &self.nickname {
                return nickname.clone();
            }
            if let Some(email) = self.emails.first() {
                return email.email.clone();
            }
            if let Some(phone) = self.phones.first() {
                return phone.number.clone();
            }
            "Unknown".to_string()
        } else {
            full_name
        }
    }

    pub fn initials(&self) -> String {
        let first = self.first_name.chars().next().unwrap_or_default();
        let last = self.last_name.chars().next().unwrap_or_default();

        if first != '\0' && last != '\0' {
            format!("{}{}", first.to_uppercase(), last.to_uppercase())
        } else if first != '\0' {
            first.to_uppercase().to_string()
        } else if last != '\0' {
            last.to_uppercase().to_string()
        } else if let Some(nickname) = &self.nickname {
            nickname.chars().next().unwrap_or('?').to_uppercase().to_string()
        } else {
            "?".to_string()
        }
    }

    pub fn primary_phone(&self) -> Option<&PhoneNumber> {
        self.phones.iter().find(|p| p.is_primary).or(self.phones.first())
    }

    pub fn primary_email(&self) -> Option<&EmailAddress> {
        self.emails.iter().find(|e| e.is_primary).or(self.emails.first())
    }

    pub fn primary_address(&self) -> Option<&PostalAddress> {
        self.addresses.first()
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        let check = |s: &str| s.to_lowercase().contains(&query);

        check(&self.first_name)
            || check(&self.last_name)
            || self.nickname.as_ref().map(|n| check(n)).unwrap_or(false)
            || self.company.as_ref().map(|c| check(c)).unwrap_or(false)
            || self.phones.iter().any(|p| check(&p.number))
            || self.emails.iter().any(|e| check(&e.email))
    }

    pub fn sort_key_first_name(&self) -> String {
        format!(
            "{} {}",
            self.first_name.to_lowercase(),
            self.last_name.to_lowercase()
        )
    }

    pub fn sort_key_last_name(&self) -> String {
        format!(
            "{} {}",
            self.last_name.to_lowercase(),
            self.first_name.to_lowercase()
        )
    }
}

impl Default for Contact {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        let mut contact = Contact::new();
        contact.first_name = "John".to_string();
        contact.last_name = "Doe".to_string();
        assert_eq!(contact.display_name(), "John Doe");
    }

    #[test]
    fn test_initials() {
        let mut contact = Contact::new();
        contact.first_name = "John".to_string();
        contact.last_name = "Doe".to_string();
        assert_eq!(contact.initials(), "JD");
    }

    #[test]
    fn test_search() {
        let mut contact = Contact::new();
        contact.first_name = "John".to_string();
        contact.last_name = "Doe".to_string();
        contact.phones.push(PhoneNumber {
            number: "555-1234".to_string(),
            phone_type: PhoneType::Mobile,
            is_primary: true,
        });

        assert!(contact.matches_search("john"));
        assert!(contact.matches_search("doe"));
        assert!(contact.matches_search("555"));
        assert!(!contact.matches_search("jane"));
    }
}
