// Contact Storage - SQLite persistence layer

use crate::data::contact::*;
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

pub struct ContactStorage {
    conn: Connection,
}

impl ContactStorage {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .context("Could not find data directory")?
            .join("winux-contacts");
        Ok(data_dir.join("contacts.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Contacts table
            CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                first_name TEXT NOT NULL DEFAULT '',
                last_name TEXT NOT NULL DEFAULT '',
                nickname TEXT,
                company TEXT,
                job_title TEXT,
                department TEXT,
                birthday TEXT,
                anniversary TEXT,
                notes TEXT,
                avatar_data BLOB,
                avatar_uri TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                website TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                etag TEXT,
                carddav_href TEXT
            );

            -- Phone numbers table
            CREATE TABLE IF NOT EXISTS phones (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                contact_id TEXT NOT NULL,
                number TEXT NOT NULL,
                phone_type TEXT NOT NULL,
                is_primary INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
            );

            -- Email addresses table
            CREATE TABLE IF NOT EXISTS emails (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                contact_id TEXT NOT NULL,
                email TEXT NOT NULL,
                email_type TEXT NOT NULL,
                is_primary INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
            );

            -- Postal addresses table
            CREATE TABLE IF NOT EXISTS addresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                contact_id TEXT NOT NULL,
                street TEXT NOT NULL DEFAULT '',
                city TEXT NOT NULL DEFAULT '',
                state TEXT NOT NULL DEFAULT '',
                postal_code TEXT NOT NULL DEFAULT '',
                country TEXT NOT NULL DEFAULT '',
                address_type TEXT NOT NULL,
                FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
            );

