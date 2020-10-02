use clap::{
 App,
 Arg,
 ArgMatches
};
use regex::Regex;
use serde::{
 Deserialize,
 Serialize
};
use std::{
 cmp::Ordering,
 collections::HashMap
};

const TEMPLATE_PATTERN_NAME: &str = "{name}";
const TEMPLATE_PATTERN_VERSION: &str = "{version}";
const TEMPLATE_PATTERN_AUTHORS: &str = "{authors}";
const TEMPLATE_PATTERN_RESPOSITORY: &str = "{repository}";
const TEMPLATE_PATTERN_LICENSE: &str = "{license}";
const TEMPLATE_PATTERN_LICENSE_FILE: &str = "{license_file}";
const TEMPLATE_PATTERN_DESCRIPTION: &str = "{description}";
const REGEX_PATTERN_ANY: &str = ".*";
const OPTION_TEMPLATE_FILE: &str = "template-file";
const OPTION_INPUT_FILE: &str = "input-file";
const OPTION_OUTPUT_FILE: &str = "output-file";
const OPTION_HEADER_LINES: &str = "header-lines";
const OPTION_FOOTER_LINES: &str = "footer-lines";
const OPTION_MATCH_NAME: &str = "match-name";
const OPTION_MATCH_LICENSE: &str = "match-license";
const OPTION_MATCH_NAME_INVERT: &str = "match-name-invert";
const OPTION_MATCH_LICENSE_INVERT: &str = "match-license-invert";
const OPTION_ESCAPE_AUTHORS: &str = "escape-authors";
const EOL: &str = "\n";

/// An entity of CargoTomlLicenses
#[derive(Serialize, Deserialize, Eq, Ord)]
struct CargoTomlLicense
{
 name: String,
 version: String,
 authors: String,
 repository: Option<String>,
 license: Option<String>,
 license_file: Option<String>,
 description: Option<String>
}

impl PartialEq for CargoTomlLicense
{
 fn eq(&self, other: &CargoTomlLicense) -> bool
 {
  self.name.eq(&other.name) && self.version.eq(&other.version)
 }
}

impl PartialOrd for CargoTomlLicense
{
 fn partial_cmp(&self, other: &CargoTomlLicense) -> Option<Ordering>
 {
  self.name.partial_cmp(&other.name).or(self.version.partial_cmp(&other.version))
 }
}

/// An output of `cargo-licenses -j` command from Cargo.toml convertible license infos
type CargoTomlLicenses = Vec<CargoTomlLicense>;

