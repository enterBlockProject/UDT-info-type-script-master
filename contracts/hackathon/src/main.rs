#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

use core::result::Result;

use ckb_std::{
    entry,
    default_alloc,
    high_level::{
        load_script,
        load_cell_data,
        load_cell_type,
        load_cell_type_hash,
        load_cell_lock_hash,
        QueryIter
    },
    error::SysError,
    ckb_types::{bytes::Bytes, prelude::*},
    ckb_constants::Source,
};

entry!(entry);
default_alloc!();

fn entry() -> i8 {
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UDTNotMatch,
    NotOwner,
    CannotUseAsInput,
    MultipleOutputs,
    DataLengthNotEnough
}

const SYMBOL_LEN: usize = 8;
const DECIMAL_LEN: usize = 1;

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

fn count_group_input() -> Result<bool, Error> {
    let zero_input = load_cell_lock_hash(0, Source::GroupInput).is_err();
    Ok(zero_input)
}

fn count_group_output() -> Result<(bool, usize), Error> {
    let first_output_data = load_cell_data(0, Source::GroupOutput);
    Ok((first_output_data.is_ok() && load_cell_data(1, Source::GroupOutput).is_err(), first_output_data.unwrap().len()))
}

fn find_udt_cell_idx(args: &Bytes) -> Result<Option<usize>, Error> {
    let idx = QueryIter::new(load_cell_type_hash, Source::Input)
        .position(|type_hash| args[..] == type_hash[..]);
    Ok(idx)
}

fn check_owner_mode(args: &Bytes) -> Result<bool, Error> {
    let is_owner_mode = QueryIter::new(load_cell_lock_hash, Source::Input)
        .find(|lock_hash| args[..] == lock_hash[..]).is_some();
    Ok(is_owner_mode)
}

fn check_data_length(group_output_data: usize) -> Result<bool, Error> {
    Ok(group_output_data > SYMBOL_LEN + DECIMAL_LEN)
}

fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();

    if !count_group_input()? {
        return Err(Error::CannotUseAsInput);
    }

    let (group_output_is_ok, group_output_data) = count_group_output()?;

    if !group_output_is_ok {
        return Err(Error::MultipleOutputs);
    }

    let udt_cell_idx = if let Some(idx) = find_udt_cell_idx(&args)? {
        idx
    } else {
        return Err(Error::UDTNotMatch);
    };

    let udt_args: Bytes = load_cell_type(udt_cell_idx, Source::Input).unwrap().args().unpack();

    if !check_owner_mode(&udt_args)? {
        return Err(Error::NotOwner);
    }

    if !check_data_length(group_output_data)? {
        return Err(Error::DataLengthNotEnough);
    }

    Ok(())
}
