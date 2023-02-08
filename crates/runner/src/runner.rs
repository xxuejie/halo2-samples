use ckb_mock_tx_types::{MockTransaction, ReprMockTransaction, Resource};
use ckb_script::{
    cost_model::{instruction_cycles, transferred_byte_cycles},
    ScriptGroupType, ScriptVersion, TransactionScriptsVerifier,
};
use ckb_types::core::cell::resolve_transaction;
use ckb_vm::{
    machine::asm::{AsmCoreMachine, AsmMachine},
    DefaultMachineBuilder, SupportMachine,
};
use clap::{arg, command, value_parser};
use flate2::read::GzDecoder;
use serde_json::from_str as from_json_str;
use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::sync::Arc;
use std::time::SystemTime;

fn main() {
    let matches = command!()
        .arg(arg!(--"tx-file" <VALUE>).required(true))
        .arg(
            arg!(--"cell-index" <VALUE>)
                .value_parser(value_parser!(u64))
                .required(true),
        )
        .arg(
            arg!(--"cell-type" <VALUE>)
                .value_parser(["input", "output"])
                .required(true),
        )
        .arg(
            arg!(--"script-type" <VALUE>)
                .value_parser(["lock", "type"])
                .required(true),
        )
        .get_matches();

    let tx_file = matches.get_one::<String>("tx-file").expect("tx file");
    let cell_index = *matches.get_one::<u64>("cell-index").expect("cell index");
    let cell_type = matches.get_one::<String>("cell-type").expect("cell type");
    let script_type = matches
        .get_one::<String>("script-type")
        .expect("script type");

    let content = if tx_file.ends_with(".gz") {
        let file = File::open(tx_file).expect("open");
        let mut gz = GzDecoder::new(file);
        let mut s = String::new();
        gz.read_to_string(&mut s).expect("gz read");
        s
    } else {
        read_to_string(tx_file).expect("read")
    };
    let repr_tx: ReprMockTransaction = from_json_str(&content).expect("json parsing");
    let mock_tx: MockTransaction = repr_tx.into();

    let verifier_resource = Resource::from_mock_tx(&mock_tx).expect("create resource");
    let resolved_tx = resolve_transaction(
        mock_tx.core_transaction(),
        &mut HashSet::new(),
        &verifier_resource,
        &verifier_resource,
    )
    .expect("resolve");

    let verifier = TransactionScriptsVerifier::new(Arc::new(resolved_tx), verifier_resource);

    let (group_type, script_hash) = match (cell_type.as_str(), script_type.as_str()) {
        ("input", "lock") => (
            ScriptGroupType::Lock,
            mock_tx.mock_info.inputs[cell_index as usize]
                .output
                .calc_lock_hash(),
        ),
        ("input", "type") => (
            ScriptGroupType::Type,
            mock_tx.mock_info.inputs[cell_index as usize]
                .output
                .type_()
                .to_opt()
                .expect("cell should have type script")
                .calc_script_hash(),
        ),
        ("output", "type") => (
            ScriptGroupType::Type,
            mock_tx
                .tx
                .raw()
                .outputs()
                .get(cell_index as usize)
                .expect("index out of bound")
                .type_()
                .to_opt()
                .expect("cell should have type script")
                .calc_script_hash(),
        ),
        _ => panic!(
            "Script {} {} {} shall not be executed!",
            cell_type, cell_index, script_type
        ),
    };

    let script_version = ScriptVersion::V1;

    let core_machine = AsmCoreMachine::new(
        script_version.vm_isa(),
        u32::MAX,
        u64::MAX,
    );
    let mut machine_builder = DefaultMachineBuilder::new(core_machine)
        .instruction_cycle_func(Box::new(instruction_cycles));
    let script_group = verifier
        .find_script_group(group_type, &script_hash)
        .expect("unknown group!");
    let program = verifier
        .extract_script(&script_group.script)
        .expect("program");
    let machine_syscalls = verifier.generate_syscalls(script_version, script_group);
    machine_builder = machine_syscalls
        .into_iter()
        .fold(machine_builder, |builder, syscall| builder.syscall(syscall));
    let mut machine = Box::pin(AsmMachine::new(machine_builder.build()));

    let (cycle, runtime) = {
        let a = SystemTime::now();
        let bytes = machine.load_program(&program, &[]).expect("load");
        let transferred_cycles = transferred_byte_cycles(bytes);
        machine
            .machine
            .add_cycles(transferred_cycles)
            .expect("add cycles");
        machine.run().expect("run");
        let b = SystemTime::now();
        let d = b.duration_since(a).expect("clock goes backwards");

        (machine.machine.cycles(), d)
    };

    println!("Cycles: {}", cycle);
    println!("Runtime: {:?}", runtime);
}
