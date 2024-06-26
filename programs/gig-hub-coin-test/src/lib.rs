use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::Transfer;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use solana_program::account_info::AccountInfo;
use std::fmt;
use borsh::{BorshSerialize, BorshDeserialize};


declare_id!("GH1vm3L2rob7GzLLQCi5t9shgJDXSWCTA8zgJjhGNKXx");

#[program]
pub mod gig_hub_coin_test {


    use super::*;
    pub fn initcounter(_ctx: Context<CreateCounter>) -> Result<()> {
        msg!("counter got Initialised");

        Ok(())
    }

    pub fn initializestatepda(
        ctx: Context<Initialisedstatepda>,  // Context containing account information
        _bump: u8, // Bump seed for PDA generation
        price: u64, // Price to be set for the state
        assign_to: Pubkey, // Public key of the wallet to which the state is assigned
        fee_payer_freelancer: bool, // Boolean to determine who pays the fee (freelancer or employer)
    ) -> Result<()> {
        msg!("state got Initialised"); // Log message indicating the start of the function

         // Validate the price to ensure it's greater than 0
        if price <= 0 {
            return Err(MyError::InvalidPrice.into());
        }

        // This check is redundant since `price` is of type u64 and cannot exceed u64::MAX
        if price > u64::MAX {
            return Err(MyError::InvalidPrice.into());
        }

         // Ensure the owner's public key is not the default public key
        if ctx.accounts.owner.key() == Pubkey::default() {
            return Err(MyError::InvalidOwner.into());
        }

         // Check that the state is not assigned to the owner themselves
        if assign_to == ctx.accounts.owner.key() {
            return Err(MyError::InvalidAssignment.into());
        }

         // Validate the bump seed to ensure it does not exceed 10
        if _bump > 10 {
            return Err(MyError::InvalidBump.into());
        }

        let state_pda = &mut ctx.accounts.statepda; // Mutable reference to the state PDA account

        // Set the state PDA's properties
        state_pda.amount = price;
        state_pda.owner = ctx.accounts.owner.key();
        state_pda.assign_wallet = assign_to;
        ctx.accounts.counter_pda.count += 1; // Increment the counter PDA's count
        state_pda.assigned_counter = ctx.accounts.counter_pda.count; // Assign the incremented count to the state PDA
        state_pda.status = "Waiting".to_string(); // Set the initial status of the state PDA
        state_pda.fee_payer_freelancer = fee_payer_freelancer; // Set who pays the fee

        Ok(())
    }
    pub fn initialisetokenpda(ctx: Context<Initialisetokenpda>, _bump1: u8) -> Result<()> {
        msg!("token got Initialised");
        let pda = ctx.accounts.tokenpda.key();
        ctx.accounts.statepda.pda_ata_adress = Some(ctx.accounts.tokenpda.key());
        msg!("token pda : {}", pda);
        Ok(())
    }

    pub fn sendsoltopda(ctx: Context<SendSolPDA>) -> Result<()> {
        let state_account = &mut ctx.accounts.statepda;
        let signer = &mut ctx.accounts.owner;
        let fee_account = &mut ctx.accounts.fee_account_pubkey;
        let fee_account_key = "5Uw3sWy6oRu5Nt7jqcUVLqMzaQd9MdrpCfyXFYzCcA5h";
        let amount_for_fee = (state_account.amount / 100) * 7;
        let amount_for_pda = if state_account.fee_payer_freelancer {
            state_account.amount - amount_for_fee
        } else {
            state_account.amount
        };

        if amount_for_pda == 0 {
            return err!(MyError::InvalidCalculation);
        } else if amount_for_pda < 0 {
            return err!(MyError::InvalidCalculation);
        } else {
            if fee_account.key().to_string() == fee_account_key
                && state_account.assign_wallet.key() == signer.key()
                && state_account.status == "Waiting"
            {
                system_program::transfer(
                    CpiContext::new(
                        ctx.accounts.system_program.to_account_info(),
                        system_program::Transfer {
                            from: signer.to_account_info(),
                            to: state_account.to_account_info(),
                        },
                    ),
                    amount_for_pda + amount_for_fee,
                )?;

                state_account.pda_total_amount = amount_for_pda;
                state_account.status = "InProgress".to_string();
            } else {
                return err!(MyError::InvalidAccount);
            }
        }

        Ok(())
    }

