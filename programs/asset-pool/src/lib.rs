use anchor_lang::prelude::*;
use anchor_spl::token;

declare_id!("DKtsagsHJeuBdFu2UWzmuEnEdr6izatQtjYLv65gMXeS");

#[error_code]
pub enum PoolError {
    #[msg("Invalid pool")]
    InvalidPool,
    #[msg("Invalid bump")]
    InvalidBump,
    #[msg("Invalid owner")]
    InvalidOwner,
}

const POOL_KEY: &[u8] = b"pool";
const TOKEN_KEY: &[u8] = b"token";

#[program]
pub mod scale_asset_pool {
    use super::*;

    pub fn create(ctx: Context<Create>, asset_pair: AssetPair, bump: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let seeds = &[
            POOL_KEY,
            ctx.accounts.owner.key.as_ref(),
            pool.pair.into(),
            &[bump],
        ];

        let addr = Pubkey::create_program_address(
            &seeds[..],
            &id(),
        )
        .map_err(|_| PoolError::InvalidBump)?;
        if addr != pool.key() {
            return Err(PoolError::InvalidBump.into());
        }

        if ctx.accounts.owner.key != &pool.owner {
            return Err(PoolError::InvalidOwner.into());
        }

        pool.pool_bump = bump;
        pool.vault = ctx.accounts.vault.key().clone();
        pool.mint = ctx.accounts.mint.key().clone();
        pool.initialized = true;
        pool.pair = asset_pair;
        pool.owner = ctx.accounts.owner.key().clone();

        msg!("pool created");
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        token::transfer(ctx.accounts.into_transfer_context(), amount)?;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let pool = &ctx.accounts.accounts.pool;
        let signers = &[
            POOL_KEY,
            pool.owner.as_ref(),
            pool.pair.into(),
            &[pool.pool_bump],
        ];
        token::transfer(
            ctx.accounts
                .into_transfer_context()
                .with_signer(&[&signers[..]]),
            amount,
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, AnchorDeserialize, AnchorSerialize)]
#[non_exhaustive]
pub enum AssetPair {
    BtcUsdc,
    EthUsdc,
}

impl Into<&[u8]> for AssetPair {
    fn into(self) -> &'static [u8] {
        match self {
            AssetPair::BtcUsdc => b"btcusdc",
            AssetPair::EthUsdc => b"ethusdc",
        }
    }
}

#[account]
pub struct PoolAccount {
    pub initialized: bool,
    pub pair: AssetPair,
    pub pool_bump: u8,
    pub owner: Pubkey,
    pub balance: u64,  // total balance in pool
    pub vault: Pubkey,
    pub mint: Pubkey,
}

impl PoolAccount {
    pub const LEN: usize = 1 + 1 + 1
        + 32
        + 8
        + 32
        + 32;
}

#[derive(Accounts)]
#[instruction(asset_pair: AssetPair)]
pub struct Create<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK:
    pub owner: UncheckedAccount<'info>,
    #[account(init,
        seeds = [owner.key().as_ref(), asset_pair.into(), POOL_KEY],
        bump,
        payer = payer,
        space = 8 + PoolAccount::LEN,
    )]
    pub pool: Account<'info, PoolAccount>,
    pub mint: Account<'info, token::Mint>,
    #[account(init,
        seeds = [pool.key().as_ref(), mint.key().as_ref(), TOKEN_KEY],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = pool,
    )]
    pub vault: Account<'info, token::TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, PoolAccount>,
    #[account(mut)]
    pub token: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub vault: Account<'info, token::TokenAccount>,
    pub token_program: Program<'info, token::Token>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub accounts: Transfer<'info>,
}

impl<'info> Deposit<'info> {
    /// transfer from payer to pool
    pub fn into_transfer_context<'a, 'b, 'c>(&self) -> CpiContext<'a, 'b, 'c, 'info, token::Transfer<'info>> {
        let cpi_program = self.accounts.token_program.to_account_info();
        let cpi_accounts = token::Transfer {
            from: self.accounts.token.to_account_info(),
            to: self.accounts.vault.to_account_info(),
            authority: self.accounts.payer.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub accounts: Transfer<'info>,
}

impl<'info> Withdraw<'info> {
    /// transfer from pool to payer, need signer seeds
    pub fn into_transfer_context<'a, 'b, 'c>(&self) -> CpiContext<'a, 'b, 'c, 'info, token::Transfer<'info>> {
        let cpi_program = self.accounts.token_program.to_account_info();
        let cpi_accounts = token::Transfer {
            from: self.accounts.vault.to_account_info(),
            to: self.accounts.token.to_account_info(),
            authority: self.accounts.pool.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
