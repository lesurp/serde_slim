#![feature(custom_attribute)]

mod baz {
    use serde_slim::serde_slim;
    serde_slim! {
    Serialized,
    pub struct Foo {
        pub x: i32,
        #[serde(skip_serializing)]
        y: i32,
    }

    pub struct Bar {
        pub foo: Foo
    }
    }

    impl Foo {
        pub fn new(x: i32, y: i32) -> Self {
            Foo { x, y }
        }
    }
}

// compiling is the test
fn main() {
    let f = baz::Foo::new(0, 1);
    let sf = baz::SerializedFoo { x: 0 };

    let b = baz::Bar { foo: f };
    let sb = baz::SerializedBar { foo: sf };
}