    pub fn sendusdctopda(ctx: Context<SendTokenPDA>, _bump1: u8, _bump2: u8) -> Result<()> {
        msg!("token process start for PDA transfer...");
        let state = &mut ctx.accounts.statepda;
        state.bump = _bump1;
        let fee_account = &mut ctx.accounts.fee_account;
        let fee_account_key = "5Uw3sWy6oRu5Nt7jqcUVLqMzaQd9MdrpCfyXFYzCcA5h";
        let bump_vector = _bump1.to_le_bytes();

        let amount_for_fee = (state.amount / 100) * 7;
        let amount_for_pda = if state.fee_payer_freelancer {
            state.amount - amount_for_fee
        } else {
            state.amount + amount_for_fee
        };

        let sender = &ctx.accounts.owner;
        let inner = vec![
            sender.key.as_ref(),
            sender.key.as_ref(),
            "state".as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        if amount_for_pda < 0 {
            return err!(MyError::InvalidCalculation);
        } else {
            if fee_account.key().to_string() == fee_account_key
                && state.assign_wallet.key() == ctx.accounts.owner.key() && state.status == "Waiting"
            {
                //for account to PDA
                let transfer_instruction = Transfer {
                    from: ctx.accounts.deposit_token_account.to_account_info(),
                    to: ctx.accounts.tokenpda.to_account_info(),
                    authority: sender.to_account_info(),
                };

                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    transfer_instruction,
                    outer.as_slice(),
                );

                msg!("transfer call start");

                anchor_spl::token::transfer(cpi_ctx, amount_for_pda)?;
                state.pda_total_amount = amount_for_pda;

                //for account to fee account
                let transfer_instruction2 = Transfer {
                    from: ctx.accounts.deposit_token_account.to_account_info(),
                    to: fee_account.to_account_info(),
                    authority: sender.to_account_info(),
                };

                let cpi_ctx2 = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    transfer_instruction2,
                    outer.as_slice(),
                );

                msg!("transfer call start for fee");

                anchor_spl::token::transfer(cpi_ctx2, amount_for_fee)?;

                state.status = "InProgress".to_string();
                msg!("succesfully transfered");
            } else {
                return err!(MyError::InvalidAccount);
            }
        }

