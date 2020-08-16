use tia::Tia;

trait TestGetter
{
 fn get_x(&self) -> i32;
 fn ref_get_y(&self) -> &String;
 fn z_ref_get(&self) -> &String;
 fn get_w(&self) -> u8;
}

trait TestSetter
{
 fn set_x(&mut self, v: i32);
 fn set_clone_y(&mut self, v: &String);
 fn z_set_into<T: Into<String>>(&mut self, v: T);
 fn set_w(&mut self, v: u8);
}

#[derive(Tia, Default, Debug)]
#[tia("TestGetter", g, "TestSetter", s)]
struct S
{
 x: i32,
 #[tia("TestGetter", rg = "ref_get", "TestSetter", rsc = "set_clone")]
 y: String,
 #[tia("TestGetter",rg+="ref_get", "TestSetter",rsi+="set_into")]
 z: String,
 #[tia("",rmg*="baz")]
 w: u8
}

#[derive(Tia)]
union U
{
 #[tia(g, s)]
 u: u128,
 f: f64
}

#[test]
fn r#struct()
{
 let mut p = S::default();

 p.set_x(123i32);
 assert_eq!(p.get_x(), 123i32);

 let y = String::from("foobar");
 p.set_clone_y(&y);
 assert_eq!(p.ref_get_y(), &y);

 let z = "hogefuga";
 p.z_set_into(z);
 assert_eq!(p.z_ref_get(), z);

 let w2 = p.baz();
 *w2 = 123u8;
 assert_eq!(p.get_w(), 123u8);
}

#[test]
fn r#union()
{
 let mut p = U {
  u: 10u128
 };

 unsafe {
  let v = p.get_u();
  assert_eq!(v, 10u128);
 }

 p.set_u(1u128);

 unsafe {
  let v = p.get_u();
  assert_eq!(v, 1u128);
 }

 p = U {
  f: 1.0f64
 };

 unsafe {
  let v = p.get_u();
  assert_ne!(v, 1u128); // maybe 4607182418800017408 in x86_64
  assert_eq!(p.f, 1f64);
 }
}
