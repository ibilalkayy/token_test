use anchor_lang::prelude::*;

pub fn initialize(
    ctx: Context<Initialize>,
    name: String,
    symbol: String,
    initial_supply: u128,
) -> Result<()> {
    let token = &mut ctx.accounts.token;
    token.name = name;
    token.symbol = symbol;
    token.total_supply = initial_supply;

    let owner_balance = &mut ctx.accounts.owner_balance;
    owner_balance.owner = *ctx.accounts.owner.key;
    owner_balance.amount = initial_supply;

    Ok(())
}

pub fn name(ctx: Context<GetToken>) -> Result<()> {
    let token = &ctx.accounts.token;
    msg!("Token name: {}", token.name);
    Ok(())
}

pub fn symbol(ctx: Context<GetToken>) -> Result<()> {
    let token = &ctx.accounts.token;
    msg!("Total symbol: {}", token.symbol);
    Ok(())
}

pub fn decimals(ctx: Context<GetToken>) -> Result<()> {
    let token = &ctx.accounts.token;
    msg!("Total Decimals: {}", token.decimals);
    Ok(())
}

pub fn total_supply(ctx: Context<GetToken>) -> Result<()> {
    let token = &ctx.accounts.token;
    msg!("Total supply: {}", token.total_supply);
    Ok(())
}

pub fn balance_of(ctx: Context<GetBalance>) -> Result<(u128)> {
    let balance = &ctx.accounts.balance;
    msg!("Total supply: {}", balance.amount);
    Ok(balance.amount)
}

pub fn transfer(ctx: Context<Transfer>, amount: u128) -> Result<()> {
    let sender = &mut ctx.accounts.sender;
    let recepient = &mut ctx.accounts.recepient;

    if sender.amount < amount {
        return Err(MyError::InsufficientFunds.into());
    }

    sender.amount = sender.amount.checked_sub(amount).unwrap();
    recepient.amount = recepient.amount.checked_add(amount).unwrap();

    msg!(
        "Transferred {} from {} to {}",
        amount,
        sender.owner,
        recepient.owner
    );

    Ok(())
}

pub fn allowance(ctx: Context<GetAllowance>) -> Result<u128> {
    let allowance = &ctx.accounts.allowance;
    msg!(
        "Allowance of spender {} for owner {} is {}",
        allowance.spender,
        allowance.owner,
        allowance.amount
    );
    Ok(allowance.amount)
}

pub fn approve(ctx: Context<Approve>, amount: u128) -> Result<()> {
    let allowance = &mut ctx.accounts.allowance;

    allowance.owner = ctx.accounts.owner.key();
    allowance.spender = ctx.accounts.spender.key();
    allowance.amount = amount;

    Ok(())
}

pub fn transfer_from(ctx: Context<TransferFrom>, amount: u128) -> Result<()> {
    let sender_balance = &mut ctx.accounts.sender_balance;
    let recipient_balance = &mut ctx.accounts.recipient_balance;
    let allowance = &mut ctx.accounts.allowance;

    // Check allowance first
    require!(allowance.amount >= amount, MyError::AllowanceExceeded);

    // Check sender has enough balance
    require!(sender_balance.amount >= amount, MyError::InsufficientFunds);

    // Do the transfer
    sender_balance.amount = sender_balance.amount.checked_sub(amount).unwrap();

    recipient_balance.amount = recipient_balance.amount.checked_add(amount).unwrap();

    // Decrease allowance
    allowance.amount = allowance.amount.checked_sub(amount).unwrap();

    msg!(
        "Transferred {} from {} to {} by spender {}",
        amount,
        sender_balance.owner,
        recipient_balance.owner,
        ctx.accounts.spender.key()
    );

    Ok(())
}

#[event]
pub struct TransferEvent {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct AnotherTransfer<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(mut)]
    pub from_account: Account<'info, TokenAccountState>,

    #[account(mut)]
    pub to_account: Account<'info, TokenAccountState>,
}

pub fn another_transfer(ctx: Context<AnotherTransfer>, amount: u64) -> Result<()> {
    let from_acc = &mut ctx.accounts.from_account;
    let to_acc = &mut ctx.accounts.to_account;

    // Check for "zero address" (in Solana, just check it's not default Pubkey::default())
    require!(
        from_acc.owner != Pubkey::default(),
        CustomError::ZeroAddress
    );
    require!(to_acc.owner != Pubkey::default(), CustomError::ZeroAddress);

    // Balance check
    require!(from_acc.balance >= amount, CustomError::InsufficientBalance);

    // Subtract & Add
    from_acc.balance = from_acc
        .balance
        .checked_sub(amount)
        .ok_or(CustomError::MathUnderflow)?;
    to_acc.balance = to_acc
        .balance
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;

    // Emit Transfer Event
    emit!(TransferEvent {
        from: from_acc.owner,
        to: to_acc.owner,
        amount,
    });

    Ok(())
}

#[account]
pub struct MintState {
    pub total_supply: u64,
}

#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(mut)]
    pub mint_state: Account<'info, MintState>,

    #[account(mut)]
    pub to_account: Account<'info, TokenAccountState>,
}