        Ok(())
    }

    pub fn sendusdctoreciever(ctx: Context<SendTokenWinner>, _bump1: u8, _bump2: u8) -> Result<()> {
        msg!("token transfer to reciever started from backend...");
        let assigned_counter_info = &ctx.accounts.statepda.assigned_counter.to_be_bytes();
        let bump_vector = _bump1.to_le_bytes();
        let amount = ctx.accounts.statepda.pda_total_amount;
        let sender = ctx.accounts.statepda.owner;
        let inner = vec![
            sender.as_ref(),
            assigned_counter_info.as_ref(),
            sender.as_ref(),
            "state".as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        if (ctx.accounts.statepda.status == Status::Canceled.to_string()
            && ctx.accounts.reciever.key() == ctx.accounts.statepda.assign_wallet.key())
            || (ctx.accounts.statepda.status == Status::Completed.to_string()
                && ctx.accounts.reciever.key() == ctx.accounts.statepda.owner.key())
        {
            let transfer_instruction = Transfer {
                from: ctx.accounts.tokenpda.to_account_info(),
                to: ctx.accounts.wallet_to_deposit_to.to_account_info(),
                authority: ctx.accounts.statepda.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_instruction,
                outer.as_slice(),
            );

            msg!("trasnfer call start");
            let state = &mut ctx.accounts.statepda;
            state.status = Status::ContractClosed.to_string();
            anchor_spl::token::transfer(cpi_ctx, amount)?;
        } else {
            return err!(MyError::InvalidAccount);
        }

        Ok(())
    }

    pub fn sendsoltoreciever(ctx: Context<SendSOLtoReciever>, _bump1: u8) -> Result<()> {
        if (ctx.accounts.statepda.status == "Cancaled"
            && ctx.accounts.owner.key() == ctx.accounts.statepda.assign_wallet.key())
            || (ctx.accounts.statepda.status == "Completed"
                && ctx.accounts.owner.key() == ctx.accounts.statepda.owner.key())
        {
            **ctx
                .accounts
                .statepda
                .to_account_info()
                .try_borrow_mut_lamports()? -= ctx.accounts.statepda.pda_total_amount;
            **ctx
                .accounts
                .owner
                .to_account_info()
                .try_borrow_mut_lamports()? += ctx.accounts.statepda.pda_total_amount;
            msg!("Send Sol from PDA to Reciever Completed");

            let state = &mut ctx.accounts.statepda;
            state.status = "ContractClosed".to_string();
        } else {
            return err!(MyError::InvalidAccount);
        }

        Ok(())
    }

    pub fn update_status(
        ctx: Context<UpdateStatus>,
        is_problem: bool,
        admin_account: Pubkey,
        solving_to: SolvingTo,
        ) -> Result<()> {
        let state_account = &mut ctx.accounts.statepda;
        let signer = &mut ctx.accounts.owner;

        if state_account.status == Status::InProgress.to_string()
            && state_account.owner.key() == signer.key()
            && !is_problem
        {
            state_account.status = Status::Canceled.to_string();
            msg!("Project cancelled by Employee");
        } else if state_account.status == Status::InProgress.to_string()
            && state_account.assign_wallet.key() == signer.key()
            && !is_problem
        {
            state_account.status = Status::Completed.to_string();
            msg!("Project completed by Employer");
        }

        if is_problem
            && (state_account.assign_wallet.key() == signer.key()
                || state_account.owner.key() == signer.key())
            && state_account.status == Status::InProgress.to_string()
            && admin_account.to_string() == "AiUUU6y6Axtb6v8EhdH6xnnPQoZ1wFQ4R2Z6U4Pm3fQM"
        {
            state_account.assigned_admin = admin_account;
            state_account.status = Status::ProblemSolving.to_string();
            msg!("Project status changed to Problem Solving");
        }

        if signer.key() == state_account.assigned_admin.key()
            && state_account.status == Status::ProblemSolving.to_string()
        {
            match solving_to {
                SolvingTo::Employee => {
                    state_account.status = Status::Completed.to_string();
                    msg!("Project problem solving by Admin (Receiver will be Employee)");
                }
                SolvingTo::Employer => {
                    state_account.status = Status::Canceled.to_string();
                    msg!("Project problem solving by Admin (Receiver will be Employer)");
                }
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct Initialisedstatepda<'info> {
    #[account(
            init,
            payer = owner,
            seeds=[owner.key.as_ref(),&(counter_pda.count + 1).to_le_bytes(),owner.key().as_ref(),"state".as_ref()],
            bump,
            space=200
        )]
    statepda: Account<'info, State>,
    #[account(
            mut,
            seeds = [b"counter"],
            bump,
        )]
    pub counter_pda: Account<'info, Counter>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct Initialisetokenpda<'info> {
    #[account(
            init,
            seeds = [owner.key.as_ref(),&(statepda.assigned_counter).to_le_bytes() ,owner.key().as_ref()],
            bump,
            payer = owner,
            token::mint = mint,
            token::authority = statepda,
        )]
    pub tokenpda: Account<'info, TokenAccount>,

    #[account(
            mut,
            seeds = [owner.key.as_ref(),&(statepda.assigned_counter).to_le_bytes() ,owner.key().as_ref(), "state".as_ref()],
            bump,
        )]
    pub statepda: Account<'info, State>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub deposit_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SendSolPDA<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub statepda: Account<'info, State>,
    //iknow its problem
    #[account(mut)]
    /// CHECK: asdasd
    pub fee_account_pubkey: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct SendTokenPDA<'info> {
    #[account(mut)]
    pub tokenpda: Account<'info, TokenAccount>,
    #[account(
        mut,
    )]
    pub statepda: Account<'info, State>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub deposit_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateCounter<'info> {
    #[account(
            init,
            payer = payer,
            space = 100,
            seeds = [b"counter"],
            bump,
        )]
    pub counter_pda: Account<'info, Counter>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendTokenWinner<'info> {
    #[account(mut)]
    pub tokenpda: Account<'info, TokenAccount>,
    #[account(mut)]
    pub statepda: Account<'info, State>,
    #[account(mut)]
    pub wallet_to_deposit_to: Account<'info, TokenAccount>,

    //pub deposit_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK not read write to this account
    pub reciever: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateStatus<'info> {
    #[account(mut)]
    pub statepda: Account<'info, State>,
    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendSOLtoReciever<'info> {
    #[account(
        mut
    )]
    pub statepda: Account<'info, State>,
    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct State {
    bump: u8,
    amount: u64,
    assign_wallet: Pubkey,
    owner: Pubkey,
    status: String,
    assigned_counter: u8,
    pda_ata_adress: Option<Pubkey>,
    fee_payer_freelancer: bool,
    assigned_admin: Pubkey,
    pda_total_amount: u64,
}

#[account]
#[derive(Default)]
pub struct Counter {
    count: u8,
}

#[error_code]
pub enum MyError {
    #[msg("Accounts doenst't mach")]
    InvalidAccount,
    #[msg("Amount couldn't calculated")]
    InvalidCalculation,
    InvalidPrice,
    InvalidAssignment,
    InvalidBump,
    InvalidOwner
}

pub enum Status {
    Waiting,
    InProgress,
    Completed,
    Canceled,
    ContractClosed,
    ProblemSolving
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum SolvingTo {
    Employee = 1,
    Employer = 2,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Status::Waiting => "Waiting",
            Status::InProgress => "InProgress",
            Status::Completed => "Completed",
            Status::Canceled => "Canceled",
            Status::ContractClosed => "ContractClosed",
            Status::ProblemSolving => "ProblemSolving",
        })
    }
}