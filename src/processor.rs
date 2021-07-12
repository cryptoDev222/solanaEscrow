use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
};
use solana::hash::Hash;
use crate::{instruction::EscrowInstruction, error::EscrowError, state::Escrow};
use spl_token::state::Account as TokenAccount;

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        match instruction {
            EscrowInstruction::InitializeEscrow { amount } => {
                msg!("Instruction: InitializeEscrow");
                Self::process_init_escrow(accounts, amount, program_id)
            },
            EscrowInstruction::Exchange { amount } => {
                msg!("Instruction: Exchange");
                Self::process_exchange(accounts, amount, program_id)
            },
            EscrowInstruction::Bid {amount} => {
                Self::process_bid(accounts, BidAmount, program_id)
            }
        }
    }    
    //you iterate through a bunch the list of accounts and check to see if the first 1 is the signer
    //if it is not the initializer account then you throw an error
    fn process_init_escrow(accounts: &[AccountInfo], amount: u64,program_id: &Pubkey, ) -> ProgramResult {
        //accounts need to be mutable so it can take ones out of it
        let account_info_iter = &mut accounts.iter();
        //1st account
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer { 
            return Err(ProgramError::MissingRequiredSignature);
         }
         //2nd account
        let temp_token_account = next_account_info(account_info_iter)?;
        //3rd account
        let token_to_receive_account = next_account_info(account_info_iter)?;
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        //4th account
    let escrow_account = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    //if you have a bunch of money that they arent even going to charge you rent
    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
        return Err(EscrowError::NotRentExempt.into());
    }

    let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
    if escrow_info.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    escrow_info.is_initialized = true;
    escrow_info.initializer_pubkey = *initializer.key;
    escrow_info.temp_token_account_pubkey = *temp_token_account.key;
    escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
    escrow_info.expected_amount = amount;
    escrow_info.highest_bid = 0;

    Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;
    //bump seed to kick it off the ed25519 curve
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        let token_program = next_account_info(account_info_iter)?;
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;
        
        msg!("Calling the token program to transfer token account ownership...");
        invoke(&owner_change_ix, &[temp_token_account.clone(),initializer.clone(),token_program.clone(),],)?;
        Ok(())
}

    // inside: impl Processor {}
