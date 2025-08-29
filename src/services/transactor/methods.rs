macro_rules! api_methods {
    ($($Variant:ident: $kebab:literal, $camel:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Method { $($Variant),+ }

        impl Method {
            pub const fn kebab(self) -> &'static str {
                match self {
                    $( Self::$Variant => $kebab ),+
                }
            }

            pub const fn camel(self) -> &'static str {
                match self {
                    $( Self::$Variant => $camel ),+
                }
            }
        }
    };
}

api_methods!(
    Account: "account", "account",
    FindAll: "find-all", "findAll",
    EnsurePerson: "ensure-person", "ensurePerson",
    Tx: "tx", "tx",
    Request: "request", "domainRequest",
    Event: "event", "event",
    Ping: "ping", "ping",
    Hello: "hello", "hello",
);
