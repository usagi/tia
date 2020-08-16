mod stringify;

use super::*;
use std::collections::HashSet;
use syn::parse::Parser;

const TIA: &str = "tia";
const COLON_COLON_SEPARATOR: &str = "::";
const COMMA: &str = ",";

#[derive(Debug)]
enum TiaParamToken
{
 TraitSymbol(String),
 DefaultDirective(String),
 CustomDirective
 {
  key:   String,
  value: FieldSymbolPolicy
 }
}

type TiaParamTokenStream = Vec<TiaParamToken>;
type TraitToAccessors = HashMap<TraitSymbol, HashSet<Accessor>>;

pub fn parse(i: syn::DeriveInput) -> Tia
{
 let target_type_symbol = i.ident.to_string();

 let root_ta = parse_root(&i);
 let trait_to_field_accessors = parse_fields(&i, &root_ta);

 Tia {
  target_type_symbol,
  trait_to_field_accessors
 }
}

fn parse_root(i: &syn::DeriveInput) -> TraitToAccessors
{
 match find_tia_attribute(&i.attrs)
 {
  Some(attribute) =>
  {
   let root_tia_params = parse_tia_params(&attribute);
   let root_ta = translate_tia_params(root_tia_params);
   root_ta
  },
  None => TraitToAccessors::default()
 }
}

fn get_field_symbol(field: &syn::Field) -> String
{
 field
  .ident
  .as_ref()
  .expect("tia implementation bug, maybe: failed to parse a field symbol. #TIA-PANIC-1017")
  .to_string()
}

fn parse_fields(i: &syn::DeriveInput, root_ta: &TraitToAccessors) -> TraitToFieldAccessors
{
 let mut ttfa = TraitToFieldAccessors::default();

 match &i.data
 {
  syn::Data::Struct(data_struct) =>
  {
   for field in &data_struct.fields
   {
    let field_symbol = get_field_symbol(field);
    let field_type = stringify::decode_type(&field.ty);
    let ta = match find_tia_attribute(&field.attrs)
    {
     Some(attribute) =>
     {
      let field_tia_token_stream = parse_tia_params(attribute);
      let field_ta = translate_tia_params(field_tia_token_stream);
      let mut ta = root_ta.clone();
      // ta (root.get, root.set)
      for (t, aa) in field_ta
      {
       let ta_aa = ta.entry(t).or_default();
       for a in aa
       {
        ta_aa.replace(a);
       }
      }
      ta
     },
     None => root_ta.clone()
    };
    for (t, a) in ta
    {
     ttfa.entry(t).or_default().insert(field_symbol.clone(), FieldParams {
      field_type: field_type.clone(),
      accessors:  a
     });
    }
   }
  },
  syn::Data::Enum(_data_enum) =>
  {
   panic!("tia not implemented feature: Enum, please write PR or Issue if you want the feature. #TIA-PANIC-1016")
  },
  syn::Data::Union(_data_union) =>
  {
   panic!("tia not implemented feature: Union, please write PR or Issue if you want the feature. #TIA-PANIC-1015")
  },
 }

 ttfa
}

