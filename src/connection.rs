//! start a connection to churchtools convention for this module:
//! "_function_name" means a function contain parameters that
//! are only meant to be used with default values that are only
//! asked for to make it easy to test them. Don't ever use "_function_name"
//! functions, use the corresponding "function_name"
//! 
#![warn(missing_docs)]
#![allow(unused)]
use std::fs;
use std::io::Read;
use std::fmt::Display;

use check_if_email_exists::{check_email, CheckEmailInput, Reachable};
use fltk::{app::App, window::Window, input::Input, button::Button, prelude::*};
use reqwest::{blocking::Client, Error as ReqwestError};
use rpassword::read_password;
use toml::{value::Map, Value};

/// The number of tries the credentials are asked before the
/// program stops. If None, then the program only stops through Ctrl+c
const NR_TYPE_TRYS: Option<u8> = Some(3u8);
/// if the credentials including the password should be saved into a
/// file. Suggestion: don't turn this on
const SAVE_PASSWORD_TO_FILE: bool = false;

/// save memory management for passwords
/// safely deletes a password when it goes out
/// of scope (TODO: how to write a test for that? probably possible with unsafe)
/// just don't ever clone passwords that are just a String or &str

#[derive(Clone)]
pub struct Password(String);

impl Password {
    /// this function gets a "pad" that is more or less not important
    /// (it's the character that you overwrite with) just needed for test purpose
    /// 
    /// panics if pad is not a valid ascii character
    fn _clear_str(string: &mut str, pad: u8) {
        let string_bytes;
        // panics if pad is not safe
        char::from_u32(pad as u32).unwrap();
        // safety: "65" is valid utf-8 code
        unsafe {
            string_bytes = string.as_bytes_mut();
        }
        // the last index of password is string_bytes.len() - 1
        // and that is the last number of 0..string_bytes.len()
        // => - [ ] never panics
        //    - every character is deleted
        for i in 0..string_bytes.len() {
            string_bytes[i] = pad;
        }
    }
    /// clear a mutable &str so it won't contain a readable password
    /// unfortunately, there is no way of deleting a password that is
    /// immutable
    pub fn clear_str(string: &mut str) {
        // safety: b'A' is a valid ascii character
        Password::_clear_str(string, b'A');
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // security: no cloning used here
        f.write_str(&self.0.bytes().map(|_| '*').collect::<String>())?;
        Ok(())
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Drop for Password {
    fn drop(&mut self) {
        Password::clear_str(&mut self.0);
    }
}


/// all possible errors [to_cred() can get]
#[derive(Debug)]
pub enum ToCredError {
    AuthFailed(ReqwestError),
    FileNoTOML(String, toml::de::Error),
}

impl Display for ToCredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToCredError::AuthFailed(err) => {
                f.write_str(&format!(
                    "Authentification failed, status code {:?}, \
                    is the password correct? Cause of error:\n{err}",
                    err.status().map(|status| status.to_string())
                ))?;
            }
            ToCredError::FileNoTOML(path, err) => {
                f.write_str(&format!(
                    "Parsing the file '{path}' failed, try deleting \
                    the file and retype the values. Cause of the error: {err}"
                ))?;
            }
        }
        Ok(())
    }
}

/// Credentials that are needed to connect to churchtools, that includes
/// the fqdn (fully qualified domain name, similar to url), the email
/// of the user with whom we want to log in and his password
#[derive(Debug)]
pub struct Credentials {
    fqdn: String,
    username: String,
    password: Password,
}

/// abbrevation for Churchtools
/// connection. Holds [Credentials]
/// and a logged in [Client]
#[derive(Debug)]
pub struct CTConn {
    cred: Credentials,
    client: Client,
}

#[derive(Debug)]
enum CTConnError {
    WrongFileFormat(String, toml::ser::Error),
    FileError(String, std::io::Error),
    ToCredFailed(ToCredError),
    MaxNumberRetries,
}

impl Display for CTConnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CTConnError::WrongFileFormat(path, err) => {
                f.write_fmt(format_args!(
                    "File '{path}' in wrong format. Consider removing \
                    the file and retyping the credentials. Cause of error:\n{err}"
                ))?;
            }
            CTConnError::FileError(path, err) => {
                f.write_fmt(format_args!(
                    "File '{path}' is not accessable. Do you have closed \
                    every program accessing the file? Cause of error:\n{err}"
                ))?;
            }
            CTConnError::ToCredFailed(err) => {
                f.write_str(&err.to_string())?;
            }
            CTConnError::MaxNumberRetries => {
                f.write_str(
                    "Max number of retries. \
                Start the program again to reconfigure or change NR_TYPE_TRYS and retry",
                )?;
            }
        }
        Ok(())
    }
}

