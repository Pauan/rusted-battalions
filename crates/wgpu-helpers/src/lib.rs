#![deny(warnings)]

extern crate proc_macro;
use proc_macro2::{Ident, Span, TokenStream};

use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::parse::{ParseStream, Parse};
use syn::{
    parse_macro_input, Error, Data, DeriveInput, Expr, Lit, Type,
    Attribute, LitInt, Token, Result,
};

use std::fmt::{Display, Formatter};


enum LayoutAttr {
    /// #[layout(step_mode = Instance)]
    StepMode(Span, Ident),

    /// #[layout(location = 0)]
    #[allow(dead_code)]
    Location(Span, u32),

    /// #[layout(format = Sint8x4)]
    Format(Span, Ident),

    /// #[layout(norm)]
    Norm(Span),
}

impl Parse for LayoutAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "step_mode" {
            input.parse::<Token![=]>()?;

            let step_mode: Ident = input.parse()?;

            Ok(Self::StepMode(ident.span(), step_mode))

        } else if ident == "location" {
            input.parse::<Token![=]>()?;

            let location: LitInt = input.parse()?;
            let location = location.base10_parse()?;

            Ok(Self::Location(ident.span(), location))

        } else if ident == "format" {
            input.parse::<Token![=]>()?;

            let format: Ident = input.parse()?;

            Ok(Self::Format(ident.span(), format))

        } else if ident == "norm" {
            Ok(Self::Norm(ident.span()))

        } else {
            Err(Error::new(ident.span(), "must be step_mode, location, format, or norm"))
        }
    }
}


struct Layout {
    attrs: Vec<LayoutAttr>,
}

impl Parse for Layout {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Punctuated::<LayoutAttr, Token![,]>::parse_terminated(input)?;

        Ok(Self { attrs: attrs.into_iter().collect() })
    }
}


fn parse_attrs(attrs: &[Attribute]) -> Result<Vec<LayoutAttr>> {
    let mut output = vec![];

    for attr in attrs {
        if attr.path().is_ident("layout") {
            let meta = attr.meta.require_list()?;

            let layout: Layout = meta.parse_args()?;

            output.extend(layout.attrs);
        }
    }

    Ok(output)
}


enum Primitive {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
    F64,
}

impl Primitive {
    fn from_ident(ident: &Ident) -> Option<Self> {
        if ident == "u8" {
            Some(Self::U8)

        } else if ident == "i8" {
            Some(Self::I8)

        } else if ident == "u16" {
            Some(Self::U16)

        } else if ident == "i16" {
            Some(Self::I16)

        } else if ident == "u32" {
            Some(Self::U32)

        } else if ident == "i32" {
            Some(Self::I32)

        } else if ident == "f32" {
            Some(Self::F32)

        } else if ident == "f64" {
            Some(Self::F64)

        } else {
            None
        }
    }

    fn to_wgpu(&self, span: Span, len: u32, norm: bool) -> Result<Ident> {
        let wgpu_name = if norm {
            match self {
                Primitive::U8 => "Unorm8",
                Primitive::I8 => "Snorm8",
                Primitive::U16 => "Unorm16",
                Primitive::I16 => "Snorm16",
                x => Err(Error::new(span, format!("norm cannot be used with type {}", x)))?,
            }

        } else {
            match self {
                Primitive::U8 => "Uint8",
                Primitive::I8 => "Sint8",
                Primitive::U16 => "Uint16",
                Primitive::I16 => "Sint16",
                Primitive::U32 => "Uint32",
                Primitive::I32 => "Sint32",
                Primitive::F32 => "Float32",
                Primitive::F64 => "Float64",
            }
        };

        match self {
            Primitive::U8 |
            Primitive::I8 |
            Primitive::U16 |
            Primitive::I16 => match len {
                2 | 4 => {
                    Ok(format_ident!("{}x{}", span = span, wgpu_name, len))
                },
                _ => {
                    Err(Error::new(span, "array length must be 2 or 4"))
                },
            },

            Primitive::U32 |
            Primitive::I32 |
            Primitive::F32 |
            Primitive::F64 => match len {
                1 => {
                    Ok(format_ident!("{}", span = span, wgpu_name))
                },
                2 | 3 | 4 => {
                    Ok(format_ident!("{}x{}", span = span, wgpu_name, len))
                },
                _ => {
                    Err(Error::new(span, "array length must be 1, 2, 3, or 4"))
                },
            },
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::U8 => "u8",
            Self::I8 => "i8",
            Self::U16 => "u16",
            Self::I16 => "i16",
            Self::U32 => "u32",
            Self::I32 => "i32",
            Self::F32 => "f32",
            Self::F64 => "f64",
        })
    }
}


