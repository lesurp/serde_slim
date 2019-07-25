#![feature(custom_attribute)]

use serde_slim::serde_slim;

serde_slim! {
Serialized,
struct Foo {
    x: i32,
    #[serde(skip_serializing)]
    y: i32,
}

struct Bar {
    foo: Foo
}
}

// compiling is the test
fn main() {
    let f = Foo { x: 0, y: 1 };
    let sf = SerializedFoo { x: 0, };

    let b = Bar { foo: f };
    let sb = SerializedBar { foo: sf };
}
