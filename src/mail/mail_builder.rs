use core::panic;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
    thread::current, sync::Arc,
};

use lettre::message::{Attachment, Body, MultiPart, SinglePart};
use regex::{Captures, Regex};
use rocket::tokio::fs;
use toml::Table;

use crate::{utils::InsignoError, InsignoConfig, auth::pending_users::token};

struct MailBuilder {
    components: HashMap<String, MailComponent>,
    regex: Regex,
    folder: PathBuf,
    ignore: HashSet<String>,
}
/*
impl MailBuilder {
    fn default() -> Self {
        Self {
            components: Default::default(),
            regex: Regex::new(r"\{(.*?)\}").unwrap(),
            ignore: vec!["link".to_string(), "user".to_string()].into_iter().collect(),

        }
    }
}*/

#[derive(Clone, Debug)]
struct MailComponent {
    message: String,
    attachments: HashMap<String, (i32, Body)>,
    last: i32,
    regex: Regex,
}

impl MailComponent {
    fn build(&self, values: HashMap<String, String>) -> Result<MultiPart, InsignoError> {
        let mex = self.message.clone();
        let mut error: Result<(), InsignoError> = Ok(());
        let err = &mut error;
        let mex = self.regex.replace_all(&mex, move |cap: &Captures| {
            if let Some(content) = values.get(&cap[1]) {
                content.clone()
            } else if let Some((id, content)) = self.attachments.get(&cap[1]) {
                format!("cid:{id}")
            } else {
                *err = Err(InsignoError::new_debug(
                    501,
                    &format!(
                        "Impossible to generate mail: can't find {}",
                        &cap[0]
                    ),
                ));
                "".to_string()
            }
        });
        error?;
        let mut ret = MultiPart::related().singlepart(SinglePart::html(mex.to_string()));
        for (id, body) in self.attachments.values() {
            ret = ret.singlepart(
                Attachment::new_inline(format!("cid:{}", id))
                    .body(body.clone(), "image/png".parse().unwrap()),
            );
        }
        Ok(ret)
    }

    fn new(
        builder: & mut MailBuilder,
        name: &str,
    ) -> Result<MailComponent, Box<dyn Error>> {
        let path = builder.folder.join(name);
        let content = std::fs::read(path)?;
        let regex= builder.regex.clone();
        let mut me = Self {
            message: String::from_utf8(content)?,
            attachments: HashMap::new(),
            last: 200,
            regex: regex.clone(),
        };
        let mut err: Result<(), Box<dyn Error>> = Ok(());
        let itsme=&mut me;
        me.message = regex
            .replace_all(&itsme.message.clone(), move |cap: &Captures| {
                
                if builder.ignore.contains(&cap[1]) {
                    return format!("{{{}}}", &cap[1]);
                }
                let comp = builder.get_or_new(&cap[1]).unwrap();
                let mret=comp.message.clone();
                let filtered: HashMap<String, (i32, Body)> = comp.attachments.clone().into_iter().filter_map(|(s, (_, body))|{
                    if itsme.attachments.contains_key(&s){
                        None
                    }else{
                        let ret = (s, (itsme.last, body));
                        itsme.last+=1;
                        Some(ret)
                    }
                }).collect();
                itsme.attachments.extend(filtered.into_iter());
                mret
                //cap[1].to_string()
            })
            .to_string();
        
        Ok(me)
    }
}

impl MailBuilder {
    fn get_or_new(&mut self, s: &str) -> Option<&MailComponent> {
        if self.components.get(s).is_some(){
            return self.components.get(s);
        }
        let path=self.folder.join(s);
        if path.extension()?.eq_ignore_ascii_case("png"){
            let file= std::fs::read(path).unwrap();
            let body = Body::new(file);
            self.components.insert(
                s.to_string(), 
                MailComponent{
                    message: s.to_string(),
                    attachments: vec![(s.to_string(), (200, body))].into_iter().collect(),
                    last: 200,
                    regex: self.regex.clone(),
                    
                });
            self.components.get(s)
        }else{
            let component= MailComponent::new(self, s).unwrap();
            self.components.insert(s.to_string(), component);
            self.components.get(s)

        }
    }

    /// try to open path toml, searches for array "templates"
    ///
    async fn new(config: InsignoConfig)->Self{
        let folder = Path::new(&config.template_folder);
        if !folder.is_dir() {
            panic!("template_folder is not a folder")
        }
        let config = folder.join("template.toml");
        let config = fs::read(config).await.unwrap();
        let config = String::from_utf8(config).unwrap();
        let config = config.parse::<Table>().unwrap();

        let templates = config
            .get(&"templates".to_string())
            .unwrap()
            .as_array()
            .unwrap();
        let map: HashMap<String, String> = HashMap::new();
        let mut me = MailBuilder{
            components: HashMap::new(),
            regex: Regex::new(r"\{(.*?)\}").unwrap(),
            ignore: vec!["link".to_string(), "user".to_string(), "email".to_string()].into_iter().collect(),
            folder: folder.into(),
        };
        for t in templates {
            let s = t.as_str().unwrap();
            if map.contains_key(s) {
                continue;
            }
            let comp = me.get_or_new(s).unwrap();
            
            
            /*let current_file = folder.join(t.as_str().unwrap());
            let current_file = fs::read(current_file).await.unwrap();
            let current_file = String::from_utf8(current_file).unwrap();
            let token_regex = Regex::new(r"\{(.*?)\}").unwrap();
            let y = &mut me;
            let w = token_regex.replace_all(&current_file, move |cap: &Captures| {
                println!("gonna catchem all {}", &cap[0]);
                y.replace(cap)
            });*/
        }
        //let comp = me.get_or_new("mail_account_creation.html").unwrap();
        //println!("{:?}", comp);
        me
    }
}

#[cfg(test)]
mod test {
    use crate::{mail::SmtpConfig, InsignoConfig};

    use super::MailBuilder;

    #[rocket::async_test]
    async fn test() {
        let conf = InsignoConfig {
            media_folder: String::default(),
            template_folder: "./templates".to_string(),
            oldest_supported_version: String::default(),
            smtp: SmtpConfig {
                server: String::default(),
                user: String::default(),
                password: String::default(),
            },
        };
        MailBuilder::new(conf).await;
        println!("nisba");
        todo!()
    }
}
