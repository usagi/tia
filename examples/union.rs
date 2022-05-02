use tia::Tia;

#[derive(Tia)]
union U
{
 #[tia(g, s)]
 u: u128,
 f: f64
}

fn main()
{
 let mut p = U {
  u: 10u128
 };

 unsafe {
  let v = p.get_u();
  println!("initial value, p.get_u(), maybe {} eq {}", v, 10u128);
 }

 p.set_u(1u128);

 unsafe {
  let v = p.get_u();
  println!("second value, p.set() -> p.get_u(), maybe {} eq {}", v, 1u128);
 }

 p = U {
  f: 1.0f64
 };

 unsafe {
  let v = p.get_u();
  println!("third value, p = U{{f:1.0f64}} -> p.get_u() {} ne {}", v, 1u128); // maybe 4607182418800017408 in x86_64
  println!("third value, p.f {} eq {}", p.f, 1f64);
 }

 print!("Complete! This is the example version designed for human-eyes, if you want the testing version then read tests/test.rs.")
}
