use crate::compiler::prelude::*;
use crate::core::Value;
use crate::example;
use crate::prelude::{
    ArgumentList, Collection, Compiled, Example, Expression, FunctionCompileContext, kind,
};
use crate::value::{KeyString, ObjectMap};

#[derive(Clone, Debug, Copy)]
pub struct ToEntries;

fn build_entry((key, value): (KeyString, Value)) -> Value {
    let entry = ObjectMap::from([("key".into(), Value::from(key)), ("value".into(), value)]);
    Value::Object(entry)
}

fn to_entries(value: Value) -> Resolved {
    let object = value.try_object()?;
    Ok(Value::Array(object.into_iter().map(build_entry).collect()))
}

impl Function for ToEntries {
    fn identifier(&self) -> &'static str {
        "to_entries"
    }

    fn usage(&self) -> &'static str {
        "Converts JSON objects into array of objects."
    }

    fn category(&self) -> &'static str {
        Category::Object.as_ref()
    }

    fn return_kind(&self) -> u16 {
        kind::ARRAY
    }

    fn return_rules(&self) -> &'static [&'static str] {
        &["The return array has same inner objects count as the key counter of `value` object."]
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "value",
            kind: kind::OBJECT,
            required: true,
            description: "The object to manipulate.",
            default: None,
        }]
    }

    fn examples(&self) -> &'static [Example] {
        &[
            example! {
                title: "Manipulate empty object",
                source: "to_entries({})",
                result: Ok("[]"),
            },
            example! {
                title: "Manipulate object",
                source: r#"to_entries({ "foo": "bar"})"#,
                result: Ok(r#"[{ "key": "foo", "value": "bar" }]"#),
            },
        ]
    }

    fn compile(
        &self,
        _state: &state::TypeState,
        _ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");
        Ok(ToEntriesFn { value }.as_expr())
    }
}

#[derive(Clone, Debug)]
struct ToEntriesFn {
    value: Box<dyn Expression>,
}

impl FunctionExpression for ToEntriesFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let value = self.value.resolve(ctx)?;
        to_entries(value)
    }

    fn type_def(&self, _state: &TypeState) -> TypeDef {
        TypeDef::array(Collection::any())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::value;

    test_function![
        to_entries => ToEntries;

        empty_object {
            args: func_args![value: value!({})],
            want: Ok(value!([])),
            tdef: TypeDef::array(Collection::any()),
        }

        object {
            args: func_args![value: value!({foo: "bar"})],
            want: Ok(value!([{key: "foo", value: "bar"}])),
            tdef: TypeDef::array(Collection::any()),
        }

        non_object {
            args: func_args![value: value!(true)],
            want: Err("expected object, got boolean"),
            tdef: TypeDef::array(Collection::any()),
        }
    ];
}
