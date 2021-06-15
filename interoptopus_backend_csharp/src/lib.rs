use interoptopus::lang::c::{CType, CompositeType, Constant, ConstantValue, EnumType, Field, FnPointerType, Function, OpaqueType, Parameter, PrimitiveType, PrimitiveValue, Variant, Documentation};
use interoptopus::util::safe_name;
use interoptopus::writer::IndentWriter;
use interoptopus::{Error, Library};
use interoptopus::patterns::TypePattern;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_header_comment: String,
    pub namespace: String,
    pub class: String,
    pub dll_name: String,
    pub strip_from_fn: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file_header_comment: "// Automatically generated by Interoptopus.".to_string(),
            namespace: "Your.Namespace".to_string(),
            class: "Interop".to_string(),
            dll_name: "library".to_string(),
            strip_from_fn: None,
        }
    }
}

pub struct Generator {
    config: Config,
    library: Library,
}

impl Generator {
    pub fn new(config: Config, library: Library) -> Self {
        Self { config, library }
    }
}

pub trait Interop {
    /// Returns the user config.
    fn config(&self) -> &Config;

    /// Returns the library to produce bindings for.
    fn library(&self) -> &Library;

    /// Converts a primitive (Rust) type to a native C# type name, e.g., `f32` to `float`.
    fn type_primitive_to_typename(&self, x: &PrimitiveType) -> String {
        match x {
            PrimitiveType::Void => "void".to_string(),
            PrimitiveType::Bool => "bool".to_string(),
            PrimitiveType::U8 => "byte".to_string(),
            PrimitiveType::U16 => "ushort".to_string(),
            PrimitiveType::U32 => "uint".to_string(),
            PrimitiveType::U64 => "ulong".to_string(),
            PrimitiveType::I8 => "sbyte".to_string(),
            PrimitiveType::I16 => "short".to_string(),
            PrimitiveType::I32 => "int".to_string(),
            PrimitiveType::I64 => "long".to_string(),
            PrimitiveType::F32 => "float".to_string(),
            PrimitiveType::F64 => "double".to_string(),
        }
    }

    /// Converts a Rust enum name such as `Error` to a C# enum name `Error`.
    fn type_enum_to_typename(&self, x: &EnumType) -> String {
        x.name().to_string()
    }

    /// TODO Converts an opaque Rust struct `Context` to a C# struct ``.
    fn type_opaque_to_typename(&self, _: &OpaqueType) -> String {
        // x.name().to_string()
        "IntPtr".to_string()
    }

    /// Converts an Rust struct name `Vec2` to a C# struct name `Vec2`.
    fn type_composite_to_typename(&self, x: &CompositeType) -> String {
        x.name().to_string()
    }

    /// Converts an Rust `fn()` to a C# delegate name such as `InteropDelegate`.
    fn type_fnpointer_to_typename(&self, x: &FnPointerType) -> String {
        vec!["InteropDelegate".to_string(), safe_name(&x.internal_name())].join("_")
    }

    /// Converts the `u32` part in a Rust field `x: u32` to a C# equivalent. Might convert pointers to `IntPtr`.
    fn type_to_typespecifier_in_field(&self, x: &CType, _field: &Field, _composite: &CompositeType) -> String {
        match &x {
            CType::Primitive(x) => self.type_primitive_to_typename(x),
            CType::Enum(x) => self.type_enum_to_typename(x),
            CType::Opaque(x) => self.type_opaque_to_typename(x),
            CType::Composite(x) => self.type_composite_to_typename(x),
            CType::ReadPointer(_) => "IntPtr".to_string(),
            CType::ReadWritePointer(_) => "IntPtr".to_string(),
            CType::FnPointer(x) => self.type_fnpointer_to_typename(x),
            CType::Pattern(x) => {
                match x {
                    TypePattern::AsciiPointer => "string".to_string()
                }
            }
        }
    }

