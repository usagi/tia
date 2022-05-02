use super::*;
use once_cell::sync::Lazy;

static DEFAULT_GET_FIELD_SYMBOL_POLICY: Lazy<FieldSymbolPolicy> = Lazy::new(|| FieldSymbolPolicy::Prefix("get".to_string()));
static DEFAULT_SET_FIELD_SYMBOL_POLICY: Lazy<FieldSymbolPolicy> = Lazy::new(|| FieldSymbolPolicy::Prefix("set".to_string()));

pub fn generate_impl_definitions(ttfa: &TraitToFieldAccessors, impl_target_symbol: &String, target_type: TargetType) -> String
{
 ttfa
  .iter()
  .map(|(trait_symbol, field_to_accessors)| generate_impl_definition(trait_symbol, field_to_accessors, &impl_target_symbol, &target_type))
  .collect::<Vec<String>>()
  .join(NO_SEPARATOR)
}

fn generate_impl_definition(
 trait_symbol: &TraitSymbol,
 field_to_accessors: &FieldSymbolToFieldParams,
 impl_target_symbol: &String,
 target_type: &TargetType
) -> String
{
 let is_pub = trait_symbol.is_empty();
 let header = generate_impl_header(trait_symbol, impl_target_symbol);
 let body = generate_impl_body(field_to_accessors, is_pub, target_type);
 let footer = generate_impl_footer();

 vec![header, body, footer]
  .join(NO_SEPARATOR)
  .into()
}

fn generate_impl_header(trait_symbol: &TraitSymbol, impl_target_symbol: &String) -> String
{
 let trait_part = match trait_symbol.is_empty()
 {
  true => EMPTY_STR.into(),
  false => format!("{} for ", trait_symbol)
 };
 format!("impl {}{}{{", trait_part, impl_target_symbol)
}

fn generate_impl_body(field_to_accessors: &FieldSymbolToFieldParams, is_pub: bool, target_type: &TargetType) -> String
{
 field_to_accessors
  .iter()
  .map(|(field_symbol, field_params)| generate_field_accessors(field_symbol, field_params, is_pub, target_type))
  .collect::<Vec<String>>()
  .join(NO_SEPARATOR)
}

fn generate_impl_footer() -> String { "}".into() }

fn generate_field_accessors(field_symbol: &FieldSymbol, field_params: &FieldParams, is_pub: bool, target_type: &TargetType) -> String
{
 let FieldParams {
  field_type,
  accessors
 } = field_params;

 accessors
  .iter()
  .map(|accessor| generate_field_accessor(field_symbol, field_type, accessor, is_pub, target_type))
  .collect::<Vec<String>>()
  .join(NO_SEPARATOR)
}

fn generate_field_accessor(
 field_symbol: &FieldSymbol,
 field_type: &FieldType,
 accessor: &Accessor,
 is_pub: bool,
 target_type: &TargetType
) -> String
{
 let fn_definition = match accessor
 {
  Accessor::Getter {
   fsp,
   ptp
  } =>
  {
   let unsafe_token = match target_type
   {
    TargetType::Union => "unsafe ",
    _ => ""
   };
   format!("{}{}", unsafe_token, generate_get_accessor(field_symbol, field_type, fsp, ptp))
  },
  Accessor::Setter {
   fsp,
   ptp
  } => generate_set_accessor(field_symbol, field_type, fsp, ptp)
 };

 let pub_token = match is_pub
 {
  true => "pub ",
  false => ""
 };

 format!("{}{}", pub_token, fn_definition)
}

fn generate_get_accessor(
 field_symbol: &FieldSymbol,
 field_type: &FieldType,
 fsp: &FieldSymbolPolicy,
 gptp: &GetterParameterTypePolicy
) -> String
{
 let fsp = match fsp
 {
  FieldSymbolPolicy::Default => &DEFAULT_GET_FIELD_SYMBOL_POLICY,
  _ => fsp
 };
 let function_symbol = generate_function_symbol(field_symbol, fsp);
 match gptp
 {
  GetterParameterTypePolicy::Move => format!("fn {}(self)->{}{{self.{}}}", function_symbol, field_type, field_symbol),
  GetterParameterTypePolicy::Value => format!("fn {}(&self)->{}{{self.{}}}", function_symbol, field_type, field_symbol),
  GetterParameterTypePolicy::Ref => format!("fn {}(&self)->&{}{{&self.{}}}", function_symbol, field_type, field_symbol),
  GetterParameterTypePolicy::RefMut =>
  {
   format!(
    "fn {}(&mut self)->&mut {}{{&mut self.{}}}",
    function_symbol, field_type, field_symbol
   )
  }
 }
}

fn generate_set_accessor(
 field_symbol: &FieldSymbol,
 field_type: &FieldType,
 fsp: &FieldSymbolPolicy,
 sptp: &SetterParameterTypePolicy
) -> String
{
 let fsp = match fsp
 {
  FieldSymbolPolicy::Default => &DEFAULT_SET_FIELD_SYMBOL_POLICY,
  _ => fsp
 };
 let function_symbol = generate_function_symbol(field_symbol, fsp);
 match sptp
 {
  SetterParameterTypePolicy::Value => format!("fn {}(&mut self,v:{}){{self.{}=v;}}", function_symbol, field_type, field_symbol),
  SetterParameterTypePolicy::RefClone =>
  {
   format!(
    "fn {}(&mut self,v:&{}){{self.{}.clone_from(v);}}",
    function_symbol, field_type, field_symbol
   )
  },
  SetterParameterTypePolicy::Into =>
  {
   format!(
    "fn {}<T:Into<{}>>(&mut self,v:T){{self.{}=v.into();}}",
    function_symbol, field_type, field_symbol
   )
  }
 }
}

fn generate_function_symbol(field_symbol: &FieldSymbol, field_symbol_policy: &FieldSymbolPolicy) -> String
{
 match field_symbol_policy
 {
  FieldSymbolPolicy::Prefix(prefix) => format!("{}_{}", prefix, field_symbol),
  FieldSymbolPolicy::Suffix(suffix) => format!("{}_{}", field_symbol, suffix),
  FieldSymbolPolicy::Fullname(fullname) => fullname.clone(),
  _ =>
  {
   panic!(
    "tia implementation bug: This message might be shown for crate users. But if you see, then report an issue please. #TIA-PANIC-2001"
   )
  }
 }
}