            -- Groups table
            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT
            );

            -- Contact-Group relationship
            CREATE TABLE IF NOT EXISTS contact_groups (
                contact_id TEXT NOT NULL,
                group_id TEXT NOT NULL,
                PRIMARY KEY (contact_id, group_id),
                FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
                FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
            );

            -- Social profiles table
            CREATE TABLE IF NOT EXISTS social_profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                contact_id TEXT NOT NULL,
                platform TEXT NOT NULL,
                username TEXT NOT NULL,
                url TEXT,
                FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
            );

            -- CardDAV accounts table
            CREATE TABLE IF NOT EXISTS carddav_accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                username TEXT NOT NULL,
                password TEXT,
                sync_token TEXT,
                last_sync TEXT
            );

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_contacts_name ON contacts(first_name, last_name);
            CREATE INDEX IF NOT EXISTS idx_contacts_favorite ON contacts(is_favorite);
            CREATE INDEX IF NOT EXISTS idx_phones_contact ON phones(contact_id);
            CREATE INDEX IF NOT EXISTS idx_emails_contact ON emails(contact_id);
            CREATE INDEX IF NOT EXISTS idx_addresses_contact ON addresses(contact_id);

            -- Enable foreign keys
            PRAGMA foreign_keys = ON;
            "#,
        )?;

        Ok(())
    }

    pub fn save_contact(&mut self, contact: &Contact) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Insert or update main contact
        tx.execute(
            r#"
            INSERT OR REPLACE INTO contacts (
                id, first_name, last_name, nickname, company, job_title, department,
                birthday, anniversary, notes, avatar_data, avatar_uri, is_favorite,
                website, created_at, updated_at, etag, carddav_href
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            "#,
            params![
                contact.id,
                contact.first_name,
                contact.last_name,
                contact.nickname,
                contact.company,
                contact.job_title,
                contact.department,
                contact.birthday.map(|d| d.to_string()),
                contact.anniversary.map(|d| d.to_string()),
                contact.notes,
                contact.avatar_data,
                contact.avatar_uri,
                contact.is_favorite as i32,
                contact.website,
                contact.created_at.to_rfc3339(),
                contact.updated_at.to_rfc3339(),
                contact.etag,
                contact.carddav_href,
            ],
        )?;

        // Clear and re-insert phones
        tx.execute("DELETE FROM phones WHERE contact_id = ?1", params![contact.id])?;
        for phone in &contact.phones {
            tx.execute(
                "INSERT INTO phones (contact_id, number, phone_type, is_primary) VALUES (?1, ?2, ?3, ?4)",
                params![
                    contact.id,
                    phone.number,
                    format!("{:?}", phone.phone_type),
                    phone.is_primary as i32,
                ],
            )?;
        }

        // Clear and re-insert emails
        tx.execute("DELETE FROM emails WHERE contact_id = ?1", params![contact.id])?;
        for email in &contact.emails {
            tx.execute(
                "INSERT INTO emails (contact_id, email, email_type, is_primary) VALUES (?1, ?2, ?3, ?4)",
                params![
                    contact.id,
                    email.email,
                    format!("{:?}", email.email_type),
                    email.is_primary as i32,
                ],
            )?;
        }

        // Clear and re-insert addresses
        tx.execute("DELETE FROM addresses WHERE contact_id = ?1", params![contact.id])?;
        for address in &contact.addresses {
            tx.execute(
                r#"INSERT INTO addresses (contact_id, street, city, state, postal_code, country, address_type)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
                params![
                    contact.id,
                    address.street,
                    address.city,
                    address.state,
                    address.postal_code,
                    address.country,
                    format!("{:?}", address.address_type),
                ],
            )?;
        }

        // Clear and re-insert group relationships
        tx.execute("DELETE FROM contact_groups WHERE contact_id = ?1", params![contact.id])?;
        for group_id in &contact.groups {
            tx.execute(
                "INSERT OR IGNORE INTO contact_groups (contact_id, group_id) VALUES (?1, ?2)",
                params![contact.id, group_id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_contact(&self, id: &str) -> Result<Option<Contact>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, first_name, last_name, nickname, company, job_title, department,
                      birthday, anniversary, notes, avatar_data, avatar_uri, is_favorite,
                      website, created_at, updated_at, etag, carddav_href
               FROM contacts WHERE id = ?1"#,
        )?;

        let contact = stmt
            .query_row(params![id], |row| {
                Ok(self.row_to_contact(row)?)
            })
            .optional()?;

        if let Some(mut contact) = contact {
            self.load_contact_relations(&mut contact)?;
            Ok(Some(contact))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_contacts(&self) -> Result<Vec<Contact>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, first_name, last_name, nickname, company, job_title, department,
                      birthday, anniversary, notes, avatar_data, avatar_uri, is_favorite,
                      website, created_at, updated_at, etag, carddav_href
               FROM contacts ORDER BY first_name, last_name"#,
        )?;

        let contacts: Vec<Contact> = stmt
            .query_map([], |row| Ok(self.row_to_contact(row)?))?
            .filter_map(|r| r.ok())
            .collect();

        let mut result = Vec::new();
        for mut contact in contacts {
            self.load_contact_relations(&mut contact)?;
            result.push(contact);
        }

        Ok(result)
    }

    pub fn get_favorites(&self) -> Result<Vec<Contact>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, first_name, last_name, nickname, company, job_title, department,
                      birthday, anniversary, notes, avatar_data, avatar_uri, is_favorite,
                      website, created_at, updated_at, etag, carddav_href
               FROM contacts WHERE is_favorite = 1 ORDER BY first_name, last_name"#,
        )?;

        let contacts: Vec<Contact> = stmt
            .query_map([], |row| Ok(self.row_to_contact(row)?))?
            .filter_map(|r| r.ok())
            .collect();

        let mut result = Vec::new();
        for mut contact in contacts {
            self.load_contact_relations(&mut contact)?;
            result.push(contact);
        }

        Ok(result)
    }

    pub fn search_contacts(&self, query: &str) -> Result<Vec<Contact>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            r#"SELECT DISTINCT c.id, c.first_name, c.last_name, c.nickname, c.company,
                      c.job_title, c.department, c.birthday, c.anniversary, c.notes,
                      c.avatar_data, c.avatar_uri, c.is_favorite, c.website,
                      c.created_at, c.updated_at, c.etag, c.carddav_href
               FROM contacts c
               LEFT JOIN phones p ON c.id = p.contact_id
               LEFT JOIN emails e ON c.id = e.contact_id
               WHERE c.first_name LIKE ?1 OR c.last_name LIKE ?1
                  OR c.nickname LIKE ?1 OR c.company LIKE ?1
                  OR p.number LIKE ?1 OR e.email LIKE ?1
               ORDER BY c.first_name, c.last_name"#,
        )?;

        let contacts: Vec<Contact> = stmt
            .query_map(params![pattern], |row| Ok(self.row_to_contact(row)?))?
            .filter_map(|r| r.ok())
            .collect();

        let mut result = Vec::new();
        for mut contact in contacts {
            self.load_contact_relations(&mut contact)?;
            result.push(contact);
        }

        Ok(result)
    }

    pub fn delete_contact(&mut self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM contacts WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn toggle_favorite(&mut self, id: &str) -> Result<bool> {
        self.conn.execute(
            "UPDATE contacts SET is_favorite = NOT is_favorite, updated_at = ?2 WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()],
        )?;

        let is_fav: i32 = self.conn.query_row(
            "SELECT is_favorite FROM contacts WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(is_fav == 1)
    }

    // Group operations
    pub fn create_group(&mut self, group: &ContactGroup) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO groups (id, name, color) VALUES (?1, ?2, ?3)",
            params![group.id, group.name, group.color],
        )?;
        Ok(())
    }

    pub fn get_all_groups(&self) -> Result<Vec<ContactGroup>> {
        let mut stmt = self.conn.prepare("SELECT id, name, color FROM groups ORDER BY name")?;
        let groups = stmt
            .query_map([], |row| {
                Ok(ContactGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(groups)
    }

    pub fn delete_group(&mut self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM groups WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_contacts_in_group(&self, group_id: &str) -> Result<Vec<Contact>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT c.id, c.first_name, c.last_name, c.nickname, c.company, c.job_title,
                      c.department, c.birthday, c.anniversary, c.notes, c.avatar_data,
                      c.avatar_uri, c.is_favorite, c.website, c.created_at, c.updated_at,
                      c.etag, c.carddav_href
               FROM contacts c
               INNER JOIN contact_groups cg ON c.id = cg.contact_id
               WHERE cg.group_id = ?1
               ORDER BY c.first_name, c.last_name"#,
        )?;

        let contacts: Vec<Contact> = stmt
            .query_map(params![group_id], |row| Ok(self.row_to_contact(row)?))?
            .filter_map(|r| r.ok())
            .collect();

        let mut result = Vec::new();
        for mut contact in contacts {
            self.load_contact_relations(&mut contact)?;
            result.push(contact);
        }

        Ok(result)
    }

    // Helper methods
    fn row_to_contact(&self, row: &rusqlite::Row) -> rusqlite::Result<Contact> {
        let birthday_str: Option<String> = row.get(7)?;
        let anniversary_str: Option<String> = row.get(8)?;
        let created_str: String = row.get(14)?;
        let updated_str: String = row.get(15)?;

        Ok(Contact {
            id: row.get(0)?,
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            nickname: row.get(3)?,
            company: row.get(4)?,
            job_title: row.get(5)?,
            department: row.get(6)?,
            birthday: birthday_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            anniversary: anniversary_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            notes: row.get(9)?,
            avatar_data: row.get(10)?,
            avatar_uri: row.get(11)?,
            is_favorite: row.get::<_, i32>(12)? == 1,
            website: row.get(13)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            etag: row.get(16)?,
            carddav_href: row.get(17)?,
            phones: Vec::new(),
            emails: Vec::new(),
            addresses: Vec::new(),
            groups: Vec::new(),
            social_profiles: Vec::new(),
        })
    }

    fn load_contact_relations(&self, contact: &mut Contact) -> Result<()> {
        // Load phones
        let mut stmt = self.conn.prepare(
            "SELECT number, phone_type, is_primary FROM phones WHERE contact_id = ?1",
        )?;
        contact.phones = stmt
            .query_map(params![contact.id], |row| {
                let type_str: String = row.get(1)?;
                Ok(PhoneNumber {
                    number: row.get(0)?,
                    phone_type: parse_phone_type(&type_str),
                    is_primary: row.get::<_, i32>(2)? == 1,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Load emails
        let mut stmt = self.conn.prepare(
            "SELECT email, email_type, is_primary FROM emails WHERE contact_id = ?1",
        )?;
        contact.emails = stmt
            .query_map(params![contact.id], |row| {
                let type_str: String = row.get(1)?;
                Ok(EmailAddress {
                    email: row.get(0)?,
                    email_type: parse_email_type(&type_str),
                    is_primary: row.get::<_, i32>(2)? == 1,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Load addresses
        let mut stmt = self.conn.prepare(
            "SELECT street, city, state, postal_code, country, address_type FROM addresses WHERE contact_id = ?1",
        )?;
        contact.addresses = stmt
            .query_map(params![contact.id], |row| {
                let type_str: String = row.get(5)?;
                Ok(PostalAddress {
                    street: row.get(0)?,
                    city: row.get(1)?,
                    state: row.get(2)?,
                    postal_code: row.get(3)?,
                    country: row.get(4)?,
                    address_type: parse_address_type(&type_str),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Load groups
        let mut stmt = self.conn.prepare(
            "SELECT group_id FROM contact_groups WHERE contact_id = ?1",
        )?;
        contact.groups = stmt
            .query_map(params![contact.id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(())
    }

    pub fn get_contact_count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM contacts",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

fn parse_phone_type(s: &str) -> PhoneType {
    match s {
        "Mobile" => PhoneType::Mobile,
        "Home" => PhoneType::Home,
        "Work" => PhoneType::Work,
        "Main" => PhoneType::Main,
        "HomeFax" => PhoneType::HomeFax,
        "WorkFax" => PhoneType::WorkFax,
        "Pager" => PhoneType::Pager,
        _ => PhoneType::Other,
    }
}

fn parse_email_type(s: &str) -> EmailType {
    match s {
        "Personal" => EmailType::Personal,
        "Work" => EmailType::Work,
        _ => EmailType::Other,
    }
}

fn parse_address_type(s: &str) -> AddressType {
    match s {
        "Home" => AddressType::Home,
        "Work" => AddressType::Work,
        _ => AddressType::Other,
    }
}
