use std::io::Cursor;

use regex::Regex;

fn parse_expects(source: &str, regex: Regex, field: usize) -> Vec<String> {
    let mut results = vec![];
    for line in source.lines() {
        let caps = regex.captures(line);
        if let Some(caps) = caps {
            results.push(caps[field].to_owned());
        }
    }

    results
}

#[derive(PartialEq, Debug)]
enum TestResult {
    Ok,
    CompileError,
    RuntimeError,
}

//TODO Handle errors
fn execute(source: &str) -> (Vec<String>, TestResult) {
    let module = match lox_compiler::compile(source) {
        Ok(module) => module,
        Err(_) => return (vec![], TestResult::CompileError),
    };

    let mut output = vec![];
    let cursor = Cursor::new(&mut output);
    let mut vm = lox_vm::bettervm::vm::Vm::with_stdout(module, cursor);
    lox_vm::bettervm::set_stdlib(&mut vm);
    let result = match vm.interpret() {
        Ok(_) => TestResult::Ok,
        Err(err) => {
            println!("Runtime error: {:?}", err);
            TestResult::RuntimeError
        },
    };

    let output = String::from_utf8(output).unwrap();

    (output.lines().map(|l| l.to_owned()).collect(), result)
}

fn harness(source: &str) {
    let expects = parse_expects(source, Regex::new(r"// expect: ?(.*)").unwrap(), 1);

    let expected_result =
        if !parse_expects(source, Regex::new(r"\[line (\d+)\] (Error.+)").unwrap(), 2).is_empty() {
            TestResult::CompileError
        } else if !parse_expects(source, Regex::new(r"// (Error.*)").unwrap(), 1).is_empty() {
            TestResult::CompileError
        } else if !parse_expects(
            source,
            Regex::new(r"// expect runtime error: (.+)").unwrap(),
            1,
        )
        .is_empty()
        {
            TestResult::RuntimeError
        } else {
            TestResult::Ok
        };

    let (output, result) = execute(source);
    assert_eq!(expects, output);
    assert_eq!(expected_result, result);
}

#[test]
fn precedence() {
    harness(include_str!("precedence.lox"));
}

#[test]
fn unexpected_character() {
    harness(include_str!("unexpected_character.lox"));
}

mod assignment {
    use super::harness;
    #[test]
    fn associativity() {
        harness(include_str!("assignment/associativity.lox"));
    }

    #[test]
    fn global() {
        harness(include_str!("assignment/global.lox"));
    }

    #[test]
    fn grouping() {
        harness(include_str!("assignment/grouping.lox"));
    }

    #[test]
    fn infix_operator() {
        harness(include_str!("assignment/infix_operator.lox"));
    }

    #[test]
    fn local() {
        harness(include_str!("assignment/local.lox"));
    }

    #[test]
    fn prefix_operator() {
        harness(include_str!("assignment/prefix_operator.lox"));
    }

    #[test]
    fn syntax() {
        harness(include_str!("assignment/syntax.lox"));
    }

    #[test]
    fn to_this() {
        harness(include_str!("assignment/to_this.lox"));
    }

    #[test]
    fn undefined() {
        harness(include_str!("assignment/undefined.lox"));
    }
}

mod block {
    use super::harness;

    #[test]
    fn empty() {
        harness(include_str!("block/empty.lox"));
    }

    #[test]
    fn scope() {
        harness(include_str!("block/scope.lox"));
    }
}

mod bool {
    use super::harness;

    #[test]
    fn equality() {
        harness(include_str!("bool/equality.lox"));
    }

    #[test]
    fn not() {
        harness(include_str!("bool/not.lox"));
    }
}

mod call {
    use super::harness;

    #[test]
    fn bool() {
        harness(include_str!("call/bool.lox"));
    }

    #[test]
    fn nil() {
        harness(include_str!("call/nil.lox"));
    }

    #[test]
    fn num() {
        harness(include_str!("call/num.lox"));
    }

    #[test]
    fn object() {
        harness(include_str!("call/object.lox"));
    }

