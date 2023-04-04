use std::mem;

use crypto::scrypt::{scrypt_simple, ScryptParams};
use regex::Regex;

pub trait Email {
    fn get_email(&mut self) -> &mut String;
}
pub trait SanitizeEmail {
    fn fmt_email(&mut self);
    fn sanitize_email(&mut self) -> Result<(), &str>;
}
impl<T: Email> SanitizeEmail for T {
    fn fmt_email(&mut self) {
        let mut email = self.get_email().to_ascii_lowercase().trim().to_string();
        mem::swap(&mut email, self.get_email());
    }
    fn sanitize_email(&mut self) -> Result<(), &str> {
        self.fmt_email();
        let email = self.get_email();
        let re = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,253}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,253}[a-zA-Z0-9])?)+$").unwrap();
        if !re.is_match(email)
        {
            return Err("Mail invalida");
        }

        Ok(())
    }
}

pub trait Password {
    fn get_password(&mut self) -> &mut String;
}
pub trait SanitizePassword {
    fn fmt_password(&mut self);
    fn sanitize_password(&mut self) -> Result<(), &str>;
    fn hash_password(&mut self);
}
impl<T: Password> SanitizePassword for T {
    fn fmt_password(&mut self) {
        let mut new_password = self.get_password().trim().to_string();
        mem::swap(self.get_password(), &mut new_password);
    }
    fn hash_password(&mut self) {
        let params = ScryptParams::new(11, 8, 1);
        let mut hashed = scrypt_simple(self.get_password(), &params).unwrap();
        mem::swap(self.get_password(), &mut hashed);
    }
    fn sanitize_password(&mut self) -> Result<(), &str> {
        self.fmt_password();
        let password = self.get_password();
        if password.len() < 8 {
            return Err("Password troppo breve, deve essere lunga almeno 8 caratteri");
        }

        if !password.chars().any(|x| x.is_ascii_uppercase()) {
            return Err("La password deve contenere almeno una maiuscola");
        }

        if !password.chars().any(|x| x.is_ascii_lowercase()) {
            return Err("La password deve contenere almeno una minuscola");
        }

        if !password.chars().any(|x| x.is_numeric()) {
            return Err("La password deve contenere almeno un numero");
        }

        if !password.chars().any(|x| !x.is_ascii_alphanumeric()) {
            return Err("La password deve contenere almeno un carattere speciale");
        }
        self.hash_password();
        Ok(())
    }
}
pub trait Name {
    fn get_name(&mut self) -> &mut String;
}
pub trait SanitizeName {
    fn fmt_name(&mut self);
    fn sanitize_name(&mut self) -> Result<(), &str>;
}
impl<T: Name> SanitizeName for T {
    fn fmt_name(&mut self) {
        let mut name: String = self.get_name().trim().to_string();
        mem::swap(&mut name, self.get_name());
    }
    fn sanitize_name(&mut self) -> Result<(), &str> {
        self.fmt_name();
        let name = self.get_name().to_string();
        let name_len = name.len();
        if name_len < 3 && 20 < name_len {
            return Err("Nome utente invalido. Deve essere lungo tra 3 e 20 caratteri (e possibilmente simile al nome)");
        }
        if !name
            .chars()
            .all(|x| x.is_alphanumeric() || x == '_' || x == ' ')
        {
            return Err(
                "Nome utente invalido. Un nome corretto può contenere lettere, numeri, spazi e _",
            );
        }
        Ok(())
    }
}