    /// Converts the `u32` part in a Rust paramter `x: u32` to a C# equivalent. Might convert pointers to `out X` or `ref X`.
    fn type_to_typespecifier_in_param(&self, x: &CType) -> String {
        match &x {
            CType::Primitive(x) => self.type_primitive_to_typename(x),
            CType::Enum(x) => self.type_enum_to_typename(x),
            CType::Opaque(x) => self.type_opaque_to_typename(x),
            CType::Composite(x) => self.type_composite_to_typename(x),
            CType::ReadPointer(z) => match **z {
                CType::Opaque(_) => "IntPtr".to_string(),
                CType::Primitive(PrimitiveType::Void) => "IntPtr".to_string(),
                CType::ReadPointer(_) => "ref IntPtr".to_string(),
                CType::ReadWritePointer(_) => "ref IntPtr".to_string(),
                _ => format!("ref {}", self.type_to_typespecifier_in_param(z)),
            },
            CType::ReadWritePointer(z) => match **z {
                CType::Opaque(_) => "IntPtr".to_string(),
                CType::Primitive(PrimitiveType::Void) => "IntPtr".to_string(),
                CType::ReadPointer(_) => "out IntPtr".to_string(),
                CType::ReadWritePointer(_) => "out IntPtr".to_string(),
                _ => format!("out {}", self.type_to_typespecifier_in_param(z)),
            },
            CType::FnPointer(x) => self.type_fnpointer_to_typename(x),
            CType::Pattern(x) => {
                match x {
                    TypePattern::AsciiPointer => "string".to_string()
                }
            }
        }
    }

    fn type_to_typespecifier_in_rval(&self, x: &CType) -> String {
        match &x {
            CType::Primitive(x) => self.type_primitive_to_typename(x),
            CType::Enum(x) => self.type_enum_to_typename(x),
            CType::Opaque(x) => self.type_opaque_to_typename(x),
            CType::Composite(x) => self.type_composite_to_typename(x),
            CType::ReadPointer(_) => "IntPtr".to_string(),
            CType::ReadWritePointer(_) => "IntPtr".to_string(),
            CType::FnPointer(x) => self.type_fnpointer_to_typename(x),
            CType::Pattern(x) => {
                match x {
                    TypePattern::AsciiPointer => "string".to_string()
                }
            }
        }
    }

    fn constant_value_to_value(&self, value: &ConstantValue) -> String {
        match value {
            ConstantValue::Primitive(x) => match x {
                PrimitiveValue::Bool(x) => format!("{}", x),
                PrimitiveValue::U8(x) => format!("{}", x),
                PrimitiveValue::U16(x) => format!("{}", x),
                PrimitiveValue::U32(x) => format!("{}", x),
                PrimitiveValue::U64(x) => format!("{}", x),
                PrimitiveValue::I8(x) => format!("{}", x),
                PrimitiveValue::I16(x) => format!("{}", x),
                PrimitiveValue::I32(x) => format!("{}", x),
                PrimitiveValue::I64(x) => format!("{}", x),
                PrimitiveValue::F32(x) => format!("{}", x),
                PrimitiveValue::F64(x) => format!("{}", x),
            },
        }
    }

    fn function_parameter_to_csharp_typename(&self, x: &Parameter, _function: &Function) -> String {
        self.type_to_typespecifier_in_param(x.the_type())
    }

    fn function_rval_to_csharp_typename(&self, function: &Function) -> String {
        self.type_to_typespecifier_in_rval(function.signature().rval())
    }

    fn function_name_to_csharp_name(&self, function: &Function) -> String {
        function.name().to_string()
    }

    fn write_to(&self, w: &mut IndentWriter) -> Result<(), Error> {
        self.write_file_header_comments(w)?;
        w.newline()?;

        self.write_imports(w)?;
        w.newline()?;

        self.write_namespace_context(w, |w| {
            self.write_class_context(w, |w| {
                self.write_native_lib_string(w)?;
                w.newline()?;

                self.write_constants(w)?;
                w.newline()?;

                self.write_functions(w)?;
                Ok(())
            })?;

            w.newline()?;
            self.write_type_definitions(w)?;

            Ok(())
        })?;

        Ok(())
    }

    fn write_file_header_comments(&self, w: &mut IndentWriter) -> Result<(), Error> {
        writeln!(w.writer(), "{}", &self.config().file_header_comment)?;
        Ok(())
    }

