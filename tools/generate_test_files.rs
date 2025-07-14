/*!
 * ZippyPack - Générateur de fichiers de test
 * 
 * Créé par : Kamion (Matthéo Le Fur)
 * Date : 26/06/2025
 * Modifié le : 14/07/2025
 * 
 * Description : Utilitaire pour générer des fichiers de test réalistes
 * optimisés pour tester la déduplication de ZippyPack
 * 
 * Version : 1.0.0
 */

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let test_dir = Path::new("test_files");
    
    // Nettoyer le dossier
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;
    
    println!("Génération des fichiers de test...");
    
    // 1. Différents composants React (similaires mais uniques)
    let react_components = vec![
        ("Header", "function Header() { return <header><h1>Mon App</h1><nav>Navigation</nav></header>; }"),
        ("Footer", "function Footer() { return <footer><p>&copy; 2024 Mon App</p></footer>; }"),
        ("Button", "function Button({ onClick, children }) { return <button onClick={onClick}>{children}</button>; }"),
        ("Modal", "function Modal({ isOpen, onClose, children }) { return isOpen ? <div className=\"modal\">{children}</div> : null; }"),
        ("Card", "function Card({ title, content }) { return <div className=\"card\"><h3>{title}</h3><p>{content}</p></div>; }"),
        ("List", "function List({ items }) { return <ul>{items.map(item => <li key={item.id}>{item.name}</li>)}</ul>; }"),
    ];
    
    for (i, (name, component)) in react_components.iter().enumerate() {
        for variant in 1..=20 {
            let filename = format!("test_files/react_{}_{:02}.jsx", name.to_lowercase(), variant);
            let mut file = File::create(&filename)?;
            
            // Ajouter des imports communs + le composant + variations
            let content = format!(
                "import React from 'react';\nimport './styles.css';\n\n{}\n\nexport default {};\n\n// Variant {}\n{}",
                component,
                name,
                variant,
                "// Component utility functions\nconst utils = { format: (text) => text.toUpperCase() };\n".repeat(100)
            );
            
            file.write_all(content.as_bytes())?;
        }
        
        println!("Créé {} variantes de {}", 20, name);
    }
    
    // 2. Différents services Python (logique métier variée)
    let python_services = vec![
        ("UserService", "class UserService:\n    def __init__(self):\n        self.users = {}\n    \n    def create_user(self, name, email):\n        return {'id': len(self.users), 'name': name, 'email': email}\n    \n    def get_user(self, user_id):\n        return self.users.get(user_id)\n"),
        ("AuthService", "class AuthService:\n    def __init__(self):\n        self.tokens = {}\n    \n    def login(self, username, password):\n        if self.validate_credentials(username, password):\n            return self.generate_token(username)\n        return None\n    \n    def validate_credentials(self, username, password):\n        return username == 'admin' and password == 'secret'\n"),
        ("DatabaseService", "class DatabaseService:\n    def __init__(self, connection_string):\n        self.connection = connection_string\n        self.queries = []\n    \n    def execute_query(self, query):\n        self.queries.append(query)\n        return {'status': 'success', 'rows': 42}\n    \n    def get_connection(self):\n        return self.connection\n"),
        ("EmailService", "class EmailService:\n    def __init__(self, smtp_server):\n        self.smtp_server = smtp_server\n        self.sent_emails = []\n    \n    def send_email(self, to, subject, body):\n        email = {'to': to, 'subject': subject, 'body': body}\n        self.sent_emails.append(email)\n        return True\n"),
        ("LoggerService", "class LoggerService:\n    def __init__(self, log_level='INFO'):\n        self.log_level = log_level\n        self.logs = []\n    \n    def log(self, message, level='INFO'):\n        log_entry = {'message': message, 'level': level, 'timestamp': 'now'}\n        self.logs.append(log_entry)\n        print(f'[{level}] {message}')\n"),
    ];
    
    for (name, service_code) in python_services.iter() {
        for variant in 1..=30 {
            let filename = format!("test_files/service_{}_{:02}.py", name.to_lowercase(), variant);
            let mut file = File::create(&filename)?;
            
            let content = format!(
                "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n\nimport json\nimport logging\nfrom typing import Dict, List, Optional\n\n{}\n\n# Utility functions (variant {})\ndef format_response(data):\n    return json.dumps(data, indent=2)\n\ndef validate_input(data):\n    return data is not None and len(str(data)) > 0\n\n{}\n\nif __name__ == '__main__':\n    service = {}()\n    print('Service initialized successfully')\n",
                service_code,
                variant,
                "# Additional helper functions\ndef process_data(items):\n    return [item.upper() for item in items]\n\ndef calculate_hash(text):\n    return hash(text) % 10000\n".repeat(50),
                name
            );
            
            file.write_all(content.as_bytes())?;
        }
        
        println!("Créé {} variantes de {}", 30, name);
    }
    
    // 3. Différents modules Rust (structures de données variées)
    let rust_modules = vec![
        ("cache", "use std::collections::HashMap;\n\npub struct Cache<K, V> {\n    data: HashMap<K, V>,\n    max_size: usize,\n}\n\nimpl<K, V> Cache<K, V> where K: std::hash::Hash + Eq {\n    pub fn new(max_size: usize) -> Self {\n        Cache { data: HashMap::new(), max_size }\n    }\n    \n    pub fn get(&self, key: &K) -> Option<&V> {\n        self.data.get(key)\n    }\n    \n    pub fn insert(&mut self, key: K, value: V) {\n        if self.data.len() >= self.max_size {\n            self.evict_oldest();\n        }\n        self.data.insert(key, value);\n    }\n}"),
        ("queue", "use std::collections::VecDeque;\n\npub struct Queue<T> {\n    items: VecDeque<T>,\n    max_capacity: usize,\n}\n\nimpl<T> Queue<T> {\n    pub fn new(capacity: usize) -> Self {\n        Queue { items: VecDeque::new(), max_capacity: capacity }\n    }\n    \n    pub fn enqueue(&mut self, item: T) -> Result<(), &'static str> {\n        if self.items.len() >= self.max_capacity {\n            return Err(\"Queue is full\");\n        }\n        self.items.push_back(item);\n        Ok(())\n    }\n    \n    pub fn dequeue(&mut self) -> Option<T> {\n        self.items.pop_front()\n    }\n}"),
        ("tree", "pub struct TreeNode<T> {\n    value: T,\n    left: Option<Box<TreeNode<T>>>,\n    right: Option<Box<TreeNode<T>>>,\n}\n\nimpl<T> TreeNode<T> where T: Ord {\n    pub fn new(value: T) -> Self {\n        TreeNode { value, left: None, right: None }\n    }\n    \n    pub fn insert(&mut self, value: T) {\n        if value < self.value {\n            match &mut self.left {\n                Some(left) => left.insert(value),\n                None => self.left = Some(Box::new(TreeNode::new(value))),\n            }\n        } else {\n            match &mut self.right {\n                Some(right) => right.insert(value),\n                None => self.right = Some(Box::new(TreeNode::new(value))),\n            }\n        }\n    }\n}"),
        ("parser", "pub struct Parser {\n    input: String,\n    position: usize,\n}\n\nimpl Parser {\n    pub fn new(input: String) -> Self {\n        Parser { input, position: 0 }\n    }\n    \n    pub fn parse_number(&mut self) -> Result<i32, &'static str> {\n        let start = self.position;\n        while self.position < self.input.len() && self.current_char().is_ascii_digit() {\n            self.position += 1;\n        }\n        \n        if start == self.position {\n            return Err(\"Expected number\");\n        }\n        \n        self.input[start..self.position].parse().map_err(|_| \"Invalid number\")\n    }\n    \n    fn current_char(&self) -> char {\n        self.input.chars().nth(self.position).unwrap_or('\\0')\n    }\n}"),
    ];
    
    for (module_name, module_code) in rust_modules.iter() {
        for variant in 1..=40 {
            let filename = format!("test_files/{}_{:02}.rs", module_name, variant);
            let mut file = File::create(&filename)?;
            
            let content = format!(
                "// Module: {} - Variant {}\n\nuse std::fmt::Debug;\nuse std::collections::HashMap;\n\n{}\n\n// Common utilities\nfn log_debug(message: &str) {{\n    println!(\"[DEBUG] {{}}\", message);\n}}\n\nfn validate_input<T: Debug>(input: &T) -> bool {{\n    println!(\"Validating: {{:?}}\", input);\n    true\n}}\n\n{}\n\n#[cfg(test)]\nmod tests {{\n    use super::*;\n    \n    #[test]\n    fn test_basic_functionality() {{\n        // Test code for variant {}\n        assert!(true);\n    }}\n}}\n",
                module_name,
                variant,
                module_code,
                "// Additional helper functions\nfn process_result<T>(result: Result<T, &str>) -> Option<T> {\n    match result {\n        Ok(value) => Some(value),\n        Err(_) => None,\n    }\n}\n\nfn format_output(data: &str) -> String {\n    format!(\"[OUTPUT] {}\", data)\n}\n".repeat(30),
                variant
            );
            
            file.write_all(content.as_bytes())?;
        }
        
        println!("Créé {} variantes de {}", 40, module_name);
    }
    
    // 4. Différents fichiers de configuration (formats variés)
    let config_types = vec![
        ("database", "{\n  \"host\": \"localhost\",\n  \"port\": 5432,\n  \"database\": \"myapp\",\n  \"username\": \"admin\",\n  \"password\": \"secret\",\n  \"ssl\": true,\n  \"timeout\": 30,\n  \"pool_size\": 10\n}"),
        ("server", "server:\n  host: 0.0.0.0\n  port: 8080\n  threads: 4\n  timeout: 60\n\nlogging:\n  level: info\n  file: app.log\n  max_size: 10MB\n\nfeatures:\n  auth: true\n  cache: true\n  metrics: true"),
        ("build", "[package]\nname = \"myapp\"\nversion = \"1.0.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = { version = \"1.0\", features = [\"derive\"] }\ntokio = { version = \"1.0\", features = [\"full\"] }\naxum = \"0.6\"\n\n[dev-dependencies]\ntokio-test = \"0.4\""),
    ];
    
    for (config_name, config_content) in config_types.iter() {
        for variant in 1..=25 {
            let filename = format!("test_files/config_{}_{:02}.json", config_name, variant);
            let mut file = File::create(&filename)?;
            
            let content = format!(
                "// Configuration file for {} - Variant {}\n{}\n\n// Additional settings\n{}\n",
                config_name,
                variant,
                config_content,
                "// Default values\n// Environment: development\n// Version: 1.0.0\n// Last updated: 2024-01-01\n".repeat(200)
            );
            
            file.write_all(content.as_bytes())?;
        }
        
        println!("Créé {} variantes de {}", 25, config_name);
    }
    
    println!("Génération terminée !");
    println!("Total: {} fichiers créés", 6*20 + 5*30 + 4*40 + 3*25);
    println!("Fichiers de code différents avec similarités naturelles pour tester la déduplication.");
    
    Ok(())
}