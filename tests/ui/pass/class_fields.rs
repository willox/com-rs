#[cfg(feature = "production")]
com::interfaces! {
    #[uuid("12345678-1234-1234-1234-12345678ABCD")]
    pub unsafe interface ISomething: com::interfaces::iunknown::IUnknown {}
}

#[cfg(feature = "production")]
com::class! {
    pub class ClassOfZero: ISomething {
    }

    impl ISomething for SomeClass {}
}

#[cfg(feature = "production")]
com::class! {
    pub class ClassOfOne: ISomething {
        one: u32,
    }

    impl ISomething for SomeClass {}
}

#[cfg(feature = "production")]
com::class! {
    pub class ClassOfTwo: ISomething {
        one: u32,
        two: u32
    }

    impl ISomething for SomeClass {}
}

fn main() {}