/// An entity of PackageJsonLicenses
#[derive(Serialize, Deserialize)]
struct PackageJsonLicense
{
 licenses: Option<PackageJsonLicenseVariant>,
 repository: Option<String>,
 publisher: Option<String>,
 email: Option<String>,
 license_file: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum PackageJsonLicenseVariant
{
 S(String),
 V(Vec<String>)
}

/// An output of `license-checker --json` command from package.json convertible license infos
type PackageJsonLicenses = HashMap<String, PackageJsonLicense>;

fn main()
{
 // Parse the command-line
 let m = parse_command_line();

 // Load the template definition
 let (template_header, template_body, template_footer) = {
  let path = m.value_of(OPTION_TEMPLATE_FILE).expect("-t(--template) argument is must required.");
  let source = std::fs::read_to_string(path).expect(&format!("Could not read the template file: {}", path));
  let lines = source.lines().collect::<Vec<_>>();

  let header_lines = m
   .value_of(OPTION_HEADER_LINES)
   .and_then(|v| v.parse::<usize>().ok())
   .unwrap_or(0usize);

  let footer_lines = m
   .value_of(OPTION_FOOTER_LINES)
   .and_then(|v| v.parse::<usize>().ok())
   .unwrap_or(0usize);

  let header_range = 0..header_lines;
  let body_range = header_range.end..lines.len() - footer_lines;
  let footer_range = body_range.end..lines.len();

  (
   lines[header_range].join(EOL),
   lines[body_range].join(EOL),
   lines[footer_range].join(EOL)
  )
 };

 // Load the licenses from a JSON file
 let licenses = {
  let path = m.value_of(OPTION_INPUT_FILE).expect("-i(--input) argument is must required.");
  let source = std::fs::read_to_string(path).expect(&format!("Could not read the input file: {}", path));
  let licenses_maybe = serde_json::from_str::<CargoTomlLicenses>(&source);
  match licenses_maybe
  {
   Ok(licenses) => licenses,
   _ =>
   {
    // license-checker type, maybe
    let licenses = serde_json::from_str::<PackageJsonLicenses>(&source).expect(&format!("Could not parse the input file: {}", path));
    convert_source(licenses)
   }
  }
 };

 // Filtering by a name regex and a license regex
 let licenses = {
  let match_name = m.value_of(OPTION_MATCH_NAME).unwrap_or(REGEX_PATTERN_ANY);
  let match_name = Regex::new(match_name).expect(&format!("--match-name, a wrong regex pattern: {}", match_name));
  let match_license = m.value_of(OPTION_MATCH_LICENSE).unwrap_or(REGEX_PATTERN_ANY);
  let match_license = Regex::new(match_license).expect(&format!("--match-license, a wrong regex pattern: {}", match_license));
  let match_name_invert = m.is_present(OPTION_MATCH_NAME_INVERT);
  let match_license_invert = m.is_present(OPTION_MATCH_LICENSE_INVERT);
  licenses.into_iter().filter(move |l| {
   let result_name = match match_name_invert
   {
    true => !match_name.is_match(&l.name),
    _ => match_name.is_match(&l.name)
   };
   match result_name
   {
    true =>
    {
     match match_license_invert
     {
      true => !match_license.is_match(&l.license.clone().unwrap_or_default()),
      _ => match_license.is_match(&l.license.clone().unwrap_or_default())
     }
    },
    _ => false
   }
  })
 };

 // Sorting
 let licenses = {
  let mut licenses = licenses.collect::<Vec<_>>();
  licenses.sort();
  licenses
 };

 // Extruding (Output)
 let escape_authors = m.is_present(OPTION_ESCAPE_AUTHORS);
 let extruded = {
  let bodies = licenses
   .into_iter()
   .map(|l| extrude(l, &template_body, escape_authors))
   .collect::<Vec<_>>()
   .join(EOL);
  let header = match template_header.is_empty()
  {
   true => String::default(),
   false => format!("{}{}", &template_header, EOL)
  };
  format!("{}{}{}{}", header, bodies, EOL, template_footer)
 };

 // Some => Output to FILE
 // None => Output to STDOUT
 let o = m.value_of(OPTION_OUTPUT_FILE);
 match o
 {
  Some(path) => std::fs::write(path, &extruded).unwrap(),
  None => println!("{}", &extruded)
 };
}

fn extrude(from: CargoTomlLicense, template: &String, escape_authors: bool) -> String
{
 let authors = match escape_authors
 {
  true => from.authors.replace("<", "&lt;").replace(">", "&gt;"),
  false => from.authors
 };

 template
  .replace(TEMPLATE_PATTERN_NAME, &from.name)
  .replace(TEMPLATE_PATTERN_VERSION, &from.version)
  .replace(TEMPLATE_PATTERN_AUTHORS, &authors)
  .replace(TEMPLATE_PATTERN_RESPOSITORY, &from.repository.unwrap_or_default())
  .replace(TEMPLATE_PATTERN_LICENSE, &from.license.unwrap_or_default())
  .replace(TEMPLATE_PATTERN_LICENSE_FILE, &from.license_file.unwrap_or_default())
  .replace(TEMPLATE_PATTERN_DESCRIPTION, &from.description.unwrap_or_default())
}

/// + `from` The licenses (from a package.json using `license-checker`; HasMap{ name@ver: { ... } })
/// + `return` The licenses converted to `cargo-licenses` convertible format (from a Cargo.toml; Vec[ { ... } ])
fn convert_source(from: PackageJsonLicenses) -> CargoTomlLicenses
{
 from
  .iter()
  .map(|(k, v)| {
   let name_at_start = k.starts_with("@");
   let name_and_version = k.trim_start_matches("@").splitn(2, "@").collect::<Vec<&str>>();
   let name = match name_at_start
   {
    true => format!("@{}", name_and_version[0]),
    _ => name_and_version[0].to_string()
   };
   let version = name_and_version[1].to_string();
   let authors = match (&v.publisher, &v.email)
   {
    (Some(publisher), Some(email)) => format!("{} <{}>", publisher, email),
    (Some(publisher), None) => format!("{}", publisher),
    (None, Some(email)) => format!("<{}>", email),
    _ => String::default()
   };
   let license = match v.licenses.clone()
   {
    Some(PackageJsonLicenseVariant::S(s)) => Some(s),
    Some(PackageJsonLicenseVariant::V(v)) => Some(v.join(",")),
    _ => None
   };
   CargoTomlLicense {
    name,
    version,
    authors,
    repository: v.repository.clone(),
    license,
    license_file: v.license_file.clone(),
    description: None
   }
  })
  .collect::<Vec<_>>()
}

fn parse_command_line() -> ArgMatches<'static>
{
 let a = App::new(env!("CARGO_PKG_NAME"))
  .arg(
   Arg::with_name(OPTION_TEMPLATE_FILE)
    .short("t")
    .long(OPTION_TEMPLATE_FILE)
    .help("A template file")
    .takes_value(true)
    .required(true)
  )
  .arg(
   Arg::with_name(OPTION_INPUT_FILE)
    .short("i")
    .long(OPTION_INPUT_FILE)
    .help("An input source JSON file")
    .takes_value(true)
    .required(true)
  )
  .arg(
   Arg::with_name(OPTION_OUTPUT_FILE)
    .short("o")
    .long(OPTION_OUTPUT_FILE)
    .help("An output file path; Output to STDOUT if not specified")
    .takes_value(true)
  )
  .arg(
   Arg::with_name(OPTION_HEADER_LINES)
    .short("h")
    .long(OPTION_HEADER_LINES)
    .help("A number of the header lines in the template file")
    .default_value("0")
    .takes_value(true)
  )
  .arg(
   Arg::with_name(OPTION_FOOTER_LINES)
    .short("f")
    .long(OPTION_FOOTER_LINES)
    .help("A number of the footer lines in the template file")
    .default_value("0")
    .takes_value(true)
  )
  .arg(
   Arg::with_name(OPTION_MATCH_NAME)
    .long(OPTION_MATCH_NAME)
    .help("A regex pattern fintering for a name")
    .takes_value(true)
  )
  .arg(
   Arg::with_name(OPTION_MATCH_LICENSE)
    .long(OPTION_MATCH_LICENSE)
    .help("A regex pattern fintering for a license")
    .takes_value(true)
  )
  .arg(
   Arg::with_name(OPTION_ESCAPE_AUTHORS)
    .long(OPTION_ESCAPE_AUTHORS)
    .help("If set => `It's Me <its_me@example.com>` -> `It's Me &ltemail@example.com&gt`")
  )
  .arg(
   Arg::with_name(OPTION_MATCH_NAME_INVERT)
    .long(OPTION_MATCH_NAME_INVERT)
    .help("If set => Invert a `--match-name` result")
  )
  .arg(
   Arg::with_name(OPTION_MATCH_LICENSE_INVERT)
    .long(OPTION_MATCH_LICENSE_INVERT)
    .help("If set => Invert a `--match-license` result")
  );

 a.get_matches()
}
