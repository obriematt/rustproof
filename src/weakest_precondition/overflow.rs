// The Rust-Proof Project is copyright 2016, Sami Sahli,
// Michael Salter, Matthew Slocum, Vincent Schuster,
// Bradley Rasmussen, Drew Gohman, and Matthew O'Brien.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions to generate overflow checks in the weakest precondition.

extern crate rustc_const_math;

use expression::*;
use rustc::mir::repr::*;


/// Routes to appropriate overflow check (signed / unsigned)
// One catch-all function for overflow checking.
pub fn overflow_check(wp: &Expression,
                      var: &VariableMappingData,
                      binop: &BinOp,
                      lvalue: &Expression,
                      rvalue: &Expression)
                      -> Expression {
    let v = var.clone();

    Expression::BinaryExpression( BinaryExpressionData {
        op: BinaryOperator::And,
        left: Box::new(wp.clone()),
        right: Box::new(
            match v.var_type.as_str() {
                "i8" => signed_overflow(binop, 8u8, lvalue, rvalue),
                "i16" => signed_overflow(binop, 16u8, lvalue, rvalue),
                "i32" => signed_overflow(binop, 32u8, lvalue, rvalue),
                "i64" => signed_overflow(binop, 64u8, lvalue, rvalue),
                "u8" | "u16" | "u32" | "u64" => {
                    unsigned_overflow(binop, lvalue, rvalue)
                },
                _ => panic!("Unsupported return type of binary operation: {}", v.var_type),
            }
        ),
    })
}

/// Routes to appropriate overflow check
// Signed: Match on the type of BinOp and call the correct function
fn signed_overflow(binop: &BinOp, size: u8, lvalue: &Expression, rvalue: &Expression)
                   -> Expression {
    match *binop {
        BinOp::Add => signed_add(size, lvalue, rvalue),
        BinOp::Mul => signed_mul(lvalue, rvalue),
        BinOp::Sub => signed_sub(size, lvalue, rvalue),
        BinOp::Div => signed_div(size, lvalue, rvalue),
        BinOp::Rem => signed_div(size, lvalue, rvalue),
        BinOp::Shl => unimplemented!(),
        BinOp::Shr => unimplemented!(),
        BinOp::BitOr => unimplemented!(),
        BinOp::BitAnd => unimplemented!(),
        BinOp::BitXor => unimplemented!(),
        BinOp::Lt => unimplemented!(),
        BinOp::Le => unimplemented!(),
        BinOp::Gt => unimplemented!(),
        BinOp::Ge => unimplemented!(),
        BinOp::Eq => unimplemented!(),
        BinOp::Ne => unimplemented!(),
    }
}

/// Creates an Expression containing overflow and underflow checks for lvalue + rvalue, assuming
/// they are bitvectors of length "size"
///
/// The following psuedocode provides a logically equivalent version of what is produced
/// (false is returned if overflow/underflow has occurred, true otherwise)
///
/// ```psuedo
/// If lvalue >= 0 && rvalue >= 0
///   If lvalue + rvalue < 0
///     false
///   Else
///     true
/// Else
///   If lvalue < 0 && rvalue < 0
///     If lvalue + rvalue >= 0
///       false
///     Else
///       true
///   Else
///     true
/// ```
fn signed_add(size: u8, lvalue: &Expression, rvalue: &Expression) -> Expression {
    Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::And,
        left: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Implication,
                left: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::And,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::GreaterThanOrEqual,
                                left: Box::new(lvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                        right: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::GreaterThanOrEqual,
                                left: Box::new(rvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                    })
                ),
                right: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::GreaterThanOrEqual,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::Addition,
                                left: Box::new(lvalue.clone()),
                                right: Box::new(rvalue.clone()),
                            })
                        ),
                        right: Box::new(
                            Expression::SignedBitVector( SignedBitVectorData {
                                size: size,
                                value: 0i64,
                            })
                        ),
                    })
                ),
            })
        ),
        right: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Implication,
                left: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::Or,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::LessThan,
                                left: Box::new(lvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                        right: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::LessThan,
                                left: Box::new(rvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                    })
                ),
                right: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::Implication,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::And,
                                left: Box::new(
                                    Expression::BinaryExpression( BinaryExpressionData{
                                        op: BinaryOperator::LessThan,
                                        left: Box::new(lvalue.clone()),
                                        right: Box::new(
                                            Expression::SignedBitVector( SignedBitVectorData {
                                                size: size,
                                                value: 0i64,
                                            })
                                        ),
                                    })
                                ),
                                right: Box::new(
                                    Expression::BinaryExpression( BinaryExpressionData{
                                        op: BinaryOperator::LessThan,
                                        left: Box::new(rvalue.clone()),
                                        right: Box::new(
                                            Expression::SignedBitVector( SignedBitVectorData {
                                                size: size,
                                                value: 0i64,
                                            })
                                        ),
                                    })
                                ),
                            })
                        ),
                        right: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::LessThan,
                                left: Box::new(
                                    Expression::BinaryExpression( BinaryExpressionData{
                                        op: BinaryOperator::Addition,
                                        left: Box::new(lvalue.clone()),
                                        right: Box::new(rvalue.clone()),
                                    })
                                ),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                    })
                ),
            })
        ),
    })
}

