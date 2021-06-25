use std::convert:TryInto;
use crate::error::EscrowError::InvalidInstruction;
use solana_program::program_error::ProgramError;

pub mod entrypoint;

//instruction doesnt actually touch any accounts it just tells you what accounts to expect and where the calling info is




pub enum EscrowInstruction{
//makes an escrow account and gives ownership to a program derived address
//program derived address is like a contract address in solidity if you forget

//accounts that it is going to create:
//1[signer] the account of the dude initiating the transaction this is needed to transfer ownership 
//for the temp account
//2[writable] temporary account for Token X made prior to instruction and owned by the initializer 
//needs to be writable because account ownership is in the data field
//3[read-only] the initializer's token account for Token Y which they get
//4[writable] the actual escrow account with all the info needs to be writable to write info to it
//5[read-only] the rent system variable that will probably not even apply to this
//6[read-only] the actual token program of whatever you are creating
 
 //amount the first person(initializer) expects of the Token Y for her X
InitializeEscrow{AmountExpected: u64}

//accepts a trade at this point
//these are the expected accounts


// the amount the second person(taker) expects to be paid in the other token, Token X in this case
Exchange{AmountExpected: u64}


impl EscrowInstruction {
    /// since everything gets encoded, this unpacks a byte buffer into an escrow instruction and the amount
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            //match looks at the first byte to figure out how to decode this into
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,},_ => return Err(InvalidInstruction.into()),})
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}


