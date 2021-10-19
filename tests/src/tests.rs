use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};

const MAX_CYCLES: u64 = 10_000_000;

const UDT_NOT_MATCH: i8 = 5;
const NOT_OWNER: i8 = 6;
const CANNOT_USE_AS_INPUT: i8 = 7;
const MULTIPLE_OUTPUTS: i8 = 8;
const DATA_LENGTH_NOT_ENOUGH: i8 = 9;

fn build_test_context(
    group_input: bool,
    group_output: bool,
    can_find_udt: bool,
    is_owner_mode: bool,
    data_length: bool
) -> (Context, TransactionView) {
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("hackathon");
    let contract_out_point = context.deploy_contract(contract_bin);

    let always_success_out_point = context.deploy_contract(ALWAYS_SUCCESS.clone());

    let lock_script_args = Default::default();

    let lock_script = context
        .build_script(&always_success_out_point, lock_script_args)
        .expect("always success lock script");

    let lock_hash : [u8; 32] = lock_script.calc_script_hash().unpack();
    let sudt_script_args : Bytes = if is_owner_mode {
        lock_hash.to_vec().into()
    } else {
        [0u8; 32].to_vec().into()
    };

    let sudt_script = context
        .build_script(&always_success_out_point, sudt_script_args)
        .expect("always success sudt script");

    let lock_sudt_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let type_hash : [u8; 32] = sudt_script.calc_script_hash().unpack();
    let hackathon_script_args : Bytes = if can_find_udt {
        type_hash.to_vec().into()
    } else {
        [0u8; 32].to_vec().into()
    };

    let hackathon_script = context
        .build_script(&contract_out_point, hackathon_script_args)
        .expect("hackathon script");

    let hackathon_script_dep = CellDep::new_builder()
        .out_point(contract_out_point)
        .build();

    let input_ckb = Capacity::bytes(1000).unwrap().as_u64();

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(input_ckb.clone().pack())
            .lock(lock_script.clone())
            .type_(Some(sudt_script.clone()).pack())
            .build(),
        [].to_vec().into(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let group_input_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(input_ckb.clone().pack())
            .lock(lock_script.clone())
            .type_(Some(hackathon_script.clone()).pack())
            .build(),
        [].to_vec().into(),
    );
    let group_input_input = CellInput::new_builder()
        .previous_output(group_input_input_out_point)
        .build();

    let output = CellOutput::new_builder()
            .capacity(input_ckb.pack())
            .lock(lock_script.clone())
            .type_(Some(hackathon_script.clone()).pack())
            .build();

    let group_output_output = CellOutput::new_builder()
        .capacity(input_ckb.pack())
        .lock(lock_script.clone())
        .type_(Some(hackathon_script.clone()).pack())
        .build();

    let hackathon_data: Bytes = if data_length {
        [1u8; 10].to_vec().into()
    } else {
        [1u8; 8].to_vec().into()
    };

    let mut outputs_data : Vec<Bytes> = vec!(hackathon_data.clone());

    if group_output {
        outputs_data.push(hackathon_data);
    }

    let mut tx = TransactionBuilder::default()
        .input(input)
        .output(output)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_sudt_script_dep)
        .cell_dep(hackathon_script_dep);

    if group_input {
        tx = tx.input(group_input_input);
    }

    if group_output {
        tx = tx.output(group_output_output);
    }

    (context, tx.build())
}

#[test]
fn test_success() {
    let (mut context, tx) = build_test_context(
        false,
        false,
        true,
        true,
        true
    );
    let tx = context.complete_tx(tx);

    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("should success");
    println!("cycles: {}", cycles);
}

#[test]
fn test_group_input() {
    let (mut context, tx) = build_test_context(
        true,
        false,
        true,
        true,
        true
    );
    let tx = context.complete_tx(tx);

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(CANNOT_USE_AS_INPUT));
}

#[test]
fn test_multiple_group_output() {
    let (mut context, tx) = build_test_context(
        false,
        true,
        true,
        true,
        true
    );
    let tx = context.complete_tx(tx);

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(MULTIPLE_OUTPUTS));
}

#[test]
fn test_cannot_find_udt() {
    let (mut context, tx) = build_test_context(
        false,
        false,
        false,
        true,
        true
    );
    let tx = context.complete_tx(tx);

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(UDT_NOT_MATCH));
}

#[test]
fn test_not_owner_of_udt() {
    let (mut context, tx) = build_test_context(
        false,
        false,
        true,
        false,
        true
    );
    let tx = context.complete_tx(tx);

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(NOT_OWNER));
}

#[test]
fn test_short_data_length() {
    let (mut context, tx) = build_test_context(
        false,
        false,
        true,
        true,
        false
    );
    let tx = context.complete_tx(tx);

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(DATA_LENGTH_NOT_ENOUGH));
}