/// Creates an Expression containing overflow and underflow checks for lvalue - rvalue, assuming
/// they are bitvectors of length "size"
///
/// The following psuedocode provides a logically equivalent version of what is produced
/// (false is returned if overflow/underflow has occurred, true otherwise)
///
/// ```psuedo
/// If lvalue >= 0 && rvalue < 0
///   If lvalue - rvalue < 0
///     false
///   Else
///     true
/// Else
///   If lvalue < 0 && rvalue >= 0
///     If lvalue - rvalue >= 0
///       false
///     Else
///       true
///   Else
///     true
/// ```
fn signed_sub(size: u8, lvalue: &Expression, rvalue: &Expression) -> Expression {
    Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::And,
        left: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Implication,
                left: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::And,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::GreaterThanOrEqual,
                                left: Box::new(lvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                        right: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::LessThan,
                                left: Box::new(rvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                    })
                ),
                right: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::GreaterThanOrEqual,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::Subtraction,
                                left: Box::new(lvalue.clone()),
                                right: Box::new(rvalue.clone()),
                            })
                        ),
                        right: Box::new(
                            Expression::SignedBitVector( SignedBitVectorData {
                                size: size,
                                value: 0i64,
                            })
                        ),
                    })
                ),
            })
        ),
        right: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Implication,
                left: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::Or,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::LessThan,
                                left: Box::new(lvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                        right: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::GreaterThanOrEqual,
                                left: Box::new(rvalue.clone()),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                    })
                ),
                right: Box::new(
                    Expression::BinaryExpression( BinaryExpressionData{
                        op: BinaryOperator::Implication,
                        left: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::And,
                                left: Box::new(
                                    Expression::BinaryExpression( BinaryExpressionData{
                                        op: BinaryOperator::LessThan,
                                        left: Box::new(lvalue.clone()),
                                        right: Box::new(
                                            Expression::SignedBitVector( SignedBitVectorData {
                                                size: size,
                                                value: 0i64,
                                            })
                                        ),
                                    })
                                ),
                                right: Box::new(
                                    Expression::BinaryExpression( BinaryExpressionData{
                                        op: BinaryOperator::GreaterThanOrEqual,
                                        left: Box::new(rvalue.clone()),
                                        right: Box::new(
                                            Expression::SignedBitVector( SignedBitVectorData {
                                                size: size,
                                                value: 0i64,
                                            })
                                        ),
                                    })
                                ),
                            })
                        ),
                        right: Box::new(
                            Expression::BinaryExpression( BinaryExpressionData{
                                op: BinaryOperator::LessThan,
                                left: Box::new(
                                    Expression::BinaryExpression( BinaryExpressionData{
                                        op: BinaryOperator::Subtraction,
                                        left: Box::new(lvalue.clone()),
                                        right: Box::new(rvalue.clone()),
                                    })
                                ),
                                right: Box::new(
                                    Expression::SignedBitVector( SignedBitVectorData {
                                        size: size,
                                        value: 0i64,
                                    })
                                ),
                            })
                        ),
                    })
                ),
            })
        ),
    })
}

