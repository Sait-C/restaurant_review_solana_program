pub mod state;
pub mod instruction;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_10::try_from_slice_unchecked,
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Address of the currently executing program
    accounts: &[AccountInfo], // Array of accounts needed to execute an instruction.
    instruction_data: &[u8], // Serialized data specific to an instruction.
) -> ProgramResult {
    
    // The instruction_data passed into the entrypoint is deserialized to determine its corresponding enum variant.
    let instruction = ReviewInstruction::unpack(instruction_data)?;
    match instruction {
        ReviewInstruction::AddReview {
            title: String,
            rating: u8,
            description: String,
        } => add_review(program_id, accounts, title, rating, description), // instruction handler
        ReviewInstruction::UpdateReview {
            title: String,
            rating: u8,
            description: String,
        } => update_review(program_id, accounts, title, rating, description), // instruction handler
    }
};

// instruction handler
// implements the logic required to execute that instruction
pub fn add_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    rating: u8,
    description: String
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // The next_account_info function is used to access the next item in the iterator. 
    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignuture);
    }

    // SEEDS CONTROL
    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), title.as_bytes().as_ref()],
    program_id,);

    if pda != *pda_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    if rating > 10 || rating < 1 {
        return Err(ProgramError::InvalidRating.into());
    }

    let account_len:usize = 1000;

    let rent = Rent::get();
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key, 
            pda_account.key, 
            rent_lamports, 
            account_len.try_into().unwrap(), 
            program_id),
    &[
        initializer.clone(), 
        pda_account.clone(), 
        system_program.clone()],
    &[&[
        initializer.key.as_ref(),
        title.as_bytes().as_ref(),
        &[bump_seed],
    ]])?;

    let mut account_data = trye_from_slice_unchecked::<AccountState>(&pda_account.data.borrow()).unwrap();

    if account_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.title = title;
    account_data.description = description;
    account_data.rating = rating;
    account_data.is_initialized = true;

    // After the account has been successfully created, the final step is to serialize data into the new account's data fields. 
    // This effectively initializes the account data, storing the data passed into the program entrypoint.
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(());
}

// instruction handler
pub fn update_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _title: String,
    rating: u8,
    description: String
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter);
    let pda_account = next_account_info(account_info_iter);

    if pda_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignuture);
    }

    let mut account_data = try_from_slice_unchecked<AccountState>(&pda_account.data.borrow()).unwrap();

    // SEEDS CONTROL
    let (pda, _bump_seed) = Pubkey::find_program_address(
        &[
            initializer.key.as_ref(), 
            account_data.title.as_bytes().as_ref()
        ], 
        program_id,
    );

    if pda != *pda_account.key {
        return Err(ReviewError::InvalidPDA.into());
    }

    if !account_data.is_initialized() {
        return Err(ReviewError::UninitializedAccount.into());
    }

    if rating > 10 || rating < 1 {
        return Err(ReviewError::InvalidRating.into());
    }

    account_data.description = description;
    account_data.rating = rating;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}