/// convert the contents of a file to
/// a [CTConn] struct if it isn't
/// possible, return a fitting error
pub fn to_cred(path: String, file_content: &str) -> Result<CTConn, ToCredError> {
    let toml_file = file_content
        .parse::<toml::Value>()
        .map_err(|err| ToCredError::FileNoTOML(path, err))?;
    let fqdn = toml_file["fqdn"].as_str().unwrap().to_string();
    let email = toml_file["email"].as_str().unwrap().to_string();
    // as I understand this article:
    // https://stackoverflow.com/questions/54237610/is-there-a-way-to-make-an-immutable-reference-mutable
    // it is generally not possible to change a &str so unfortunately, the password is leaked
    // here into the memory without being cleared. I don't see a possiblity to use this library without
    // having this problem
    let password = &mut toml_file["password"].as_str().unwrap().clone();
    let cred = Credentials {
        fqdn,
        username: email,
        password: Password(password.to_string()),
    };
    let client = try_cred_ct(&cred).map_err(|err| ToCredError::AuthFailed(err))?;
    Ok(CTConn { cred, client })
}

/// gives an iterator that counts up as often as given by
/// NR_TYPE_TRYS
fn nr_type_loop_iter() -> impl Iterator<Item = u8> {
    match NR_TYPE_TRYS {
        None => Box::new(0u8..) as Box<dyn Iterator<Item = u8>>,
        Some(number) => Box::new(0u8..number) as Box<dyn Iterator<Item = u8>>,
    }
}

/// ask the user for the credentials
/// returns a (connection)[CTConn] to churchtools
/// repeat every typing in until it is correct
/// as defined in [NR_TYPE_TRYS]
/// panics if a non-utf8 character comes from input
/// returns None if a user has failed to
/// type in the correct values [NR_TYPE_TRYS] times
fn type_cred(input: &mut impl Read) -> Option<CTConn> {
    let mut fqdn = String::new();
    // in my opinion, the unwraps of input.read_to_string()
    // should never fail
    // in "production" where "input" is std::io::stdin()
    // as there, you can only type utf-8 characters
    // as far as I understand

    for _ in nr_type_loop_iter() {
        println!();
        print!("Please type in the fqdn of your churchtools website, \
            not ending with a \".\". Example for a fqdn: \"www.google.de.\" \
            (without the quotes): ");
        input.read_to_string(&mut fqdn).unwrap();
        if is_valid_address(&fqdn) {
            break;
        }
    }
    if !is_valid_address(&fqdn) {
        return None;
    }
    let mut email = String::new();
    for _ in nr_type_loop_iter() {
        println!();
        print!("Please type in the email of the user of your churchtools website: ");
        input.read_to_string(&mut email).unwrap();
        if is_valid_email(email.clone()) {
            break;
        }
    }
    if !is_valid_email(email.clone()) {
        return None;
    }
    let mut password;
    for _ in nr_type_loop_iter() {
        println!();
        print!("Please type in the password of that user: ");
        match read_password() {
            Ok(input) => password = Password(input),
            Err(_) => panic!(
                "I have no idea what to suggest to do as I have no idea \
                what this error means"
            ),
        };
        // mut to delete the password afterwards
        let cred = Credentials {
            fqdn: fqdn.clone(),
            username: email.clone(),
            password: password,
        };
        let client = try_cred_ct(&cred);
        match client {
            Ok(client) => {
                // only need to delete the password here, in every other case
                // the password isn't correct anyways
                return Some(CTConn { cred, client });
            }
            Err(err) => {
                println!("couldn't connect, reason: {}", err);
            }
        }
    }
    None
}

/// abbrevation for try Credentials churchtools
/// sign into churchtools with the credentials given
/// return a valid connection that has a CSRF Token saved
fn try_cred_ct(cred: &Credentials) -> Result<Client, ReqwestError> {
    let client = reqwest::blocking::Client::new();
    let post_str = format!(
        "https://{}/?q=login/ajax?email={}&password={}",
        cred.fqdn, cred.username, cred.password
    );
    client.post(post_str).send()?;
    Ok(client)
}

/// exactly the same code as readme in check-if-email-exists
/// but synchronous and adapted to String email
fn is_valid_email(email: String) -> bool {
    // Let's say we want to test the deliverability of someone@gmail.com.
    let input = CheckEmailInput::new(email);

    // Verify this email, using async/await syntax.
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(check_email(&input));
    // `result` is a `Vec<CheckEmailOutput>`, where the CheckEmailOutput
    // struct contains all information about our email.
    match result.is_reachable {
        Reachable::Safe => true,
        // documentation says, when not regulary checked,
        // this is assumed to be risky, we will never check
        // the underlying mail
        Reachable::Risky => true,
        Reachable::Invalid => false,
        // if only this provider blocks you
        // but you are not a hacker, then
        // you don't want this program to complain
        Reachable::Unknown => true,
    }
}