fn signed_mul(lvalue: &Expression, rvalue: &Expression) -> Expression {
    let overflow: Expression = Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::SignedMultiplicationDoesNotOverflow,
        left: Box::new(lvalue.clone()),
        right: Box::new(rvalue.clone()),
    });

    let underflow: Expression = Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::SignedMultiplicationDoesNotUnderflow,
        left: Box::new(lvalue.clone()),
        right: Box::new(rvalue.clone()),
    });

    Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::And,
        left: Box::new(overflow),
        right: Box::new(underflow),
    })
}

fn signed_div(size: u8, lvalue: &Expression, rvalue: &Expression) -> Expression {
    let condition = Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::And,
        left: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Equal,
                left: Box::new(lvalue.clone()),
                right: Box::new(
                    Expression::SignedBitVector( SignedBitVectorData{
                        size: size,
                        value: match size {
                            8u8 => i8::min_value() as i64,
                            16u8 => i16::min_value() as i64,
                            32u8 => i32::min_value() as i64,
                            64u8 => i64::min_value() as i64,
                            _ => panic!("unsupported integer type"),
                        },
                    })
                ),
            })
        ),
        right: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Equal,
                left: Box::new(rvalue.clone()),
                right: Box::new(
                    Expression::SignedBitVector( SignedBitVectorData{
                        size: size,
                        value: -1i64,
                    })
                )
            })
        ),
    });

    Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::And,
        left: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Implication,
                left: Box::new(condition.clone()),
                right: Box::new(
                    Expression::BooleanLiteral(false)
                ),
            })
        ),
        right: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Implication,
                left: Box::new(
                    Expression::UnaryExpression( UnaryExpressionData{
                        op: UnaryOperator::Not,
                        e: Box::new(condition.clone()),
                    })
                ),
                right: Box::new(
                    Expression::BooleanLiteral(true)
                ),
            })
        ),
    })
}

/// Routes to appropriate overflow check
// Unsigned: Match on the type of BinOp and call the correct function
fn unsigned_overflow(binop: &BinOp, lvalue: &Expression, rvalue: &Expression) -> Expression {
    match *binop {
        BinOp::Add => unsigned_add(lvalue, rvalue),
        BinOp::Sub => unsigned_sub(lvalue, rvalue),
        BinOp::Mul => unsigned_mul(lvalue, rvalue),
        BinOp::Div => unreachable!(),
        BinOp::Rem => unreachable!(),
        BinOp::Shl => unimplemented!(),
        BinOp::Shr => unimplemented!(),
        BinOp::BitOr => unimplemented!(),
        BinOp::BitAnd => unimplemented!(),
        BinOp::BitXor => unimplemented!(),
        BinOp::Lt => unimplemented!(),
        BinOp::Le => unimplemented!(),
        BinOp::Gt => unimplemented!(),
        BinOp::Ge => unimplemented!(),
        BinOp::Eq => unimplemented!(),
        BinOp::Ne => unimplemented!(),
    }
}

fn unsigned_mul(lvalue: &Expression, rvalue: &Expression) -> Expression {
    Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::UnsignedMultiplicationDoesNotOverflow,
        left: Box::new(lvalue.clone()),
        right: Box::new(rvalue.clone()),
    })
}

// l + r >= l
fn unsigned_add(lvalue: &Expression, rvalue: &Expression) -> Expression {
    Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::GreaterThanOrEqual,
        //l + r
        left: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Addition,
                left: Box::new(lvalue.clone()),
                right: Box::new(rvalue.clone()),
            })
        ),
        // r
        right: Box::new(rvalue.clone()),
    })
}

// l - r <= l
fn unsigned_sub(lvalue: &Expression, rvalue: &Expression) -> Expression {
    Expression::BinaryExpression( BinaryExpressionData{
        op: BinaryOperator::LessThanOrEqual,
        //l - r
        left: Box::new(
            Expression::BinaryExpression( BinaryExpressionData{
                op: BinaryOperator::Subtraction,
                left: Box::new(lvalue.clone()),
                right: Box::new(rvalue.clone()),
            })
        ),
        // r
        right: Box::new(rvalue.clone()),
    })
}