fn translate_tia_params(tia_param_token_stream: TiaParamTokenStream) -> TraitToAccessors
{
 let mut ta = TraitToAccessors::default();

 let mut current_trait_symbol = String::default();
 for tia_param_token in tia_param_token_stream
 {
  match tia_param_token
  {
   TiaParamToken::TraitSymbol(trait_symbol) => current_trait_symbol = trait_symbol,
   TiaParamToken::DefaultDirective(ref key) =>
   {
    let accessor = match &key[..]
    {
     "gm" =>
     {
      Accessor::Getter {
       fsp: FieldSymbolPolicy::Default,
       ptp: GetterParameterTypePolicy::Move
      }
     },
     "g" =>
     {
      Accessor::Getter {
       fsp: FieldSymbolPolicy::Default,
       ptp: GetterParameterTypePolicy::Value
      }
     },
     "rg" =>
     {
      Accessor::Getter {
       fsp: FieldSymbolPolicy::Default,
       ptp: GetterParameterTypePolicy::Ref
      }
     },
     "rmg" =>
     {
      Accessor::Getter {
       fsp: FieldSymbolPolicy::Default,
       ptp: GetterParameterTypePolicy::RefMut
      }
     },
     "s" =>
     {
      Accessor::Setter {
       fsp: FieldSymbolPolicy::Default,
       ptp: SetterParameterTypePolicy::Value
      }
     },
     "rsc" =>
     {
      Accessor::Setter {
       fsp: FieldSymbolPolicy::Default,
       ptp: SetterParameterTypePolicy::RefClone
      }
     },
     "rsi" =>
     {
      Accessor::Setter {
       fsp: FieldSymbolPolicy::Default,
       ptp: SetterParameterTypePolicy::Into
      }
     },
     _ =>
     {
      panic!(
       "tia syntax error: Check around of directives, maybe you wrote an unsupported keyword or typo such as `&g`, `rms` or `brabrabra` \
        #TIA-PANIC-1014."
      )
     },
    };
    ta.entry(current_trait_symbol.clone()).or_default().replace(accessor);
   },
   TiaParamToken::CustomDirective {
    key,
    value
   } =>
   {
    let fsp = value;
    let accessor = match &key[..]
    {
     "g" =>
     {
      Accessor::Getter {
       fsp,
       ptp: GetterParameterTypePolicy::Value
      }
     },
     "rg" =>
     {
      Accessor::Getter {
       fsp,
       ptp: GetterParameterTypePolicy::Ref
      }
     },
     "rmg" =>
     {
      Accessor::Getter {
       fsp,
       ptp: GetterParameterTypePolicy::RefMut
      }
     },
     "s" =>
     {
      Accessor::Setter {
       fsp,
       ptp: SetterParameterTypePolicy::Value
      }
     },
     "rsc" =>
     {
      Accessor::Setter {
       fsp,
       ptp: SetterParameterTypePolicy::RefClone
      }
     },
     "rsi" =>
     {
      Accessor::Setter {
       fsp,
       ptp: SetterParameterTypePolicy::Into
      }
     },
     _ =>
     {
      panic!(
       r#"tia syntax error: Check around of directive, maybe you wrote an unsupported keyword or typo such as `&g="bad-symbol"`, `rms='bad_quote'` or `brabrabra`. #TIA-PANIC-1013"#
      )
     },
    };
    ta.entry(current_trait_symbol.clone()).or_default().replace(accessor);
   }
  }
 }

 ta
}

fn find_tia_attribute(attributes: &[syn::Attribute]) -> Option<&syn::Attribute>
{
 attributes.iter().find(|&a| {
  let attribute_path_segments = &a.path.segments;
  match attribute_path_segments.first()
  {
   Some(s) if s.ident == TIA => true,
   _ => false
  }
 })
}

fn parse_tia_params(attribute: &syn::Attribute) -> TiaParamTokenStream
{
 let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
 let tokens = attribute.tokens.clone();
 let tokens = parser
  .parse(tokens.into())
  .expect("tia syntax error: syn parse was failed. #TIA-PANIC-1012");

 match tokens.first()
 {
  // #[tia(x,y,...)] pattern
  Some(syn::Expr::Tuple(root)) =>
  {
   let mut tia_params: TiaParamTokenStream = vec![];
   for expr in root.elems.iter()
   {
    tia_params.append(&mut parse_tia_param_syn_expr(expr));
   }
   tia_params
  },

  // #[tia(x)] or #[tia(t:x)] pattern
  Some(syn::Expr::Paren(root)) => parse_tia_param_syn_expr(root.expr.as_ref()),
  // It is not an error that the tia attribute does not exists.
  _ => vec![]
 }
}