/// checks if an fqdn is available
/// doesn't check, if it is a churchtools website though
/// TODO: check if it is a churchtools website
fn is_valid_address(fqdn: &str) -> bool {
    match Client::new().get(format!("https://{fqdn}/")).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// path is the path of the config file
/// input should be std::io::stdin()
/// save_password_to_file should be SAVE_PASSWORD_TO_FILE
fn _get_ctconn(path: String, input: &mut impl Read, save_password_to_file: bool) -> Result<CTConn, CTConnError> {
    if save_password_to_file {
        match fs::read_to_string(&path) {
            Ok(result) => to_cred(path, &result).map_err(|err| CTConnError::ToCredFailed(err)),
            Err(_) => {
                let conn = type_cred(input).ok_or(CTConnError::MaxNumberRetries)?;
                let mut result = Map::new();
                result.insert("fqdn".to_string(), Value::String(conn.cred.fqdn.clone()));
                // TODO: unwrap panics
                std::fs::write(path.clone(), toml::to_string(&result).unwrap())
                    .map_err(|err| CTConnError::FileError(path, err))?;
                Ok(conn)
            }
        }
    }
    else {
        type_cred(input).ok_or(CTConnError::MaxNumberRetries)
    }
}

/// asks the user for [Credentials]
/// doesn't handle user giving wrong values
fn app_cred(fqdn: String) -> Credentials {
    let mut app = App::default();
    let mut window = Window::new(100, 100, 550, 135, "birthdays app");
    let mut fqdn = Input::new(150, 5, 300, 30, "fqdn of churchtools: ");
    let mut username = Input::new(150, 35, 300, 30, "username:");
    let mut password = fltk::input::SecretInput::new(150, 65, 300, 30, "password:");
    let mut button = Button::new(300, 100, 60, 30, "send");
    fqdn.set_tooltip("A \"fully qualified domain name\" (fqdn) \
        is a part of an url/link. It defines uniquely an IP adress and \
        therefore a server. The fqdn that is  needed here is the fqdn \
        from churchtools, without a trailing point \".\". An example \
        would be \"xxx.church.tools\" where \"xxx\" stand for the acronym your church has.");
    window.end();
    window.show();
    app.run().unwrap();
    Credentials {fqdn: fqdn.value(), username: username.value(), password: Password(password.value())}
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    #[test]
    fn test_clear_str() {
        password_test_env(|mut password: String| {
            password = remove_occurences(password, 'A', 'B');
            let password_copy = password.clone();
            super::Password::_clear_str(&mut password, b'A');
            let loop_iter = password.chars().zip(password_copy.chars());
            for (char1, char2) in loop_iter {
                assert_ne!(char1, char2, "password is not deleted correctly");
            }
        });
    }
    #[test]
    fn test_password_display() {
        password_test_env(|mut password: String| {
            password = remove_occurences(password, 'A', 'B');
            for (char1, char2) in format!("{}", password).chars().zip(password.chars()) {
                assert_ne!(char1, char2, "password is shown on Display")
            }
            for (char1, char2) in format!("{:?}", password).chars().zip(password.chars()) {
                assert_ne!(char1, char2, "password is shown on Debug")
            }
        })
    }
    fn remove_occurences(mut password: String, from_occ: char, to_occ: char) -> String {
        password = password.replace(from_occ, to_occ.to_string().as_str());
        password
    }
    fn password_test_env<F>(password_handler: F) where F: FnOnce(String) {
        // doesn't compile if it would contain non-ascii characters (try!)
        // doesn't include the 'A' symbol as this is to what the password is checked against
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzBCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!$%&/()=?+*#'{[]}\\~,.-;:_<>|";
        const PASSWORD_LENGTH: usize = 32;

        let mut count_equal: [u8; PASSWORD_LENGTH] = [0; PASSWORD_LENGTH];
        for i in 0..count_equal.len() {
            count_equal[i] = 0;
        }
        for _ in 0..1000 {
            let alphanumeric = Alphanumeric { };
            let byte_password = thread_rng()
                .sample_iter(& alphanumeric)
                .take(32)
                .collect::<Vec<u8>>();
            let one_char = || CHARSET[thread_rng().gen_range(0..CHARSET.len())] as char;
            let mut password: String = std::iter::repeat_with(one_char).take(PASSWORD_LENGTH).collect();
        }
        for res in count_equal {
            assert_eq!(res, 0)
        }
    }
}