    fn write_imports(&self, w: &mut IndentWriter) -> Result<(), Error> {
        w.indented(|w| writeln!(w, r#"using System;"#))?;
        w.indented(|w| writeln!(w, r#"using System.Runtime.InteropServices;"#))?;

        Ok(())
    }

    fn write_native_lib_string(&self, w: &mut IndentWriter) -> Result<(), Error> {
        w.indented(|w| writeln!(w, r#"public const string NativeLib = "{}";"#, self.config().dll_name))?;
        Ok(())
    }

    fn write_constants(&self, w: &mut IndentWriter) -> Result<(), Error> {
        for constant in self.library().constants() {
            self.write_constant(w, constant)?;
            w.newline()?;
        }

        Ok(())
    }

    fn write_constant(&self, w: &mut IndentWriter, constant: &Constant) -> Result<(), Error> {
        self.write_documentation(w, constant.documentation())?;

        w.indented(|w| write!(w, r#"public const "#))?;

        write!(w.writer(), "{} ", self.type_to_typespecifier_in_rval(&constant.the_type()))?;
        write!(w.writer(), "{} = ", constant.name())?;
        write!(w.writer(), "{};", self.constant_value_to_value(constant.value()))?;

        w.newline()?;

        Ok(())
    }

    fn write_functions(&self, w: &mut IndentWriter) -> Result<(), Error> {
        for function in self.library().functions() {
            self.write_function(w, function)?;
            w.newline()?;
        }

        Ok(())
    }

    fn write_function(&self, w: &mut IndentWriter, function: &Function) -> Result<(), Error> {
        self.write_documentation(w, function.documentation())?;
        self.write_function_annotation(w, function)?;
        self.write_function_declaration(w, function)?;
        Ok(())
    }

    fn write_documentation(&self, w: &mut IndentWriter, documentation: &Documentation) -> Result<(), Error> {
        for line in documentation.lines() {
            w.indented(|w| writeln!(w, r#"/// {}"#, line))?;
        }

        Ok(())
    }

    fn write_function_annotation(&self, w: &mut IndentWriter, function: &Function) -> Result<(), Error> {
        w.indented(|w| {
            writeln!(
                w,
                r#"[DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "{}")]"#,
                function.name()
            )
        })?;
        Ok(())
    }

    fn write_function_declaration(&self, w: &mut IndentWriter, function: &Function) -> Result<(), Error> {
        w.indented(|w| write!(w, r#"public static extern "#))?;

        write!(w.writer(), "{}", self.function_rval_to_csharp_typename(function))?;
        write!(w.writer(), " {}(", self.function_name_to_csharp_name(function))?;

        let params = function.signature().params();
        for (i, p) in params.iter().enumerate() {
            write!(w.writer(), "{}", self.function_parameter_to_csharp_typename(p, function))?;
            write!(w.writer(), " {}", p.name())?;
            if i < params.len() - 1 {
                write!(w.writer(), ", ")?;
            }
        }

        writeln!(w.writer(), ");")?;
        Ok(())
    }

    fn write_type_definitions(&self, w: &mut IndentWriter) -> Result<(), Error> {
        for the_type in self.library().types() {
            self.write_type_definition(w, the_type)?;
        }

        Ok(())
    }

    fn write_type_definition(&self, w: &mut IndentWriter, the_type: &CType) -> Result<(), Error> {
        match the_type {
            CType::Primitive(_) => {}
            CType::Enum(e) => {
                self.write_type_definition_enum(w, e)?;
                w.newline()?;
            }
            CType::Opaque(_) => {}
            CType::Composite(c) => {
                self.write_type_definition_composite(w, c)?;
                w.newline()?;
            }
            CType::FnPointer(f) => {
                self.write_type_definition_fn_pointer(w, f)?;
                w.newline()?;
            }
            CType::ReadPointer(_) => {}
            CType::ReadWritePointer(_) => {}
            CType::Pattern(_) => {}
        }
        Ok(())
    }

    fn write_type_definition_fn_pointer(&self, w: &mut IndentWriter, the_type: &FnPointerType) -> Result<(), Error> {
        self.write_type_definition_fn_pointer_annotation(w, the_type)?;
        self.write_type_definition_fn_pointer_body(w, the_type)?;
        Ok(())
    }

    fn write_type_definition_fn_pointer_annotation(&self, w: &mut IndentWriter, _the_type: &FnPointerType) -> Result<(), Error> {
        w.indented(|w| writeln!(w, r#"[UnmanagedFunctionPointer(CallingConvention.Cdecl)]"#))?;
        Ok(())
    }

    fn write_type_definition_fn_pointer_body(&self, w: &mut IndentWriter, the_type: &FnPointerType) -> Result<(), Error> {
        w.indented(|w| write!(w, "public delegate {} ", self.type_to_typespecifier_in_rval(the_type.signature().rval())))?;
        write!(w.writer(), "{}(", self.type_fnpointer_to_typename(the_type))?;

        let params = the_type.signature().params();
        for (i, param) in params.iter().enumerate() {
            write!(w.writer(), "{} x{}", self.type_to_typespecifier_in_param(param.the_type()), i)?;

            if i < params.len() - 1 {
                write!(w.writer(), ", ")?;
            }
        }

        writeln!(w.writer(), ");")?;
        Ok(())
    }

    fn write_type_definition_enum(&self, w: &mut IndentWriter, the_type: &EnumType) -> Result<(), Error> {
        self.write_documentation(w, the_type.documentation())?;
        w.indented(|w| writeln!(w, r#"public enum {}"#, the_type.name()))?;
        w.indented(|w| writeln!(w, r#"{{"#))?;
        w.indent();

        for variant in the_type.variants() {
            self.write_type_definition_enum_variant(w, variant, the_type)?;
        }

        w.unindent();
        w.indented(|w| writeln!(w, r#"}}"#))?;
        Ok(())
    }

    fn write_type_definition_enum_variant(&self, w: &mut IndentWriter, variant: &Variant, _the_type: &EnumType) -> Result<(), Error> {
        let variant_name = variant.name();
        let variant_value = variant.value();
        self.write_documentation(w, variant.documentation())?;
        w.indented(|w| writeln!(w, r#"{} = {},"#, variant_name, variant_value))?;
        Ok(())
    }

    fn write_type_definition_composite(&self, w: &mut IndentWriter, the_type: &CompositeType) -> Result<(), Error> {
        self.write_documentation(w, the_type.documentation())?;
        self.write_type_definition_composite_annotation(w, the_type)?;
        self.write_type_definition_composite_body(w, the_type)?;
        Ok(())
    }

    fn write_type_definition_composite_annotation(&self, w: &mut IndentWriter, _the_type: &CompositeType) -> Result<(), Error> {
        w.indented(|w| writeln!(w, r#"[Serializable]"#))?;
        w.indented(|w| writeln!(w, r#"[StructLayout(LayoutKind.Sequential)]"#))?;

        Ok(())
    }

    fn write_type_definition_composite_body(&self, w: &mut IndentWriter, the_type: &CompositeType) -> Result<(), Error> {
        w.indented(|w| writeln!(w, r#"public partial struct {}"#, the_type.name()))?;
        w.indented(|w| writeln!(w, r#"{{"#))?;
        w.indent();

        for field in the_type.fields() {
            self.write_documentation(w, field.documentation())?;
            self.write_type_definition_composite_body_field(w, field, the_type)?;
        }

        w.unindent();
        w.indented(|w| writeln!(w, r#"}}"#))?;
        Ok(())
    }

    fn write_type_definition_composite_body_field(&self, w: &mut IndentWriter, field: &Field, the_type: &CompositeType) -> Result<(), Error> {
        let field_name = field.name();
        let type_name = self.type_to_typespecifier_in_field(field.the_type(), field, the_type);
        w.indented(|w| writeln!(w, r#"public {} {};"#, type_name, field_name))?;
        Ok(())
    }

    fn write_namespace_context(&self, w: &mut IndentWriter, f: impl FnOnce(&mut IndentWriter) -> Result<(), Error>) -> Result<(), Error> {
        w.indented(|w| writeln!(w, r#"namespace {}"#, self.config().namespace))?;
        w.indented(|w| writeln!(w, r#"{{"#))?;
        w.indent();

        f(w)?;

        w.unindent();
        w.indented(|w| writeln!(w, r#"}}"#))?;

        Ok(())
    }

    fn write_class_context(&self, w: &mut IndentWriter, f: impl FnOnce(&mut IndentWriter) -> Result<(), Error>) -> Result<(), Error> {
        w.indented(|w| writeln!(w, r#"public static partial class {}"#, self.config().class))?;
        w.indented(|w| writeln!(w, r#"{{"#))?;
        w.indent();

        f(w)?;

        w.unindent();
        w.indented(|w| writeln!(w, r#"}}"#))?;

        Ok(())
    }
}

impl Interop for Generator {
    fn config(&self) -> &Config {
        &self.config
    }

    fn library(&self) -> &Library {
        &self.library
    }
}
