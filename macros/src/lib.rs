// #![deny(warnings)]
#![allow(warnings)]

extern crate proc_macro;
extern crate rand;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro2;

use proc_macro2::Span;
use rand::Rng;
use syn::synom::Synom;
use syn::token::{Colon, Comma, Eq, Static};
use syn::{Expr, FnArg, Ident, Item, ItemFn, ReturnType, Stmt, Type, Visibility};

use proc_macro::TokenStream;

/// Attribute to declare the entry point of the program
///
/// **NOTE** This macro must be invoked once and must be invoked from an accessible module, ideally
/// from the root of the crate.
///
/// The specified function will be called by the reset handler *after* RAM has been initialized. In
/// the case of the `thumbv7em-none-eabihf` target the FPU will also be enabled before the function
/// is called.
///
/// The type of the specified function must be `fn() -> !` (never ending function)
///
/// # Examples
///
/// ``` no_run
/// # #![no_main]
/// # use cortex_m_rt_macros::entry;
/// #[entry]
/// fn main() -> ! {
///     loop {
///         /* .. */
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = syn::parse(input).expect("`#[entry]` must be applied to a function");

    // check the function signature
    assert!(
        f.constness.is_none()
            && f.vis == Visibility::Inherited
            && f.unsafety.is_none()
            && f.abi.is_none()
            && f.decl.inputs.is_empty()
            && f.decl.generics.params.is_empty()
            && f.decl.generics.where_clause.is_none()
            && f.decl.variadic.is_none()
            && match f.decl.output {
                ReturnType::Default => false,
                ReturnType::Type(_, ref ty) => match **ty {
                    Type::Never(_) => true,
                    _ => false,
                },
            },
        "`#[entry]` function must have signature `fn() -> !`"
    );

    assert_eq!(
        args.to_string(),
        "",
        "`entry` attribute must have no arguments"
    );

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let ident = f.ident;
    let block = f.block;

    quote!(
        #[export_name = "main"]
        #(#attrs)*
        pub fn #ident() -> ! #block
    ).into()
}

struct ExceptionArgs {
    first: Ident,
    second: Option<State>,
}

impl Synom for ExceptionArgs {
    named!(parse -> Self, do_parse!(
        first: syn!(Ident) >>
            second: option!(syn!(State)) >> (
                ExceptionArgs { first, second }
            )
    ));
}

struct State {
    _comma: Comma,
    _static: Static,
    ident: Ident,
    _colon: Colon,
    ty: Type,
    _eq: Eq,
    expr: Expr,
}

impl Synom for State {
    named!(parse -> Self, do_parse!(
        _comma: punct!(,) >>
            _static: syn!(Static) >>
            ident: syn!(Ident) >>
            _colon: punct!(:) >>
            ty: syn!(Type) >>
            _eq: punct!(=) >>
            expr: syn!(Expr) >> (
                State { _comma, _static, ident, _colon, ty, _eq, expr }
            )
    ));
}

