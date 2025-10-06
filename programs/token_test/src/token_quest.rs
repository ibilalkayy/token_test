use anchor_lang::prelude::*;

use anchor_spl::token::{Mint, Token, TokenAccount};
pub const DECIMALS: u128 = 10000;

// Equivalent to your SafeMath constants
pub const MAX_FEE_PERCENTAGE: u64 = 300; // 3%
pub const STAKING_PERIOD: i64 = 60; // seconds
pub const LOCKING_PERIOD: i64 = 30; // seconds

pub fn safe_add(a: u128, b: u128) -> Result<u128> {
    a.checked_add(b)
        .ok_or_else(|| error!(ErrorCode::AdditionOverflow))
}

pub fn safe_sub(a: u128, b: u128) -> Result<u128> {
    a.checked_sub(b)
        .ok_or_else(|| error!(ErrorCode::SubtractionOverflow))
}

pub fn safe_mul(a: u128, b: u128) -> Result<u128> {
    if a == 0 || b == 0 {
        return Ok(0);
    }
    let c = a
        .checked_mul(b)
        .ok_or_else(|| error!(ErrorCode::MultiplicationOverflow))?;
    if c / a != b {
        return Err(error!(ErrorCode::MultiplicationOverflow));
    }
    Ok(c / DECIMALS)
}

pub fn safe_div(a: u128, b: u128) -> Result<u128> {
    if b == 0 {
        return Err(error!(ErrorCode::DivisionByZero));
    }
    Ok((a * DECIMALS) / b)
}

#[account]
pub struct TokenQuest {
    pub owner: Pubkey,               // Like Ownable
    pub bal: u64,                    // global balance
    pub fee_percentage: u64,         // fee %
    pub fee_taker: Pubkey,           // fee receiver
    pub accepted_token_mint: Pubkey, // like IERC20 in Solidity
    pub user_tax_on_deposit: bool,   // tax toggle
    pub user_tax_on_withdraw: bool,  // tax toggle
    pub last_deposit_ts: i64,        // last deposit timestamp
                                     // For simplicity: map-like behavior must be handled via PDAs, not raw mapping
}

// Equivalent to your ClientStake struct
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ClientStake {
    pub amount: u64,
    pub stake_timestamp: i64,
    pub is_native: bool,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<TokenQuest>(), // account size
    )]
    pub token_quest: Account<'info, TokenQuest>,

    #[account(mut)]
    pub payer: Signer<'info>, // who pays for init

    pub system_program: Program<'info, System>,
}

#[program]
pub mod token_quest {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        initial_fee_percentage: u64,
        fee_taker: Pubkey,
        accepted_token_mint: Pubkey,
        user_tax_on_deposit: bool,
        user_tax_on_withdraw: bool,
    ) -> Result<()> {
        require!(
            initial_fee_percentage <= MAX_FEE_PERCENTAGE,
            ErrorCode::FeeTooHigh
        );
        require!(fee_taker != Pubkey::default(), ErrorCode::InvalidFeeTaker);
        require!(
            accepted_token_mint != Pubkey::default(),
            ErrorCode::InvalidToken
        );

        let token_quest = &mut ctx.accounts.token_quest;
        token_quest.owner = ctx.accounts.payer.key(); // same as Ownable(msg.sender)
        token_quest.bal = 0;
        token_quest.fee_percentage = initial_fee_percentage;
        token_quest.fee_taker = fee_taker;
        token_quest.accepted_token_mint = accepted_token_mint;
        token_quest.user_tax_on_deposit = user_tax_on_deposit;
        token_quest.user_tax_on_withdraw = user_tax_on_withdraw;
        token_quest.last_deposit_ts = 0;

        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("SafeMath: addition overflow")]
    AdditionOverflow,
    #[msg("SafeMath: subtraction overflow")]
    SubtractionOverflow,
    #[msg("SafeMath: multiplication overflow")]
    MultiplicationOverflow,
    #[msg("SafeMath: division by zero")]
    DivisionByZero,

    #[msg("Initial fee exceeds maximum allowed")]
    FeeTooHigh,
    #[msg("Fee taker cannot be zero address")]
    InvalidFeeTaker,
    #[msg("Accepted token cannot be zero address")]
    InvalidToken,
}
