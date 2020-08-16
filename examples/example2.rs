use tia::Tia; // use

#[derive(Tia, Debug, Default)] // derive
#[tia(rg, s)] // <-- tia directives, for all fields
struct MyStruct
{
 #[tia(rmg)] // <-- #[tia(rg, s)] + #[tia(rmg)] => #[tia(rmg, s)]
 foo: i32,
 #[tia(rsi)] // <-- #[tia(rg, s)] + #[tia(rsi)] => #[tia(rg, rsi)]
 bar: String,

 baz: f64, // <-- #[tia(rg, s)]

 #[tia(g)] // <-- #[tia(rg, s)] + #[tia(g)] => #[tia(g, s)] !! NOTE: Could be use for Copy-ables such as u8, but g pattern could not be use non-Copy-ables such as Vec<u8>
 brabrabra: u8,

 #[tia(gm)] // <-- #[tia(rg, s)] + #[tia(g)] => #[tia(gm, s)] !! WARNING: Could be move any types, but gm pattern will drop self
 hogefuga: Vec<u8>
}

fn main()
{
 let mut mys = MyStruct::default();

 // rmg; reference-mut-getter
 // with per-field level directive overwriting.
 {
  let foo = mys.get_foo(); // <-- &mut i32
  *foo = 42;
  dbg!(&foo);
  dbg!(&mys);
 }

 // rsi: reference-setter-into
 // with per-field level directive overwriting.
 {
  let a: &str = "Hello, ";
  let b: String = String::from("tia.");
  let c: &String = &b;

  mys.set_bar(a); // &str
  println!("a: mys.bar = {}", mys.get_bar());

  mys.set_bar(b.clone()); // String; This effect move, thus the example is a -> c -> b
  println!("b: mys.bar = {}", mys.get_bar());

  mys.set_bar(c); // &String
  println!("c: mys.bar = {}", mys.get_bar());
 }

 let x = mys.get_brabrabra(); // it will be Copy, mys will live
 dbg!(x, &mys);

 let y = mys.get_hogefuga(); // gm, get-move accessor will be drop mys
 dbg!(y);
 // mys was dropped, it could not be compile.
 //dbg!(mys)
}