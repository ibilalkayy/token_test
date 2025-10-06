pub mod erc20_token;
pub mod token_quest;
use anchor_lang::prelude::*;

declare_id!("FvL8EsJaexUA9K5rsS8eXJsQB81ftvadprKp7MaMs7KL");

#[program]
mod counter {
    use super::*;

    pub fn initialize(ctx: Context<InitializeCounter>) -> Result<()> {
        ctx.accounts.count.number = 0; // start from 0
        Ok(())
    }

    pub fn set_number(ctx: Context<UpdateCounter>, new_number: u64) -> Result<()> {
        ctx.accounts.count.number = new_number;
        Ok(())
    }

    pub fn increment(ctx: Context<UpdateCounter>) -> Result<()> {
        ctx.accounts.count.number = ctx.accounts.count.number + 1;
        Ok(())
    }
}

#[account]
pub struct Count {
    pub number: u64,
}

#[derive(Accounts)]
pub struct InitializeCounter<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub count: Account<'info, Count>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    #[account(mut)]
    pub count: Account<'info, Count>,
}
