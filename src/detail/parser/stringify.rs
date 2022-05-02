use super::*;

pub fn decode_expr(expr: &syn::Expr) -> String
{
 match expr
 {
  syn::Expr::Verbatim(v) => v.to_string(),
  _unsupported_expr =>
  {
   dbg!(_unsupported_expr);
   panic!("tia not implemented feature: Most of `Epxr` decording, please write PR or Issue if you want the feature. #TIA-PANIC-3008")
  }
 }
}

pub fn decode_type(ty: &syn::Type) -> String
{
 match ty
 {
  syn::Type::Array(ar) =>
  {
   let elem = decode_type(
    ar
     .elem
     .as_ref()
   );
   let len = decode_expr(&ar.len);
   format!("[{};{}]", elem, len)
  },
  syn::Type::BareFn(_bf) =>
  {
   panic!("tia not implemented feature: BareFn, please write PR or Issue if you want the feature. #TIA-PANIC-3007")
  },
  syn::Type::Group(_gr) => panic!("tia not implemented feature: Group, please write PR or Issue if you want the feature. #TIA-PANIC-3006"),
  syn::Type::ImplTrait(_it) =>
  {
   panic!("tia not implemented feature: ImplTrait, please write PR or Issue if you want the feature. #TIA-PANIC-3005")
  },
  syn::Type::Infer(_) => "_".into(),
  syn::Type::Macro(m) =>
  {
   let path = m
    .mac
    .path
    .segments
    .iter()
    .map(|s| {
     s.ident
      .to_string()
    })
    .collect::<Vec<_>>()
    .join(COLON_COLON_SEPARATOR);
   let delimiters = match m
    .mac
    .delimiter
   {
    syn::MacroDelimiter::Brace(_) => ("{", "}"),
    syn::MacroDelimiter::Bracket(_) => ("<", ">"),
    syn::MacroDelimiter::Paren(_) => ("(", ")")
   };
   format!(
    "{}!{}{}{}",
    path,
    delimiters.0,
    m.mac
     .tokens
     .to_string(),
    delimiters.1
   )
  },
  syn::Type::Never(_) => "!".into(),
  syn::Type::Paren(p) =>
  {
   format!(
    "({})",
    decode_type(
     p.elem
      .as_ref()
    )
   )
  },
  syn::Type::Path(type_path) =>
  {
   type_path
    .path
    .segments
    .iter()
    .map(|s| {
     format!(
      "{}{}",
      s.ident
       .to_string(),
      decode_type_path_arguments(&s.arguments)
     )
    })
    .collect::<Vec<_>>()
    .join(COLON_COLON_SEPARATOR)
  },
  syn::Type::Ptr(p) =>
  {
   format!(
    "*{}{}",
    p.mutability
     .map(|_| "mut ")
     .unwrap_or_default(),
    decode_type(
     p.elem
      .as_ref()
    )
   )
  },
  syn::Type::Reference(re) =>
  {
   let lifetime = re
    .lifetime
    .as_ref()
    .map(|lifetime| format!("{} ", lifetime.to_string()))
    .unwrap_or_default();
   let m = re
    .mutability
    .map(|_| "mut ".to_string())
    .unwrap_or_default();
   let elem = decode_type(
    &re
     .elem
     .as_ref()
   );
   format!("&{}{}{}", lifetime, m, elem)
  },
  syn::Type::Slice(sl) =>
  {
   let t = decode_type(
    sl
     .elem
     .as_ref()
   );
   format!("[{}]", t)
  },
  syn::Type::TraitObject(_ts) =>
  {
   panic!("tia not implemented feature: TraitObject, please write PR or Issue if you want the feature. #TIA-PANIC-3004")
  },
  syn::Type::Tuple(t) =>
  {
   format!(
    "({})",
    t.elems
     .iter()
     .map(|t| decode_type(t))
     .collect::<Vec<_>>()
     .join(COMMA)
   )
  },
  syn::Type::Verbatim(v) => v.to_string(),
  _unknown =>
  {
   eprintln!("[Warning] Tia detect unknown token stream part. #TIA-WARNING-3003");
   "".to_string()
  }
 }
}

pub fn decode_type_path_arguments(arguments: &syn::PathArguments) -> String
{
 match arguments
 {
  syn::PathArguments::AngleBracketed(bracket) =>
  {
   let body = bracket
    .args
    .iter()
    .map(|generic_argument| {
     match generic_argument
     {
      syn::GenericArgument::Type(ty) => decode_type(ty),
      syn::GenericArgument::Lifetime(lifetime) => lifetime.to_string(),
      syn::GenericArgument::Binding(_binding) =>
      {
       panic!("tia not implemented feature: Binding, please write PR or Issue if you want the feature. #TIA-PANIC-3002")
      },
      syn::GenericArgument::Constraint(_constraint) =>
      {
       panic!("tia not implemented feature: Constraint, please write PR or Issue if you want the feature. #TIA-PANIC-3001")
      },
      syn::GenericArgument::Const(_con) => "const ".into()
     }
    })
    .collect::<Vec<_>>()
    .join(NO_SEPARATOR);
   format!("<{}>", body)
  },
  syn::PathArguments::Parenthesized(parenthesized) =>
  {
   let i = parenthesized
    .inputs
    .iter()
    .map(|ty| decode_type(ty))
    .collect::<Vec<_>>()
    .join(COMMA);
   let o = decode_return_type(&parenthesized.output);
   format!("({}){}", i, o)
  },
  syn::PathArguments::None => String::default()
 }
}

pub fn decode_return_type(t: &syn::ReturnType) -> String
{
 match t
 {
  syn::ReturnType::Default => String::default(),
  syn::ReturnType::Type(_, ty) => format!("->{}", decode_type(ty.as_ref()))
 }
}
