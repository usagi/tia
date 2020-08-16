mod generator;
mod parser;

use proc_macro as pm;
use std::collections::{
 HashMap,
 HashSet
};

const EMPTY_STR: &str = "";
use self::EMPTY_STR as NO_SEPARATOR;

#[derive(Debug)]
pub struct Tia
{
 target_type_symbol:       TargetTypeSymbol,
 target_type:              TargetType,
 trait_to_field_accessors: TraitToFieldAccessors
}

pub type TraitToFieldAccessors = HashMap<TraitSymbol, FieldSymbolToFieldParams>;
pub type FieldSymbolToFieldParams = HashMap<FieldSymbol, FieldParams>;
pub type TargetTypeSymbol = String;
pub type FieldSymbol = String;
pub type FieldType = String;
pub type TraitSymbol = String;

#[derive(Debug)]
pub enum TargetType
{
 Struct,
 Enum,
 Union
}

#[derive(Debug, Clone, Eq)]
pub enum Accessor
{
 Setter
 {
  fsp: FieldSymbolPolicy,
  ptp: SetterParameterTypePolicy
 },
 Getter
 {
  fsp: FieldSymbolPolicy,
  ptp: GetterParameterTypePolicy
 }
}

impl PartialEq for Accessor
{
 fn eq(&self, other: &Self) -> bool
 {
  match (self, other)
  {
   (
    Accessor::Getter {
     fsp: _,
     ptp: _
    },
    Accessor::Getter {
     fsp: _,
     ptp: _
    }
   )
   | (
    Accessor::Setter {
     fsp: _,
     ptp: _
    },
    Accessor::Setter {
     fsp: _,
     ptp: _
    }
   ) => true,
   _ => false
  }
 }
}

impl std::hash::Hash for Accessor
{
 fn hash<H: std::hash::Hasher>(&self, h: &mut H)
 {
  match self
  {
   Accessor::Getter {
    fsp: _,
    ptp: _
   } => 0.hash(h),
   Accessor::Setter {
    fsp: _,
    ptp: _
   } => 1.hash(h)
  }
 }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GetterParameterTypePolicy
{
 Move,
 Value,
 Ref,
 RefMut
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SetterParameterTypePolicy
{
 Value,
 RefClone,
 Into
}

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FieldSymbolPolicy
{
 Default,
 Prefix(String),
 Suffix(String),
 Fullname(String)
}

#[derive(Debug, Default, Clone)]
pub struct FieldParams
{
 pub field_type: FieldType,
 pub accessors:  HashSet<Accessor>
}

impl Tia
{
 pub fn new(i: syn::DeriveInput) -> Self
 {
  parser::parse(i)
 }
}

impl Into<pm::TokenStream> for Tia
{
 fn into(self) -> pm::TokenStream
 {
  let impl_definitions = generator::generate_impl_definitions(&self.trait_to_field_accessors, &self.target_type_symbol, self.target_type);

  if cfg!(feature = "print")
  {
   eprintln!("[proc-macro:tia +print Target: {}]", &self.target_type_symbol);
   eprintln!("{}", &impl_definitions);
  }

  if cfg!(feature = "file") || cfg!(feature = "file-pretty") || cfg!(feature = "include") || cfg!(feature = "include-pretty")
  {
   write_file(&self.target_type_symbol, &impl_definitions)
  };

  impl_definitions
   .parse::<pm::TokenStream>()
   .expect("tia::into<proc_macro::TokenStream> was failed. #TIA-PANIC-4001")
 }
}

fn write_file(target_type_symbol: &String, source: &String)
{
 use std::{
  fs::{
   create_dir_all,
   File
  },
  io::Write
 };

 let out_dir = "src/.tia/";
 create_dir_all(out_dir).unwrap();

 let path = format!("{}{}.rs", out_dir, &target_type_symbol);
 let mut file = File::create(&path).unwrap();
 write!(file, "{}", source).unwrap();

 if cfg!(feature = "file-pretty") || cfg!(feature = "include-pretty")
 {
  let result = if file_pretty(&path) { "<OK>" } else { "<NG>" };
  eprintln!(
   "[proc-macro:tia +file-pretty Target: {} => {} ; rustfmt {} ]",
   &target_type_symbol, &path, result
  );
 }
 else
 {
  eprintln!("[proc-macro:tia +file Target: {} => {}]", &target_type_symbol, &path);
 }
}

fn file_pretty(path: &String) -> bool
{
 std::process::Command::new("rustfmt")
  .arg("-q")
  .arg(path)
  .stdin(std::process::Stdio::null())
  .stdout(std::process::Stdio::null())
  .stderr(std::process::Stdio::null())
  .spawn()
  .and_then(|mut child_process| child_process.wait())
  .is_ok()
}