fn parse_tia_param_syn_expr(expr: &syn::Expr) -> TiaParamTokenStream
{
 let mut tia_params: TiaParamTokenStream = vec![];
 match expr
 {
  syn::Expr::Lit(e_lit) =>
  {
   if let syn::Lit::Str(a) = &e_lit.lit
   {
    let trait_symbol = TiaParamToken::TraitSymbol(a.value());
    tia_params.push(trait_symbol)
   }
  },
  syn::Expr::Path(e_path) => tia_params.push(parse_tia_param_syn_expr_path(e_path)),
  syn::Expr::Assign(e_assign) => tia_params.push(parse_tia_param_syn_expr_assign(e_assign)),
  syn::Expr::AssignOp(e_assign_op) => tia_params.push(parse_tia_param_syn_expr_assign_op(e_assign_op)),
  syn::Expr::Type(e_type) =>
  {
   let (p0, p1) = parse_tia_param_syn_expr_type(e_type);
   tia_params.push(p0);
   tia_params.push(p1);
  },
  _e_unsupported =>
  {
   dbg!(_e_unsupported);
   panic!(r#"tia syntax error; Check #[tia(...)] (tia proc-macro pattern) of struct|enum|union scope, maybe. #TIA-PANIC-1011"#)
  }
 };
 tia_params
}

fn parse_tia_param_syn_expr_type(e_type: &syn::ExprType) -> (TiaParamToken, TiaParamToken)
{
 (
  match e_type.expr.as_ref()
  {
   syn::Expr::Path(e_path) => TiaParamToken::TraitSymbol(e_path.path.segments.first().unwrap().ident.to_string()),
   _ => panic!(r#"tia syntax error; Check around of `XXX:` (trait symbol pattern), maybe. #TIA-PANIC-10010"#)
  },
  match e_type.ty.as_ref()
  {
   syn::Type::Path(t_path) => TiaParamToken::DefaultDirective(t_path.path.segments.first().unwrap().ident.to_string()),
   _ => panic!(r#"tia syntax error; Check around of `XXX: YYY` (first tia directive after a trait symbol pattern), maybe. #TIA-PANIC-1009"#)
  }
 )
}

fn parse_tia_param_syn_expr_path(e_path: &syn::ExprPath) -> TiaParamToken
{
 TiaParamToken::DefaultDirective(e_path.path.segments.first().unwrap().ident.to_string())
}

fn parse_tia_param_syn_expr_assign_op(e_assign_op: &syn::ExprAssignOp) -> TiaParamToken
{
 match e_assign_op.left.as_ref()
 {
  syn::Expr::Path(left_part) =>
  {
   let key = left_part
    .path
    .segments
    .first()
    .expect(r#"tia syntax error; Check around of `PPP=` (key part of a tia directive with string pattern), maybe. #TIA-PANIC-1008"#)
    .ident
    .to_string();

   match e_assign_op.right.as_ref()
   {
    syn::Expr::Lit(right_part) =>
    {
     match (&right_part.lit, e_assign_op.op)
     {
      (syn::Lit::Str(right_str), syn::BinOp::AddEq(_)) =>
      {
       TiaParamToken::CustomDirective {
        key,
        value: FieldSymbolPolicy::Suffix(right_str.value())
       }
      },
      (syn::Lit::Str(right_str), syn::BinOp::MulEq(_)) =>
      {
       TiaParamToken::CustomDirective {
        key,
        value: FieldSymbolPolicy::Fullname(right_str.value())
       }
      },
      _ =>
      {
       panic!(
        r#"tia syntax error; Check around of `"value"` (key part of a `key+="value"` or `key*="value"` pattern tia directive with string pattern), maybe. #TIA-PANIC-1007"#
       )
      },
     }
    },
    _ =>
    {
     panic!(
      r#"tia syntax error; Check around of `"value"` (value part of a `key+="value"` or `key*="value"` pattern tia directive with string pattern), maybe. #TIA-PANIC-1006"#
     )
    },
   }
  },
  _ =>
  {
   panic!(
    r#"tia syntax error; Check around of `key="value"` (key-value pair of tia directive with string pattern), maybe. #TIA-PANIC-1005"#
   )
  },
 }
}

fn parse_tia_param_syn_expr_assign(e_assign: &syn::ExprAssign) -> TiaParamToken
{
 match e_assign.left.as_ref()
 {
  syn::Expr::Path(left_part) =>
  {
   let key = left_part
    .path
    .segments
    .first()
    .expect(r#"tia syntax error; Check around of `PPP=` (key part of a tia directive with string pattern), maybe. #TIA-PANIC-1004"#)
    .ident
    .to_string();

   match e_assign.right.as_ref()
   {
    syn::Expr::Lit(right_part) =>
    {
     match &right_part.lit
     {
      syn::Lit::Str(right_str) =>
      {
       TiaParamToken::CustomDirective {
        key,
        value: FieldSymbolPolicy::Prefix(right_str.value())
       }
      },
      _ =>
      {
       panic!(
        r#"tia syntax error; Check around of `"value"` (key part of a `key="value"` pattern tia directive with string pattern), maybe. #TIA-PANIC-1003"#
       )
      },
     }
    },
    _ =>
    {
     panic!(
      r#"tia syntax error; Check around of `"value"` (value part of a `key="value"` pattern tia directive with string pattern), maybe. #TIA-PANIC-1002"#
     )
    },
   }
  },
  _ =>
  {
   panic!(
    r#"tia syntax error; Check around of `key="value"` (key-value pair of tia directive with string pattern), maybe. #TIA-PANIC-1001"#
   )
  },
 }
}
