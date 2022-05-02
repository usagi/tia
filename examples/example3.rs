use tia::Tia;

trait FooGettable<T>
{
 fn get_foo(&self) -> T;
}
trait Fruit
{
 fn get_bar(&self) -> &String;
}
trait Sushi
{
 fn tuna(&self) -> u8;
 fn avocado(&mut self, v: u8);
}

// include!(".tia/MyStruct.rs");
#[derive(Tia, Debug, Default)] // derive
struct MyStruct
{
 #[tia(s, "FooGettable<i32>", g)]
 foo: i32,
 #[tia("Fruit", rg, "", rsi)]
 bar: String,
 #[tia("Sushi",g*="tuna",s*="avocado")] // <- `g` and `s`: Sushi trait
 baz: u8
}

/// Build ok = Test ok
fn main()
{
 let mut mys = MyStruct::default();
 mys.set_foo(123);
 mys.set_bar("meow");
 let foo_gettable = &mys as &dyn FooGettable<i32>;
 let fruit = &mys as &dyn Fruit;
 println!("{}, {}", foo_gettable.get_foo(), fruit.get_bar());
 let sushi = &mut mys as &mut dyn Sushi;
 sushi.avocado(32);
 println!("{}", sushi.tuna());
}
