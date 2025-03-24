use std::collections::HashMap;
use std::fs;

use fluent::{FluentBundle, FluentResource, FluentArgs};
use unic_langid::LanguageIdentifier;

// 支持的语言
#[derive(PartialEq, Clone, Copy)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Chinese => "zh-CN",
        }
    }
    
    pub fn get_display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }
    
    pub fn all() -> Vec<Language> {
        vec![Language::English, Language::Chinese]
    }
    
    pub fn from_str(lang_str: &str) -> Option<Language> {
        match lang_str {
            "en-US" => Some(Language::English),
            "zh-CN" => Some(Language::Chinese),
            _ => None,
        }
    }
}

// 管理本地化的主结构
pub struct Locale {
    current_language: Language,
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl Locale {
    // 创建一个新的本地化管理器，使用指定的语言
    pub fn new(lang: Language) -> Self {
        let mut locale = Self {
            current_language: lang,
            bundles: HashMap::new(),
        };
        
        // 加载语言文件
        locale.load_resources();
        
        locale
    }
    
    // 设置当前语言
    pub fn set_language(&mut self, lang: Language) {
        self.current_language = lang;
    }
    
    // 获取当前语言
    pub fn get_language(&self) -> &Language {
        &self.current_language
    }
    
    // 加载语言资源文件
    fn load_resources(&mut self) {
        let langs = Language::all();
        
        for lang in langs {
            let lang_str = lang.as_str();
            let lang_id: LanguageIdentifier = lang_str.parse().expect("Invalid language identifier");
            
            let mut bundle = FluentBundle::new(vec![lang_id]);
            
            // 加载主语言文件
            let main_path = format!("src/i18n/locales/{}/main.ftl", lang_str);
            if let Ok(source) = Self::read_file(&main_path) {
                let resource = FluentResource::try_new(source)
                    .expect(&format!("Failed to parse Fluent resource for {}", lang_str));
                
                bundle
                    .add_resource(resource)
                    .expect(&format!("Failed to add Fluent resource for {}", lang_str));
                
                self.bundles.insert(lang_str.to_string(), bundle);
            }
        }
    }
    
    // 读取文件内容
    fn read_file(path: &str) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }
    
    // 获取指定键对应的本地化文本
    pub fn get_message(&self, key: &str) -> String {
        let lang_str = self.current_language.as_str();
        
        if let Some(bundle) = self.bundles.get(lang_str) {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    let result = bundle.format_pattern(pattern, None, &mut errors);
                    
                    if errors.is_empty() {
                        return result.to_string();
                    }
                }
            }
        }
        
        // 如果找不到对应的消息，返回键名
        format!("Unknown message key: {}", key)
    }
    
    // 检索一个消息并替换参数
    pub fn get_message_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let lang_str = self.current_language.as_str();
        
        if let Some(bundle) = self.bundles.get(lang_str) {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    
                    // 创建args map
                    let mut fluent_args = FluentArgs::new();
                    for &(name, value) in args {
                        fluent_args.set(name, value.to_string());
                    }
                    
                    let result = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
                    
                    if errors.is_empty() {
                        return result.to_string();
                    }
                }
            }
        }
        
        // 如果找不到对应的消息，返回键名
        format!("Unknown message key: {}", key)
    }
}