fn parse_len(expr: &Expr) -> Result<u32> {
    if let Expr::Lit(lit) = expr {
        if let Lit::Int(int) = &lit.lit {
            let int: u32 = int.base10_parse()?;
            return Ok(int);
        }
    }

    Err(Error::new_spanned(expr, "array length must be a number"))
}


fn parse_primitive(ty: &Type) -> Result<Primitive> {
    match ty {
        Type::Path(path) => {
            let ident = path.path.require_ident()?;

            if let Some(primitive) = Primitive::from_ident(ident) {
                return Ok(primitive);
            }
        },

        Type::Group(group) => {
            return parse_primitive(&group.elem);
        },

        Type::Paren(paren) => {
            return parse_primitive(&paren.elem);
        },

        _ => {},
    }

    Err(Error::new_spanned(ty, "type cannot be converted into a wgpu::VertexFormat"))
}


fn parse_type(ty: &Type, norm: bool) -> Result<Ident> {
    match ty {
        Type::Array(array) => {
            let span = array.span();

            let len = parse_len(&array.len)?;

            let primitive = parse_primitive(&array.elem)?;

            Ok(primitive.to_wgpu(span, len, norm)?)
        },

        Type::Group(group) => {
            parse_type(&group.elem, norm)
        },

        Type::Paren(paren) => {
            parse_type(&paren.elem, norm)
        },

        ty => {
            let primitive = parse_primitive(ty)?;

            primitive.to_wgpu(ty.span(), 1, norm)
        },
    }
}


fn parse(input: DeriveInput) -> Result<TokenStream> {
    let name = input.ident;
    let vis = input.vis;

    let mut step_mode = format_ident!("Vertex");
    let mut location = 0;
    let mut attributes: Vec<TokenStream> = vec![];

    for attr in parse_attrs(&input.attrs)? {
        match attr {
            LayoutAttr::StepMode(_, ident) => {
                step_mode = ident;
            },
            LayoutAttr::Location(_, l) => {
                location = l;
            },
            LayoutAttr::Format(span, _) => {
                Err(Error::new(span, "format must be used on a field, not on the struct"))?
            },
            LayoutAttr::Norm(span) => {
                Err(Error::new(span, "norm must be used on a field, not on the struct"))?
            },
        }
    }

    match input.data {
        Data::Struct(data) => {
            for field in data.fields.iter() {
                let mut format = None;
                let mut norm = false;

                for attr in parse_attrs(&field.attrs)? {
                    match attr {
                        LayoutAttr::StepMode(span, _) => {
                            Err(Error::new(span, "step_mode must be used on the struct, not a field"))?;
                        },
                        LayoutAttr::Location(_, l) => {
                            location = l;
                        },
                        LayoutAttr::Format(_, f) => {
                            format = Some(f);
                        },
                        LayoutAttr::Norm(_) => {
                            norm = true;
                        },
                    }
                }

                let ty = if let Some(format) = format {
                    format

                } else {
                    parse_type(&field.ty, norm)?
                };

                let span = field.ty.span();

                attributes.push(quote_spanned!( span => #location => #ty ));

                location += 1;
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            Err(Error::new(Span::call_site(), "VertexLayout can only be used with structs"))?;
        },
    }

    let output = quote! {
        impl #name {
            #vis const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
                array_stride: ::std::mem::size_of::<#name>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::#step_mode,
                attributes: &wgpu::vertex_attr_array![#(#attributes),*],
            };
        }
    };

    Ok(TokenStream::from(output))
}


#[proc_macro_derive(VertexLayout, attributes(layout))]
pub fn vertex_layout(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    parse(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