    #[test]
    fn string() {
        harness(include_str!("call/string.lox"));
    }
}

mod class {
    use super::harness;

    #[test]
    fn empty() {
        harness(include_str!("class/empty.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn inherit_self() {
        harness(include_str!("class/inherit_self.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn inherited_method() {
        harness(include_str!("class/inherited_method.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn local_inherit_other() {
        harness(include_str!("class/local_inherit_other.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn local_inherit_self() {
        harness(include_str!("class/local_inherit_self.lox"));
    }

    #[test]
    fn local_reference_self() {
        harness(include_str!("class/local_reference_self.lox"));
    }

    #[test]
    fn reference_self() {
        harness(include_str!("class/reference_self.lox"));
    }
}

mod closure {
    use super::harness;

    #[test]
    fn assign_to_closure() {
        harness(include_str!("closure/assign_to_closure.lox"));
    }

    #[test]
    fn assign_to_shadowed_later() {
        harness(include_str!("closure/assign_to_shadowed_later.lox"));
    }

    #[test]
    fn close_over_function_parameter() {
        harness(include_str!("closure/close_over_function_parameter.lox"));
    }

    #[test]
    fn close_over_later_variable() {
        harness(include_str!("closure/close_over_later_variable.lox"));
    }

    #[test]
    fn close_over_method_parameter() {
        harness(include_str!("closure/close_over_method_parameter.lox"));
    }

    #[test]
    fn closed_closure_in_function() {
        harness(include_str!("closure/closed_closure_in_function.lox"));
    }

    #[test]
    fn nested_closure() {
        harness(include_str!("closure/nested_closure.lox"));
    }

    #[test]
    fn open_closure_in_function() {
        harness(include_str!("closure/open_closure_in_function.lox"));
    }

    #[test]
    fn reference_closure_multiple_times() {
        harness(include_str!("closure/reference_closure_multiple_times.lox"));
    }

    #[test]
    fn reuse_closure_slot() {
        harness(include_str!("closure/reuse_closure_slot.lox"));
    }

    #[test]
    fn shadow_closure_with_local() {
        harness(include_str!("closure/shadow_closure_with_local.lox"));
    }

    #[test]
    fn unused_closure() {
        harness(include_str!("closure/unused_closure.lox"));
    }

    #[test]
    fn unused_later_closure() {
        harness(include_str!("closure/unused_later_closure.lox"));
    }
}

mod comments {
    use super::harness;

    #[test]
    fn line_at_eof() {
        harness(include_str!("comments/line_at_eof.lox"));
    }

    #[test]
    fn only_line_comment_and_line() {
        harness(include_str!("comments/only_line_comment_and_line.lox"));
    }

    #[test]
    fn only_line_comment() {
        harness(include_str!("comments/only_line_comment.lox"));
    }

    #[test]
    fn unicode() {
        harness(include_str!("comments/unicode.lox"));
    }
}

mod constructor {
    use super::harness;

    #[test]
    fn arguments() {
        harness(include_str!("constructor/arguments.lox"));
    }
    #[test]
    fn call_init_early_return() {
        harness(include_str!("constructor/call_init_early_return.lox"));
    }
    #[test]
    fn call_init_explicitly() {
        harness(include_str!("constructor/call_init_explicitly.lox"));
    }
    #[test]
    fn default_arguments() {
        harness(include_str!("constructor/default_arguments.lox"));
    }
    #[test]
    fn default() {
        harness(include_str!("constructor/default.lox"));
    }
    #[test]
    fn early_return() {
        harness(include_str!("constructor/early_return.lox"));
    }
    #[test]
    fn extra_arguments() {
        harness(include_str!("constructor/extra_arguments.lox"));
    }
    #[test]
    fn init_not_method() {
        harness(include_str!("constructor/init_not_method.lox"));
    }
    #[test]
    fn missing_arguments() {
        harness(include_str!("constructor/missing_arguments.lox"));
    }
    #[test]
    fn return_in_nested_function() {
        harness(include_str!("constructor/return_in_nested_function.lox"));
    }
    #[test]
    fn return_value() {
        harness(include_str!("constructor/return_value.lox"));
    }
}

mod field {
    use super::harness;

    #[test]
    fn call_function_field() {
        harness(include_str!("field/call_function_field.lox"));
    }
    #[test]
    fn call_nonfunction_field() {
        harness(include_str!("field/call_nonfunction_field.lox"));
    }
    #[test]
    fn get_and_set_method() {
        harness(include_str!("field/get_and_set_method.lox"));
    }
    #[test]
    fn get_on_bool() {
        harness(include_str!("field/get_on_bool.lox"));
    }
    #[test]
    fn get_on_class() {
        harness(include_str!("field/get_on_class.lox"));
    }
    #[test]
    fn get_on_function() {
        harness(include_str!("field/get_on_function.lox"));
    }
    #[test]
    fn get_on_nil() {
        harness(include_str!("field/get_on_nil.lox"));
    }
    #[test]
    fn get_on_num() {
        harness(include_str!("field/get_on_num.lox"));
    }
    #[test]
    fn get_on_string() {
        harness(include_str!("field/get_on_string.lox"));
    }
    #[test]
    fn many() {
        harness(include_str!("field/many.lox"));
    }
    #[test]
    fn method_binds_this() {
        harness(include_str!("field/method_binds_this.lox"));
    }
    #[test]
    fn method() {
        harness(include_str!("field/method.lox"));
    }
    #[test]
    fn on_instance() {
        harness(include_str!("field/on_instance.lox"));
    }
    #[test]
    fn set_evaluation_order() {
        harness(include_str!("field/set_evaluation_order.lox"));
    }
    #[test]
    fn set_on_bool() {
        harness(include_str!("field/set_on_bool.lox"));
    }
    #[test]
    fn set_on_class() {
        harness(include_str!("field/set_on_class.lox"));
    }
    #[test]
    fn set_on_function() {
        harness(include_str!("field/set_on_function.lox"));
    }
    #[test]
    fn set_on_nil() {
        harness(include_str!("field/set_on_nil.lox"));
    }
    #[test]
    fn set_on_num() {
        harness(include_str!("field/set_on_num.lox"));
    }
    #[test]
    fn set_on_string() {
        harness(include_str!("field/set_on_string.lox"));
    }
    #[test]
    fn undefined() {
        harness(include_str!("field/undefined.lox"));
    }
}

mod r#for {
    use super::harness;

    #[test]
    fn class_in_body() {
        harness(include_str!("for/class_in_body.lox"));
    }
    #[test]
    fn closure_in_body() {
        harness(include_str!("for/closure_in_body.lox"));
    }
    #[test]
    fn fun_in_body() {
        harness(include_str!("for/fun_in_body.lox"));
    }
    #[test]
    fn return_closure() {
        harness(include_str!("for/return_closure.lox"));
    }
    #[test]
    fn return_inside() {
        harness(include_str!("for/return_inside.lox"));
    }
    #[test]
    fn scope() {
        harness(include_str!("for/scope.lox"));
    }
    #[test]
    fn statement_condition() {
        harness(include_str!("for/statement_condition.lox"));
    }
    #[test]
    fn statement_increment() {
        harness(include_str!("for/statement_increment.lox"));
    }
    #[test]
    fn statement_initializer() {
        harness(include_str!("for/statement_initializer.lox"));
    }
    #[test]
    fn syntax() {
        harness(include_str!("for/syntax.lox"));
    }
    #[test]
    fn var_in_body() {
        harness(include_str!("for/var_in_body.lox"));
    }
}

mod function {
    use super::harness;

    #[test]
    fn body_must_be_block() {
        harness(include_str!("function/body_must_be_block.lox"));
    }
    #[test]
    fn empty_body() {
        harness(include_str!("function/empty_body.lox"));
    }
    #[test]
    fn extra_arguments() {
        harness(include_str!("function/extra_arguments.lox"));
    }
    #[test]
    fn local_mutual_recursion() {
        harness(include_str!("function/local_mutual_recursion.lox"));
    }
    #[test]
    fn local_recursion() {
        harness(include_str!("function/local_recursion.lox"));
    }
    #[test]
    fn missing_arguments() {
        harness(include_str!("function/missing_arguments.lox"));
    }
    #[test]
    fn missing_comma_in_parameters() {
        harness(include_str!("function/missing_comma_in_parameters.lox"));
    }
    #[test]
    fn mutual_recursion() {
        harness(include_str!("function/mutual_recursion.lox"));
    }
    #[test]
    fn nested_call_with_arguments() {
        harness(include_str!("function/nested_call_with_arguments.lox"));
    }
    #[test]
    fn parameters() {
        harness(include_str!("function/parameters.lox"));
    }
    #[test]
    fn print() {
        harness(include_str!("function/print.lox"));
    }
    #[test]
    fn recursion() {
        harness(include_str!("function/recursion.lox"));
    }
}

mod r#if {
    use super::harness;

    #[test]
    fn class_in_else() {
        harness(include_str!("if/class_in_else.lox"));
    }
    #[test]
    fn class_in_then() {
        harness(include_str!("if/class_in_then.lox"));
    }
    #[test]
    fn dangling_else() {
        harness(include_str!("if/dangling_else.lox"));
    }
    #[test]
    fn r#else() {
        harness(include_str!("if/else.lox"));
    }
    #[test]
    fn fun_in_else() {
        harness(include_str!("if/fun_in_else.lox"));
    }
    #[test]
    fn fun_in_then() {
        harness(include_str!("if/fun_in_then.lox"));
    }
    #[test]
    fn r#if() {
        harness(include_str!("if/if.lox"));
    }
    #[test]
    fn truth() {
        harness(include_str!("if/truth.lox"));
    }
    #[test]
    fn var_in_else() {
        harness(include_str!("if/var_in_else.lox"));
    }
    #[test]
    fn var_in_then() {
        harness(include_str!("if/var_in_then.lox"));
    }
}

mod inheritance {
    use super::harness;

    #[test]
    #[ignore = "not yet implemented"]
    fn constructor() {
        harness(include_str!("inheritance/constructor.lox"));
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn inherit_from_function() {
        harness(include_str!("inheritance/inherit_from_function.lox"));
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn inherit_from_nil() {
        harness(include_str!("inheritance/inherit_from_nil.lox"));
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn inherit_from_number() {
        harness(include_str!("inheritance/inherit_from_number.lox"));
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn inherit_methods() {
        harness(include_str!("inheritance/inherit_methods.lox"));
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn parenthesized_superclass() {
        harness(include_str!("inheritance/parenthesized_superclass.lox"));
    }
    #[test]
    #[ignore = "not yet implemented"]
    fn set_fields_from_base_class() {
        harness(include_str!("inheritance/set_fields_from_base_class.lox"));
    }
}

mod logical_operator {
    use super::harness;

    #[test]
    fn and_truth() {
        harness(include_str!("logical_operator/and_truth.lox"));
    }

    #[test]
    fn and() {
        harness(include_str!("logical_operator/and.lox"));
    }

    #[test]
    fn or_truth() {
        harness(include_str!("logical_operator/or_truth.lox"));
    }

    #[test]
    fn or() {
        harness(include_str!("logical_operator/or.lox"));
    }
}

mod method {
    use super::harness;

    #[test]
    fn arity() {
        harness(include_str!("method/arity.lox"));
    }

    #[test]
    fn empty_block() {
        harness(include_str!("method/empty_block.lox"));
    }

    #[test]
    fn extra_arguments() {
        harness(include_str!("method/extra_arguments.lox"));
    }

    #[test]
    fn missing_arguments() {
        harness(include_str!("method/missing_arguments.lox"));
    }

    #[test]
    fn not_found() {
        harness(include_str!("method/not_found.lox"));
    }

    #[test]
    fn print_bound_method() {
        harness(include_str!("method/print_bound_method.lox"));
    }

    #[test]
    fn refer_to_name() {
        harness(include_str!("method/refer_to_name.lox"));
    }
}

mod nil {
    use super::harness;

    #[test]
    fn literal() {
        harness(include_str!("nil/literal.lox"));
    }
}

mod number {
    use super::harness;

    #[test]
    fn decimal_point_at_eof() {
        harness(include_str!("number/decimal_point_at_eof.lox"));
    }
    #[test]
    fn leading_dot() {
        harness(include_str!("number/leading_dot.lox"));
    }
    #[test]
    fn literals() {
        // Removed from test because println! works differently
        // print -0;      // expect: -0
        harness(include_str!("number/literals.lox"));
    }
    #[test]
    fn nan_equality() {
        harness(include_str!("number/nan_equality.lox"));
    }
    #[test]
    fn trailing_dot() {
        harness(include_str!("number/trailing_dot.lox"));
    }
}

mod operator {
    use super::harness;

    #[test]
    fn add_bool_nil() {
        harness(include_str!("operator/add_bool_nil.lox"));
    }

    #[test]
    fn add_bool_num() {
        harness(include_str!("operator/add_bool_num.lox"));
    }

    #[test]
    fn add_bool_string() {
        harness(include_str!("operator/add_bool_string.lox"));
    }

    #[test]
    fn add_nil_nil() {
        harness(include_str!("operator/add_nil_nil.lox"));
    }

    #[test]
    fn add_num_nil() {
        harness(include_str!("operator/add_num_nil.lox"));
    }

    #[test]
    fn add_string_nil() {
        harness(include_str!("operator/add_string_nil.lox"));
    }

    #[test]
    fn add() {
        harness(include_str!("operator/add.lox"));
    }

    #[test]
    fn comparison() {
        harness(include_str!("operator/comparison.lox"));
    }

    #[test]
    fn divide_nonnum_num() {
        harness(include_str!("operator/divide_nonnum_num.lox"));
    }

    #[test]
    fn divide_num_nonnum() {
        harness(include_str!("operator/divide_num_nonnum.lox"));
    }

    #[test]
    fn divide() {
        harness(include_str!("operator/divide.lox"));
    }

    #[test]
    fn equals_class() {
        harness(include_str!("operator/equals_class.lox"));
    }

    #[test]
    fn equals_method() {
        harness(include_str!("operator/equals_method.lox"));
    }

    #[test]
    fn equals() {
        harness(include_str!("operator/equals.lox"));
    }

    #[test]
    fn greater_nonnum_num() {
        harness(include_str!("operator/greater_nonnum_num.lox"));
    }

    #[test]
    fn greater_num_nonnum() {
        harness(include_str!("operator/greater_num_nonnum.lox"));
    }

    #[test]
    fn greater_or_equal_nonnum_num() {
        harness(include_str!("operator/greater_or_equal_nonnum_num.lox"));
    }

    #[test]
    fn greater_or_equal_num_nonnum() {
        harness(include_str!("operator/greater_or_equal_num_nonnum.lox"));
    }

    #[test]
    fn less_nonnum_num() {
        harness(include_str!("operator/less_nonnum_num.lox"));
    }

    #[test]
    fn less_num_nonnum() {
        harness(include_str!("operator/less_num_nonnum.lox"));
    }

    #[test]
    fn less_or_equal_nonnum_num() {
        harness(include_str!("operator/less_or_equal_nonnum_num.lox"));
    }

    #[test]
    fn less_or_equal_num_nonnum() {
        harness(include_str!("operator/less_or_equal_num_nonnum.lox"));
    }

    #[test]
    fn multiply_nonnum_num() {
        harness(include_str!("operator/multiply_nonnum_num.lox"));
    }

    #[test]
    fn multiply_num_nonnum() {
        harness(include_str!("operator/multiply_num_nonnum.lox"));
    }

    #[test]
    fn multiply() {
        harness(include_str!("operator/multiply.lox"));
    }

    #[test]
    fn negate_nonnum() {
        harness(include_str!("operator/negate_nonnum.lox"));
    }

    #[test]
    fn negate() {
        harness(include_str!("operator/negate.lox"));
    }

    #[test]
    fn not_class() {
        harness(include_str!("operator/not_class.lox"));
    }

    #[test]
    fn not_equals() {
        harness(include_str!("operator/not_equals.lox"));
    }

    #[test]
    fn not() {
        harness(include_str!("operator/not.lox"));
    }

    #[test]
    fn subtract_nonnum_num() {
        harness(include_str!("operator/subtract_nonnum_num.lox"));
    }

    #[test]
    fn subtract_num_nonnum() {
        harness(include_str!("operator/subtract_num_nonnum.lox"));
    }

    #[test]
    fn subtract() {
        harness(include_str!("operator/subtract.lox"));
    }
}

mod print {
    use super::harness;

    #[test]
    fn missing_argument() {
        harness(include_str!("print/missing_argument.lox"));
    }
}

mod regression {
    use super::harness;

    #[test]
    fn regression_40() {
        harness(include_str!("regression/40.lox"));
    }

    #[test]
    fn regression_394() {
        harness(include_str!("regression/394.lox"));
    }
}

mod r#return {
    use super::harness;

    #[test]
    fn after_else() {
        harness(include_str!("return/after_else.lox"));
    }
    #[test]
    fn after_if() {
        harness(include_str!("return/after_if.lox"));
    }
    #[test]
    fn after_while() {
        harness(include_str!("return/after_while.lox"));
    }
    #[test]
    fn at_top_level() {
        harness(include_str!("return/at_top_level.lox"));
    }
    #[test]
    fn in_function() {
        harness(include_str!("return/in_function.lox"));
    }
    #[test]
    fn in_method() {
        harness(include_str!("return/in_method.lox"));
    }
    #[test]
    fn return_nil_if_no_value() {
        harness(include_str!("return/return_nil_if_no_value.lox"));
    }
}

mod string {
    use super::harness;

    #[test]
    fn error_after_multiline() {
        harness(include_str!("string/error_after_multiline.lox"));
    }
    #[test]
    fn literals() {
        harness(include_str!("string/literals.lox"));
    }
    #[test]
    fn multiline() {
        harness(include_str!("string/multiline.lox"));
    }
    #[test]
    fn unterminated() {
        harness(include_str!("string/unterminated.lox"));
    }
}

mod super_tests {
    use super::harness;

    #[test]
    #[ignore = "not yet implemented"]
    fn bound_method() {
        harness(include_str!("super/bound_method.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn call_other_method() {
        harness(include_str!("super/call_other_method.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn call_same_method() {
        harness(include_str!("super/call_same_method.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn closure() {
        harness(include_str!("super/closure.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn constructor() {
        harness(include_str!("super/constructor.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn extra_arguments() {
        harness(include_str!("super/extra_arguments.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn indirectly_inherited() {
        harness(include_str!("super/indirectly_inherited.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn missing_arguments() {
        harness(include_str!("super/missing_arguments.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn no_superclass_bind() {
        harness(include_str!("super/no_superclass_bind.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn no_superclass_call() {
        harness(include_str!("super/no_superclass_call.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn no_superclass_method() {
        harness(include_str!("super/no_superclass_method.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parenthesized() {
        harness(include_str!("super/parenthesized.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn reassign_superclass() {
        harness(include_str!("super/reassign_superclass.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn super_at_top_level() {
        harness(include_str!("super/super_at_top_level.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn super_in_closure_in_inherited_method() {
        harness(include_str!("super/super_in_closure_in_inherited_method.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn super_in_inherited_method() {
        harness(include_str!("super/super_in_inherited_method.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn super_in_top_level_function() {
        harness(include_str!("super/super_in_top_level_function.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn super_without_dot() {
        harness(include_str!("super/super_without_dot.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn super_without_name() {
        harness(include_str!("super/super_without_name.lox"));
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn this_in_superclass_method() {
        harness(include_str!("super/this_in_superclass_method.lox"));
    }
}

mod this {
    use super::harness;

    #[test]
    fn closure() {
        harness(include_str!("this/closure.lox"));
    }
    #[test]
    fn nested_class() {
        harness(include_str!("this/nested_class.lox"));
    }
    #[test]
    fn nested_closure() {
        harness(include_str!("this/nested_closure.lox"));
    }
    #[test]
    fn this_at_top_level() {
        harness(include_str!("this/this_at_top_level.lox"));
    }
    #[test]
    fn this_in_method() {
        harness(include_str!("this/this_in_method.lox"));
    }
    #[test]
    fn this_in_top_level_function() {
        harness(include_str!("this/this_in_top_level_function.lox"));
    }
}

mod variable {
    use super::harness;

    #[test]
    fn collide_with_parameter() {
        harness(include_str!("variable/collide_with_parameter.lox"));
    }
    #[test]
    fn duplicate_local() {
        harness(include_str!("variable/duplicate_local.lox"));
    }
    #[test]
    fn duplicate_parameter() {
        harness(include_str!("variable/duplicate_parameter.lox"));
    }
    #[test]
    fn early_bound() {
        harness(include_str!("variable/early_bound.lox"));
    }
    #[test]
    fn in_middle_of_block() {
        harness(include_str!("variable/in_middle_of_block.lox"));
    }
    #[test]
    fn in_nested_block() {
        harness(include_str!("variable/in_nested_block.lox"));
    }
    #[test]
    fn local_from_method() {
        harness(include_str!("variable/local_from_method.lox"));
    }
    #[test]
    fn redeclare_global() {
        harness(include_str!("variable/redeclare_global.lox"));
    }
    #[test]
    fn redefine_global() {
        harness(include_str!("variable/redefine_global.lox"));
    }
    #[test]
    fn scope_reuse_in_different_blocks() {
        harness(include_str!("variable/scope_reuse_in_different_blocks.lox"));
    }
    #[test]
    fn shadow_and_local() {
        harness(include_str!("variable/shadow_and_local.lox"));
    }
    #[test]
    fn shadow_global() {
        harness(include_str!("variable/shadow_global.lox"));
    }
    #[test]
    fn shadow_local() {
        harness(include_str!("variable/shadow_local.lox"));
    }
    #[test]
    fn undefined_global() {
        harness(include_str!("variable/undefined_global.lox"));
    }
    #[test]
    fn undefined_local() {
        harness(include_str!("variable/undefined_local.lox"));
    }
    #[test]
    fn uninitialized() {
        harness(include_str!("variable/uninitialized.lox"));
    }
    #[test]
    fn unreached_undefined() {
        harness(include_str!("variable/unreached_undefined.lox"));
    }
    #[test]
    fn use_false_as_var() {
        harness(include_str!("variable/use_false_as_var.lox"));
    }
    #[test]
    fn use_global_in_initializer() {
        harness(include_str!("variable/use_global_in_initializer.lox"));
    }
    #[test]
    fn use_local_in_initializer() {
        harness(include_str!("variable/use_local_in_initializer.lox"));
    }
    #[test]
    fn use_nil_as_var() {
        harness(include_str!("variable/use_nil_as_var.lox"));
    }
    #[test]
    fn use_this_as_var() {
        harness(include_str!("variable/use_this_as_var.lox"));
    }
}

mod r#while {
    use super::harness;

    #[test]
    fn class_in_body() {
        harness(include_str!("while/class_in_body.lox"));
    }
    #[test]
    fn closure_in_body() {
        harness(include_str!("while/closure_in_body.lox"));
    }
    #[test]
    fn fun_in_body() {
        harness(include_str!("while/fun_in_body.lox"));
    }
    #[test]
    fn return_closure() {
        harness(include_str!("while/return_closure.lox"));
    }
    #[test]
    fn return_inside() {
        harness(include_str!("while/return_inside.lox"));
    }
    #[test]
    fn syntax() {
        harness(include_str!("while/syntax.lox"));
    }
    #[test]
    fn var_in_body() {
        harness(include_str!("while/var_in_body.lox"));
    }
}