pub fn mint(ctx: Context<Mint>, amount: u64) -> Result<()> {
    let mint_state = &mut ctx.accounts.mint_state;
    let to_acc = &mut ctx.accounts.to_account;

    // Require non-zero address (Pubkey::default() = 111...111)
    require!(to_acc.owner != Pubkey::default(), CustomError::ZeroAddress);

    // Increase total supply
    mint_state.total_supply = mint_state
        .total_supply
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;

    // Add balance
    to_acc.balance = to_acc
        .balance
        .checked_add(amount)
        .ok_or(CustomError::MathOverflow)?;

    // Emit Transfer event (from "zero address")
    emit!(TransferEvent {
        from: Pubkey::default(), // zero address
        to: to_acc.owner,
        amount,
    });

    Ok(())
}

///

#[event]
pub struct ApprovalEvent {
    pub owner: Pubkey,
    pub spender: Pubkey,
    pub amount: u128,
}

#[derive(Accounts)]
pub struct AnotherApprove<'info> {
    /// Owner (must sign)
    #[account(mut, signer)]
    pub owner: AccountInfo<'info>,

    /// Spender (does not sign, just a pubkey)
    /// CHECK: Not dangerous, only used as pubkey
    pub spender: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + 32 + 32 + 8 + 1, // discriminator + owner + spender + amount + bump
        seeds = [b"allowance", owner.key().as_ref(), spender.key().as_ref()],
        bump
    )]
    pub allowance: Account<'info, Allowance>,

    pub system_program: Program<'info, System>,
}

pub fn another_approve(ctx: Context<Approve>, amount: u128) -> Result<()> {
    let allowance = &mut ctx.accounts.allowance;

    // Require valid addresses
    require!(
        ctx.accounts.owner.key() != Pubkey::default(),
        CustomError::ZeroAddress
    );
    require!(
        ctx.accounts.spender.key() != Pubkey::default(),
        CustomError::ZeroAddress
    );

    // Set allowance
    allowance.owner = ctx.accounts.owner.key();
    allowance.spender = ctx.accounts.spender.key();
    allowance.amount = amount;

    // Save bump
    let (_pda, bump) = Pubkey::find_program_address(
        &[
            b"allowance",
            ctx.accounts.owner.key().as_ref(),
            ctx.accounts.spender.key().as_ref(),
        ],
        ctx.program_id,
    );
    allowance.bump = bump;

    // Emit event (Approval)
    emit!(ApprovalEvent {
        owner: allowance.owner,
        spender: allowance.spender,
        amount,
    });

    Ok(())
}

#[error_code]
pub enum CustomError {
    #[msg("ERC20: transfer from the zero address")]
    ZeroAddress,
    #[msg("ERC20: transfer amount exceeds balance")]
    InsufficientBalance,
    #[msg("Math underflow")]
    MathUnderflow,
    #[msg("Math overflow")]
    MathOverflow,
}

#[error_code]
pub enum MyError {
    #[msg("Insufficient funds for this transfer.")]
    InsufficientFunds,
    #[msg("Allowance exceeded.")]
    AllowanceExceeded,
}

#[account]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub total_supply: u128,
    pub decimals: u8,
}

#[account]
pub struct TokenAccountState {
    pub owner: Pubkey,
    pub balance: u64,
}

#[account]
pub struct Allowance {
    pub owner: Pubkey,
    pub spender: Pubkey,
    pub amount: u128,
    pub bump: u8,
}

#[account]
pub struct Balance {
    pub owner: Pubkey,
    pub amount: u128,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space = 8 + 32 + 32 + 8)]
    pub token: Account<'info, Token>,

    #[account(init, payer = owner, space = 8 + 32 + 8)]
    pub owner_balance: Account<'info, Balance>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(mut, signer)]
    pub owner: AccountInfo<'info>, // equivalent to msg.sender
    /// CHECK: spender doesn’t need to sign
    pub spender: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + 32 + 32 + 8, // discriminator + 2 pubkeys + u64
        seeds = [b"allowance", owner.key().as_ref(), spender.key().as_ref()],
        bump
    )]
    pub allowance: Account<'info, Allowance>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferFrom<'info> {
    #[account(mut, has_one = owner)]
    pub sender_balance: Account<'info, Balance>,

    #[account(mut)]
    pub recipient_balance: Account<'info, Balance>,

    pub spender: Signer<'info>,

    /// CHECK: only used for matching has_one = owner
    pub owner: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"allowance", sender_balance.owner.as_ref(), spender.key().as_ref()],
        bump = allowance.bump
    )]
    pub allowance: Account<'info, Allowance>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut, has_one = owner)]
    pub sender: Account<'info, Balance>,

    #[account(mut)]
    pub recepient: Account<'info, Balance>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetToken<'info> {
    pub token: Account<'info, Token>,
}

#[derive(Accounts)]
pub struct GetBalance<'info> {
    pub balance: Account<'info, Balance>,
}

#[derive(Accounts)]
pub struct GetAllowance<'info> {
    pub allowance: Account<'info, Allowance>,
}
