<details>
  <summary>Rust API Guidelines Checklist</summary>
  
- **Naming** _(crate aligns with Rust naming conventions)_
    - [ ] Casing conforms to RFC 430 ([C-CASE](https://rust-lang.github.io/api-guidelines/naming.html#c-case))
    - [ ] Ad-hoc conversions follow `as_`, `to_`, `into_` conventions ([C-CONV](https://rust-lang.github.io/api-guidelines/naming.html#c-conv))
    - [ ] Getter names follow Rust convention ([C-GETTER](https://rust-lang.github.io/api-guidelines/naming.html#c-getter))
    - [ ] Methods on collections that produce iterators follow `iter`, `iter_mut`, `into_iter` ([C-ITER](https://rust-lang.github.io/api-guidelines/naming.html#c-iter))
    - [ ] Iterator type names match the methods that produce them ([C-ITER-TY](https://rust-lang.github.io/api-guidelines/naming.html#c-iter-ty))
    - [ ] Feature names are free of placeholder words ([C-FEATURE](https://rust-lang.github.io/api-guidelines/naming.html#c-feature))
    - [ ] Names use a consistent word order ([C-WORD-ORDER](https://rust-lang.github.io/api-guidelines/naming.html#c-word-order))
- **Interoperability** _(crate interacts nicely with other library functionality)_
    - [ ] Types eagerly implement common traits ([C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-common-traits))
        - `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`,
          `Display`, `Default`
    - [ ] Conversions use the standard traits `From`, `AsRef`, `AsMut` ([C-CONV-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-conv-traits))
    - [ ] Collections implement `FromIterator` and `Extend` ([C-COLLECT](https://rust-lang.github.io/api-guidelines/interoperability.html#c-collect))
    - [ ] Data structures implement Serde's `Serialize`, `Deserialize` ([C-SERDE](https://rust-lang.github.io/api-guidelines/interoperability.html#c-serde))
    - [ ] Types are `Send` and `Sync` where possible ([C-SEND-SYNC](https://rust-lang.github.io/api-guidelines/interoperability.html#c-send-sync))
    - [ ] Error types are meaningful and well-behaved ([C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#c-good-err))
    - [ ] Binary number types provide `Hex`, `Octal`, `Binary` formatting ([C-NUM-FMT](https://rust-lang.github.io/api-guidelines/interoperability.html#c-num-fmt))
    - [ ] Generic reader/writer functions take `R: Read` and `W: Write` by value ([C-RW-VALUE](https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value))
- **Macros** _(crate presents well-behaved macros)_
    - [ ] Input syntax is evocative of the output ([C-EVOCATIVE](https://rust-lang.github.io/api-guidelines/macros.html#c-evocative))
    - [ ] Macros compose well with attributes ([C-MACRO-ATTR](https://rust-lang.github.io/api-guidelines/macros.html#c-macro-attr))
    - [ ] Item macros work anywhere that items are allowed ([C-ANYWHERE](https://rust-lang.github.io/api-guidelines/macros.html#c-anywhere))
    - [ ] Item macros support visibility specifiers ([C-MACRO-VIS](https://rust-lang.github.io/api-guidelines/macros.html#c-macro-vis))
    - [ ] Type fragments are flexible ([C-MACRO-TY](https://rust-lang.github.io/api-guidelines/macros.html#c-macro-ty))
- **Documentation** _(crate is abundantly documented)_
    - [ ] Crate level docs are thorough and include examples ([C-CRATE-DOC](https://rust-lang.github.io/api-guidelines/documentation.html#c-crate-doc))
    - [ ] All items have a rustdoc example ([C-EXAMPLE](https://rust-lang.github.io/api-guidelines/documentation.html#c-example))
    - [ ] Examples use `?`, not `try!`, not `unwrap` ([C-QUESTION-MARK](https://rust-lang.github.io/api-guidelines/documentation.html#c-question-mark))
    - [ ] Function docs include error, panic, and safety considerations ([C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure))
    - [ ] Prose contains hyperlinks to relevant things ([C-LINK](https://rust-lang.github.io/api-guidelines/documentation.html#c-link))
    - [ ] Cargo.toml includes all common metadata ([C-METADATA](https://rust-lang.github.io/api-guidelines/documentation.html#c-metadata))
        - authors, description, license, homepage, documentation, repository,
          keywords, categories
    - [ ] Release notes document all significant changes ([C-RELNOTES](https://rust-lang.github.io/api-guidelines/documentation.html#c-relnotes))
    - [ ] Rustdoc does not show unhelpful implementation details ([C-HIDDEN](https://rust-lang.github.io/api-guidelines/documentation.html#c-hidden))
- **Predictability** _(crate enables legible code that acts how it looks)_
    - [ ] Smart pointers do not add inherent methods ([C-SMART-PTR](https://rust-lang.github.io/api-guidelines/documentation.html#c-smart-ptr))
    - [ ] Conversions live on the most specific type involved ([C-CONV-SPECIFIC](https://rust-lang.github.io/api-guidelines/documentation.html#c-cow-specific))
    - [ ] Functions with a clear receiver are methods ([C-METHOD](https://rust-lang.github.io/api-guidelines/documentation.html#c-method))
    - [ ] Functions do not take out-parameters ([C-NO-OUT](https://rust-lang.github.io/api-guidelines/documentation.html#c-no-out))
    - [ ] Operator overloads are unsurprising ([C-OVERLOAD](https://rust-lang.github.io/api-guidelines/documentation.html#c-overload))
    - [ ] Only smart pointers implement `Deref` and `DerefMut` ([C-DEREF](https://rust-lang.github.io/api-guidelines/documentation.html#c-deref))
    - [ ] Constructors are static, inherent methods ([C-CTOR](https://rust-lang.github.io/api-guidelines/documentation.html#c-ctor))
- **Flexibility** _(crate supports diverse real-world use cases)_
    - [ ] Functions expose intermediate results to avoid duplicate work ([C-INTERMEDIATE](https://rust-lang.github.io/api-guidelines/documentation.html#c-intermediate))
    - [ ] Caller decides where to copy and place data ([C-CALLER-CONTROL](https://rust-lang.github.io/api-guidelines/documentation.html#c-caller-control))
    - [ ] Functions minimize assumptions about parameters by using generics ([C-GENERIC](https://rust-lang.github.io/api-guidelines/documentation.html#c-generic))
    - [ ] Traits are object-safe if they may be useful as a trait object ([C-OBJECT](https://rust-lang.github.io/api-guidelines/documentation.html#c-object))
- **Type safety** _(crate leverages the type system effectively)_
    - [ ] Newtypes provide static distinctions ([C-NEWTYPE](https://rust-lang.github.io/api-guidelines/documentation.html#c-newtype))
    - [ ] Arguments convey meaning through types, not `bool` or `Option` ([C-CUSTOM-TYPE](https://rust-lang.github.io/api-guidelines/documentation.html#c-custom-type))
    - [ ] Types for a set of flags are `bitflags`, not enums ([C-BITFLAG](https://rust-lang.github.io/api-guidelines/documentation.html#c-bitflags))
    - [ ] Builders enable construction of complex values ([C-BUILDER](https://rust-lang.github.io/api-guidelines/documentation.html#c-builder))
- **Dependability** _(crate is unlikely to do the wrong thing)_
    - [ ] Functions validate their arguments ([C-VALIDATE](https://rust-lang.github.io/api-guidelines/documentation.html#c-validate))
    - [ ] Destructors never fail ([C-DTOR-FAIL](https://rust-lang.github.io/api-guidelines/documentation.html#c-dtor-fail))
    - [ ] Destructors that may block have alternatives ([C-DTOR-BLOCK](https://rust-lang.github.io/api-guidelines/documentation.html#c-dtor-block))
- **Debuggability** _(crate is conducive to easy debugging)_
    - [ ] All public types implement `Debug` ([C-DEBUG](https://rust-lang.github.io/api-guidelines/documentation.html#c-debug))
    - [ ] `Debug` representation is never empty ([C-DEBUG-NONEMPTY](https://rust-lang.github.io/api-guidelines/documentation.html#c-debug-nonempty))
- **Future proofing** _(crate is free to improve without breaking users' code)_
    - [ ] Sealed traits protect against downstream implementations ([C-SEALED](https://rust-lang.github.io/api-guidelines/documentation.html#c-sealed))
    - [ ] Structs have private fields ([C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/documentation.html#c-struct-private))
    - [ ] Newtypes encapsulate implementation details ([C-NEWTYPE-HIDE](https://rust-lang.github.io/api-guidelines/documentation.html#c-newtype-hide))
    - [ ] Data structures do not duplicate derived trait bounds ([C-STRUCT-BOUNDS](https://rust-lang.github.io/api-guidelines/documentation.html#c-struct-bounds))
- **Necessities** _(to whom they matter, they really matter)_
    - [ ] Public dependencies of a stable crate are stable ([C-STABLE](https://rust-lang.github.io/api-guidelines/documentation.html#c-stable))
    - [ ] Crate and its dependencies have a permissive license ([C-PERMISSIVE](https://rust-lang.github.io/api-guidelines/documentation.html#c-permissive))

</details>

