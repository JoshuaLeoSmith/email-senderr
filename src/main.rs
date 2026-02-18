use config::Config;
use std::collections::HashMap;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
fn main() {
    let settings = Config::builder()
        .add_source(config::File::with_name("src/Settings.toml"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();


    let from = format!("{} <{}>", settings.get("name").unwrap(), settings.get("username").unwrap());

    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(settings.get("recipient").unwrap().parse().unwrap())
        .subject("Test subject33333!!!")
        .body(String::from("hello my friend id 3333333lke to say hi")).unwrap();

    let creds = Credentials::new(
        String::from(settings.get("username").unwrap()),
        String::from(settings.get("password").unwrap()),
    );

    let mailer = SmtpTransport::relay(settings.get("host").unwrap()).unwrap()
        .credentials(creds)
        .build();

    // 4. Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }

}


// Requirements
// hit google api and login
// load template
// load args
// load attachments
// load email mapped to name????
// confirm???
// send