fn process_exchange(accounts: &[AccountInfo],amount_expected_by_taker: u64, expected_hash: [u8: 32],program_id: &Pubkey,)
 -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    //First account: Signer
    let taker = next_account_info(account_info_iter)?;
    if !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    //Second account taker's Goldeneye 007
    let takers_sending_token_account = next_account_info(account_info_iter)?;
    //Third account: taker's nintendo 64 they'll be getting
    let takers_token_to_receive_account = next_account_info(account_info_iter)?;
    //Fourth account: PDA temp escrow
    let pdas_temp_token_account = next_account_info(account_info_iter)?;
    let pdas_temp_token_account_info = TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;
    let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

    if amount_expected_by_taker != pdas_temp_token_account_info.amount {
        return Err(EscrowError::ExpectedAmountMismatch.into());
    }

    if amount_expected_by_taker <= pdas_tem_token_account_info.highest_bid{
        return Err(EscrowError::BrokeBoy.into());
    }
    //5th account: Initializer's main account
    let initializers_main_account = next_account_info(account_info_iter)?;
    //6th account: Initializer's account for Goldeneye 007
    let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
    //7th acount: Escrow account holding the info
    let escrow_account = next_account_info(account_info_iter)?;
    let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

    if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_pubkey != *initializers_main_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_token_to_receive_account_pubkey
        != *initializers_token_to_receive_account.key
    {
        return Err(ProgramError::InvalidAccountData);
    }
    //8th account: read only token
    let token_program = next_account_info(account_info_iter)?;

    let transfer_to_initializer_ix = spl_token::instruction::transfer(
        token_program.key,
        takers_sending_token_account.key,
        initializers_token_to_receive_account.key,
        taker.key,
        &[&taker.key],
        escrow_info.expected_amount,
    )?;
    msg!("Calling the token program to transfer tokens to the escrow's initializer...");
    invoke(
        &transfer_to_initializer_ix,
        &[  
            takers_sending_token_account.clone(),
            initializers_token_to_receive_account.clone(),
            taker.clone(),
            token_program.clone(),
        ],
    )?;
    //9th account: PDA account
    let pda_account = next_account_info(account_info_iter)?;

    let transfer_to_taker_ix = spl_token::instruction::transfer(
        token_program.key,
        pdas_temp_token_account.key,
        takers_token_to_receive_account.key,
        &pda,
        &[&pda],
        pdas_temp_token_account_info.amount,
    )?;
    msg!("Calling the token program to transfer tokens to the taker...");
    invoke_signed(
        &transfer_to_taker_ix,
        &[
            pdas_temp_token_account.clone(),
            takers_token_to_receive_account.clone(),
            pda_account.clone(),
            token_program.clone(),
        ],
        &[&[&b"escrow"[..], &[nonce]]],
    )?;

    let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
        token_program.key,
        pdas_temp_token_account.key,
        initializers_main_account.key,
        &pda,
        &[&pda],
    )?;
    msg!("Calling the token program to close pda's temp account...");
    invoke_signed(
        &close_pdas_temp_acc_ix,
        &[
            pdas_temp_token_account.clone(),
            initializers_main_account.clone(),
            pda_account.clone(),
            token_program.clone(),
        ],
        &[&[&b"escrow"[..], &[nonce]]],
    )?;

    msg!("Closing the escrow account...");
    **initializers_main_account.lamports.borrow_mut() = initializers_main_account
        .lamports()
        .checked_add(escrow_account.lamports())
        .ok_or(EscrowError::AmountOverflow)?;
    **escrow_account.lamports.borrow_mut() = 0;
    *escrow_account.data.borrow_mut() = &mut [];

    Ok(())
}

fn process_bid(accounts: &[AccountInfo],amount_expected_by_taker: u64, currentbid: u64, randomHash: [u8; 32],program_id: &Pubkey,)
 -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    //First account: Signer
    let taker = next_account_info(account_info_iter)?;
    if !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    //Second account taker's Goldeneye 007
    let takers_sending_token_account = next_account_info(account_info_iter)?;
    //Third account: taker's nintendo 64 they'll be getting
    let takers_token_to_receive_account = next_account_info(account_info_iter)?;
    //Fourth account: PDA temp escrow
    let pdas_temp_token_account = next_account_info(account_info_iter)?;
    let pdas_temp_token_account_info = TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;
    let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

    if amount_expected_by_taker != pdas_temp_token_account_info.amount {
        return Err(EscrowError::ExpectedAmountMismatch.into());
    }

    if currentbid <= pdas_tem_token_account_info.highest_bid{
        return Err(EscrowError::BrokeBoy.into());
    }
    //5th account: Initializer's main account
    let initializers_main_account = next_account_info(account_info_iter)?;
    //6th account: Initializer's account for Goldeneye 007
    let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
    //7th acount: Escrow account holding the info
    let escrow_account = next_account_info(account_info_iter)?;
    let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

    if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_pubkey != *initializers_main_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if escrow_info.initializer_token_to_receive_account_pubkey
        != *initializers_token_to_receive_account.key
    {
        return Err(ProgramError::InvalidAccountData);
    }
    //8th account: read only token
    let token_program = next_account_info(account_info_iter)?;

    let transfer_to_initializer_ix = spl_token::instruction::transfer(
        token_program.key,
        takers_sending_token_account.key,
        initializers_token_to_receive_account.key,
        taker.key,
        &[&taker.key],
        escrow_info.expected_amount,
    )?;
    //9th account: PDA account
    let pda_account = next_account_info(account_info_iter)?;

    let transfer_to_taker_ix = spl_token::instruction::transfer(
        token_program.key,
        pdas_temp_token_account.key,
        takers_token_to_receive_account.key,
        &pda,
        &[&pda],
        pdas_temp_token_account_info.amount,
    )?;

    Ok(())
    }
}