/// Attribute to declare an exception handler
///
/// **NOTE** This macro must be invoked from an accessible module, ideally from the root of the
/// crate.
///
/// # Syntax
///
/// ```
/// # use cortex_m_rt_macros::exception;
/// #[exception]
/// fn SysTick() {
///     // ..
/// }
///
/// # fn main() {}
/// ```
///
/// where the name of the function must be one of:
///
/// - `DefaultHandler`
/// - `NonMaskableInt`
/// - `HardFault`
/// - `MemoryManagement` (a)
/// - `BusFault` (a)
/// - `UsageFault` (a)
/// - `SecureFault` (b)
/// - `SVCall`
/// - `DebugMonitor` (a)
/// - `PendSV`
/// - `SysTick`
///
/// (a) Not available on Cortex-M0 variants (`thumbv6m-none-eabi`)
///
/// (b) Only available on ARMv8-M
///
/// # Usage
///
/// `#[exception] fn HardFault(..` sets the hard fault handler. The handler must have signature
/// `fn(&ExceptionFrame) -> !`. This handler is not allowed to return as that can cause undefined
/// behavior.
///
/// `#[exception] fn DefaultHandler(..` sets the *default* handler. All exceptions which have not
/// been assigned a handler will be serviced by this handler. This handler must have signature
/// `fn(irqn: i16)`. `irqn` is the IRQ number (See CMSIS); `irqn` will be a negative number when the
/// handler is servicing a core exception; `irqn` will be a positive number when the handler is
/// servicing a device specific exception (interrupt).
///
/// `#[exception] fn Name(..` overrides the default handler for the exception with the given `Name`.
/// When overriding these other exception it's possible to add state to them by declaring `static
/// mut` variables at the beginning of the body of the function. These variables will be safe to
/// access from the function body.
///
/// Exception handlers can only be called by the hardware. Other parts of the program can't refer to
/// the exception handler much less invoke them as if they were functions.
///
/// # Examples
///
/// - Setting the `HardFault` handler
///
/// ```
/// # extern crate cortex_m_rt;
/// # extern crate cortex_m_rt_macros;
/// # use cortex_m_rt_macros::exception;
/// #[exception]
/// fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
///     // prints the exception frame as a panic message
///     panic!("{:#?}", ef);
/// }
///
/// # fn main() {}
/// ```
///
/// - Setting the default handler
///
/// ```
/// # use cortex_m_rt_macros::exception;
/// #[exception]
/// fn DefaultHandler(irqn: i16) {
///     println!("IRQn = {}", irqn);
/// }
///
/// # fn main() {}
/// ```
///
/// - Overriding the `SysTick` handler
///
/// ```
/// extern crate cortex_m_rt as rt;
///
/// use rt::exception;
///
/// #[exception]
/// fn SysTick() {
///     static mut COUNT: i32 = 0;
///
///     // `COUNT` is safe to access and has type `&mut i32`
///     *COUNT += 1;
///
///     println!("{}", COUNT);
/// }
///
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn exception(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = syn::parse(input).expect("`#[exception]` must be applied to a function");

    assert_eq!(
        args.to_string(),
        "",
        "`exception` attribute must have no arguments"
    );

    let ident = f.ident;

    enum Exception {
        DefaultHandler,
        HardFault,
        Other,
    }

    let ident_s = ident.to_string();
    let exn = match &*ident_s {
        "DefaultHandler" => Exception::DefaultHandler,
        "HardFault" => Exception::HardFault,
        // NOTE that at this point we don't check if the exception is available on the target (e.g.
        // MemoryManagement is not available on Cortex-M0)
        "NonMaskableInt" | "MemoryManagement" | "BusFault" | "UsageFault" | "SecureFault"
        | "SVCall" | "DebugMonitor" | "PendSV" | "SysTick" => Exception::Other,
        _ => panic!("{} is not a valid exception name", ident_s),
    };

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let block = f.block;
    let stmts = block.stmts;

    let mut rng = rand::thread_rng();
    let hash = (0..16)
        .map(|i| {
            if i == 0 || rng.gen() {
                ('a' as u8 + rng.gen::<u8>() % 25) as char
            } else {
                ('0' as u8 + rng.gen::<u8>() % 10) as char
            }
        }).collect::<String>();
    let hash = Ident::new(&hash, Span::call_site());

    match exn {
        Exception::DefaultHandler => {
            assert!(
                f.constness.is_none()
                    && f.vis == Visibility::Inherited
                    && f.unsafety.is_none()
                    && f.abi.is_none()
                    && f.decl.inputs.len() == 1
                    && f.decl.generics.params.is_empty()
                    && f.decl.generics.where_clause.is_none()
                    && f.decl.variadic.is_none()
                    && match f.decl.output {
                        ReturnType::Default => true,
                        ReturnType::Type(_, ref ty) => match **ty {
                            Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                            _ => false,
                        },
                    },
                "`#DefaultHandler` function must have signature `fn(i16)`"
            );

            let arg = match f.decl.inputs[0] {
                FnArg::Captured(ref arg) => arg,
                _ => unreachable!(),
            };

            quote!(
                #[export_name = #ident_s]
                #(#attrs)*
                pub extern "C" fn #hash() {
                    extern crate core;

                    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;

                    let #arg = unsafe { core::ptr::read(SCB_ICSR) as u8 as i16 - 16 };

                    #(#stmts)*
                }
            ).into()
        }
        Exception::HardFault => {
            assert!(
                f.constness.is_none()
                    && f.vis == Visibility::Inherited
                    && f.unsafety.is_none()
                    && f.abi.is_none()
                    && f.decl.inputs.len() == 1
                    && match f.decl.inputs[0] {
                        FnArg::Captured(ref arg) => match arg.ty {
                            Type::Reference(ref r) => {
                                r.lifetime.is_none() && r.mutability.is_none()
                            }
                            _ => false,
                        },
                        _ => false,
                    }
                    && f.decl.generics.params.is_empty()
                    && f.decl.generics.where_clause.is_none()
                    && f.decl.variadic.is_none()
                    && match f.decl.output {
                        ReturnType::Default => false,
                        ReturnType::Type(_, ref ty) => match **ty {
                            Type::Never(_) => true,
                            _ => false,
                        },
                    },
                "`#[exception(HardFault)]` function must have signature `fn(&ExceptionFrame) -> !`"
            );

            let arg = match f.decl.inputs[0] {
                FnArg::Captured(ref arg) => arg,
                _ => unreachable!(),
            };

            let pat = &arg.pat;

            quote!(
                #[export_name = "UserHardFault"]
                #(#attrs)*
                pub extern "C" fn #hash(#arg) -> ! {
                    extern crate cortex_m_rt;

                    // further type check of the input argument
                    let #pat: &cortex_m_rt::ExceptionFrame = #pat;

                    #(#stmts)*
                }
            ).into()
        }
        Exception::Other => {
            assert!(
                f.constness.is_none()
                    && f.vis == Visibility::Inherited
                    && f.unsafety.is_none()
                    && f.abi.is_none()
                    && f.decl.inputs.is_empty()
                    && f.decl.generics.params.is_empty()
                    && f.decl.generics.where_clause.is_none()
                    && f.decl.variadic.is_none()
                    && match f.decl.output {
                        ReturnType::Default => true,
                        ReturnType::Type(_, ref ty) => match **ty {
                            Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                            _ => false,
                        },
                    },
                "`#[exception]` functions other than `DefaultHandler` and `HardFault` must \
                 have signature `fn()`"
            );

            // Collect all the `static mut` at the beginning of the function body. We'll make them
            // safe
            let mut istmts = stmts.into_iter();

            let mut statics = vec![];
            let mut stmts = vec![];
            while let Some(stmt) = istmts.next() {
                match stmt {
                    Stmt::Item(Item::Static(var)) => if var.mutability.is_some() {
                        statics.push(var);
                    } else {
                        stmts.push(Stmt::Item(Item::Static(var)));
                    },
                    _ => {
                        stmts.push(stmt);
                        break;
                    }
                }
            }

            stmts.extend(istmts);

            let vars = statics
                .into_iter()
                .map(|var| {
                    let ident = var.ident;
                    // `let` can't shadow a `static mut` so we must give the `static` a different
                    // name. We'll create a new name by appending an underscore to the original name
                    // of the `static`.
                    let mut ident_ = ident.to_string();
                    ident_.push('_');
                    let ident_ = Ident::new(&ident_, Span::call_site());
                    let ty = var.ty;
                    let expr = var.expr;

                    quote!(
                    static mut #ident_: #ty = #expr;
                    #[allow(non_snake_case, unsafe_code)]
                    let #ident: &mut #ty = unsafe { &mut #ident_ };
                )
                }).collect::<Vec<_>>();

            quote!(
                #[export_name = #ident_s]
                #(#attrs)*
                pub fn #hash() {
                    extern crate cortex_m_rt;

                    // check that this exception actually exists
                    cortex_m_rt::Exception::#ident;

                    #(#vars)*

                    #(#stmts)*
                }
            ).into()
        }
    }
}

