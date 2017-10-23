extern crate git2;
extern crate hex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate ring;
extern crate rusoto_credential;
extern crate time;
extern crate url;

use git2::{Cred, CredentialType, Error};
use hex::ToHex;
use regex::Regex;
use ring::{digest, hmac};
use rusoto_credential::{DefaultCredentialsProvider, ProvideAwsCredentials};
use std::error::Error as StdError;
use time::{Tm, now_utc};
use url::Url;

macro_rules! gtry {
    ($expr:expr) => (match $expr {
        Ok(val) => val,
        Err(err) => {
            return Err(Error::from_str(err.description()));
        }
    })
}

pub fn codecommit_credentials(url_str: &str,
                              _: Option<&str>,
                              _: CredentialType)
                              -> Result<Cred, Error> {
    lazy_static! {
        static ref HOST_RE: Regex = Regex::new(r"git-codecommit\.([a-z0-9-]+)\.amazonaws\.com")
            .unwrap();
    }

    let url = gtry!(Url::parse(url_str));

    // Verify hostname is git-codecommit.*.amazonaws.com. This will fail if a partition on a
    // different domain suffix (i.e. China) eventually gets CodeCommit.
    let host = url.host_str().unwrap_or("");
    let region = match HOST_RE.captures(host) {
        Some(cap) => {
            match cap.get(1) {
                Some(s) => s.as_str(),
                None => return Cred::default(),
            }
        }
        None => return Cred::default(),
    };

    let credentials = gtry!(gtry!(DefaultCredentialsProvider::new()).credentials());

    let username = match *credentials.token() {
        Some(ref t) => format!("{}%{}", credentials.aws_access_key_id(), t),
        None => credentials.aws_access_key_id().to_string(),
    };
    debug!("username: {}", username);

    let date = now_utc();
    let canonical_request = format!("GIT\n{}\n\nhost:{}\n\nhost\n", url.path(), host);
    debug!("canonical_request: {:?}", canonical_request);
    let string_to_sign = format!("AWS4-HMAC-SHA256\n{}\n{}/{}/codecommit/aws4_request\n{}",
                                 gtry!(date.strftime("%Y%m%dT%H%M%S")),
                                 gtry!(date.strftime("%Y%m%d")),
                                 region,
                                 to_hexdigest(canonical_request));
    debug!("string_to_sign: {:?}", string_to_sign);
    let signing_key = signing_key(credentials.aws_secret_access_key(),
                                  date,
                                  region,
                                  "codecommit");
    let password = format!("{}{}",
                           gtry!(date.strftime("%Y%m%dT%H%M%SZ")),
                           signature(&string_to_sign, &signing_key));
    debug!("password: {}", password);

    Cred::userpass_plaintext(&username, &password)
}

// Below functions are lifted from rusoto::signature (MIT license)

fn to_hexdigest<T: AsRef<[u8]>>(t: T) -> String {
    let h = digest::digest(&digest::SHA256, t.as_ref());
    h.as_ref().to_hex().to_string()
}

fn signature(string_to_sign: &str, signing_key: &hmac::SigningKey) -> String {
    hmac::sign(signing_key, string_to_sign.as_bytes()).as_ref().to_hex().to_string()
}

fn signing_key(secret: &str, date: Tm, region: &str, service: &str) -> hmac::SigningKey {
    let date_key = hmac::SigningKey::new(&digest::SHA256, format!("AWS4{}", secret).as_bytes());
    let date_hmac = hmac::sign(&date_key,
                               date.strftime("%Y%m%d").unwrap().to_string().as_bytes());

    let region_key = hmac::SigningKey::new(&digest::SHA256, date_hmac.as_ref());
    let region_hmac = hmac::sign(&region_key, region.as_bytes());

    let service_key = hmac::SigningKey::new(&digest::SHA256, region_hmac.as_ref());
    let service_hmac = hmac::sign(&service_key, service.as_bytes());

    let signing_key = hmac::SigningKey::new(&digest::SHA256, service_hmac.as_ref());
    let signing_hmac = hmac::sign(&signing_key, b"aws4_request");

    hmac::SigningKey::new(&digest::SHA256, signing_hmac.as_ref())
}
