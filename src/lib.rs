pub mod entrypoint;
pud mod instruction;
pud mod error;
pub mod processor;
pub mod state;

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

//entrypoint is where everything gets passed into and it takes a process instruction
entrypoint!(process_instruction);
fn process_instruction(program_id: &Pubkey,accounts: &[AccountInfo],instruction_data: &[u8],) -> ProgramResult {
    msg!("process_instruction: {}: {} accounts, data={:?}",program_id,accounts.len(),instruction_data);
    Ok(())
}