/// Attribute to mark which function will be called at the beginning of the reset handler.
///
/// The function must have the signature of `unsafe fn()`.
///
/// The function passed will be called before static variables are initialized. Any access of static
/// variables will result in undefined behavior.
///
/// # Examples
///
/// ```
/// # use cortex_m_rt_macros::pre_init;
/// #[pre_init]
/// unsafe fn before_main() {
///     // do something here
/// }
///
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn pre_init(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = syn::parse(input).expect("`#[pre_init]` must be applied to a function");

    // check the function signature
    assert!(
        f.constness.is_none()
            && f.vis == Visibility::Inherited
            && f.unsafety.is_some()
            && f.abi.is_none()
            && f.decl.inputs.is_empty()
            && f.decl.generics.params.is_empty()
            && f.decl.generics.where_clause.is_none()
            && f.decl.variadic.is_none()
            && match f.decl.output {
                ReturnType::Default => true,
                ReturnType::Type(_, ref ty) => match **ty {
                    Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                    _ => false,
                },
            },
        "`#[pre_init]` function must have signature `unsafe fn()`"
    );

    assert_eq!(
        args.to_string(),
        "",
        "`pre_init` attribute must have no arguments"
    );

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let ident = f.ident;
    let block = f.block;

    quote!(
        #[export_name = "__pre_init"]
        #(#attrs)*
        pub unsafe fn #ident() #block
    ).into()
}
