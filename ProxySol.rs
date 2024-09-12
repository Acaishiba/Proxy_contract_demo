use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer as TokenTransfer, TokenAccount, Token};
use anchor_lang::solana_program::{system_instruction, program::invoke_signed};

declare_id!("6vk1y983dXQwsc68DcnrPwo7zYscsbx6nYghApTR82Vf");

#[program]
mod solana_proxy {
    use super::*;

    // Initialize the ProxyAccount and set the deployer as the owner
    pub fn initialize_proxy_account(ctx: Context<InitializeProxyAccount>) -> Result<()> {
        let proxy_account = &mut ctx.accounts.proxy_account;
        proxy_account.aa_accounts = Vec::new();  // Initialize with an empty AA account list
        proxy_account.owner = ctx.accounts.deployer.key(); // Set the owner as the deployer
        Ok(())
    }

    // Create an AA account using PDA (Program Derived Address)
    pub fn create_pda_aa_account(ctx: Context<CreatePDAAccount>, bump: u8) -> Result<()> {
        let proxy_account = &mut ctx.accounts.proxy_account;
        let pda_account = &mut ctx.accounts.pda_account;

        proxy_account.aa_accounts.push(pda_account.to_account_info().key());  // Add the newly created PDA AA account public key to the list
        pda_account.bump = bump;  // Store the PDA bump value
        pda_account.owner = *ctx.program_id;  // Set the PDA owner to the program ID itself
        Ok(())
    }

    // Control the AA account via the deployer, perform SOL or SPL Token transfers
    pub fn control_aa_account(ctx: Context<ControlAAAccount>, amount: u64, action: u8) -> Result<()> {
        let proxy_account = &ctx.accounts.proxy_account;
        let pda_account = &ctx.accounts.pda_account;
        let from_token_account = &ctx.accounts.from_token_account;

        // 仅输出 pda_account 和 from_token_account 的所有者，其他账户不检查
        msg!("PDAAccount owner: {}", pda_account.owner);
        msg!("FromTokenAccount owner: {}", from_token_account.owner);

        // 确保只有合约所有者（deployer）可以执行操作
        if ctx.accounts.deployer.key() != proxy_account.owner {
            return Err(ProgramError::IllegalOwner.into());
        }

        // 确保 pda_account 的所有权属于当前程序
        if pda_account.owner != *ctx.program_id {
            return Err(ProgramError::IllegalOwner.into());
        }

        // 如果操作是 SPL Token 转账，确保 from_token_account 权限正确
        if action == 2 && from_token_account.owner != *ctx.program_id {
            return Err(ProgramError::IllegalOwner.into());
        }

        // 使用 longer lifetime 的 proxy_account_key
        let proxy_account_key = proxy_account.key();
        let seeds = &[b"aa_account".as_ref(), proxy_account_key.as_ref(), &[pda_account.bump]];
        let signer = &[&seeds[..]];

        match action {
            // 执行 SOL 转账
            1 => {
                let transfer_instruction = system_instruction::transfer(
                    &ctx.accounts.pda_account.to_account_info().key(),
                    &ctx.accounts.to_account.key(),
                    amount,
                );
                invoke_signed(
                    &transfer_instruction,
                    &[
                        ctx.accounts.pda_account.to_account_info(),
                        ctx.accounts.to_account.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                    signer,
                )?;
            },
            // 执行 SPL Token 转账
            2 => {
                let cpi_accounts = TokenTransfer {
                    from: ctx.accounts.from_token_account.to_account_info(),
                    to: ctx.accounts.to_token_account.to_account_info(),
                    authority: ctx.accounts.pda_account.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

                token::transfer(cpi_ctx, amount)?;
            },
            _ => {
                return Err(ProgramError::InvalidArgument.into());
            }
        }

        Ok(())
    }

    // Interact with other applications via Proxy
    pub fn interact_with_application(ctx: Context<InteractWithApplication>, aa_account: Pubkey) -> Result<()> {
        // Define logic for interacting with other applications, e.g., DEX trades, etc.
        Ok(())
    }

    // 手动反序列化并输出 PDA 账户结构
    pub fn inspect_pda_account(ctx: Context<InspectPDAAccount>) -> Result<()> {
        let account_info = &ctx.accounts.pda_account.to_account_info();

        // 使用 Anchor 的 try_deserialize 方法进行反序列化
        let pda_account_data: PDAAccount = match PDAAccount::try_deserialize(&mut &account_info.data.borrow()[..]) {
            Ok(data) => data,  // 反序列化成功
            Err(_) => {
                msg!("Failed to deserialize account");
                return Err(ProgramError::InvalidAccountData.into());
            }
        };

        // 输出反序列化后的数据
        msg!("PDA Account bump: {}", pda_account_data.bump);
        msg!("PDA Account owner: {}", pda_account_data.owner);

        Ok(())
    }
}

// Context for initializing ProxyAccount
#[derive(Accounts)]
pub struct InitializeProxyAccount<'info> {
    #[account(init, payer = deployer, space = 8 + 32 * 10 + 32)] // Allocate space for 10 AA accounts and an owner
    pub proxy_account: Account<'info, ProxyAccount>,
    #[account(mut)]
    pub deployer: Signer<'info>,  // Deployer as the signer
    pub system_program: Program<'info, System>,
}

// Context for creating an AA account via PDA
#[derive(Accounts)]
pub struct CreatePDAAccount<'info> {
    #[account(mut)]
    pub proxy_account: Account<'info, ProxyAccount>,  // ProxyAccount
    #[account(init, seeds = [b"aa_account".as_ref(), proxy_account.key().as_ref()], bump, payer = deployer, space = 8 + 1 + 32)]  // 8 bytes for Solana, 1 byte for bump, 32 bytes for owner
    pub pda_account: Account<'info, PDAAccount>,  // PDA account
    #[account(mut)]
    pub deployer: Signer<'info>,  // Deployer as the signer
    pub system_program: Program<'info, System>,
}

// Context for controlling AA accounts via the deployer
#[derive(Accounts)]
pub struct ControlAAAccount<'info> {
    #[account(mut)]
    pub proxy_account: Account<'info, ProxyAccount>,  // ProxyAccount
    #[account(mut)]
    pub pda_account: Account<'info, PDAAccount>,  // PDA-created AA account
    #[account(mut)]
    pub to_account: AccountInfo<'info>,  // Destination account (receiving SOL)
    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,  // Source account for SPL Token
    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>,  // Destination account for SPL Token
    #[account(mut)]
    pub deployer: Signer<'info>,  // Deployer as the signer (owner)
    pub system_program: Program<'info, System>,  // Solana System Program
    pub token_program: Program<'info, Token>,  // SPL Token Program
}

// Context for interacting with an application
#[derive(Accounts)]
pub struct InteractWithApplication<'info> {
    #[account(mut)]
    pub pda_account: Account<'info, PDAAccount>,  // AA account
}

// Context for inspecting PDAAccount (新增)
#[derive(Accounts)]
pub struct InspectPDAAccount<'info> {
    #[account(mut)]
    pub pda_account: Account<'info, PDAAccount>,  // 要检查的PDA账户
}

// Proxy contract account structure
#[account]
pub struct ProxyAccount {
    pub aa_accounts: Vec<Pubkey>,  // Stores public keys of AA accounts
    pub owner: Pubkey,  // Owner's public key (deployer's public key)
}

// PDA AA account structure
#[account]
pub struct PDAAccount {
    pub bump: u8,  // Bump value for the PDA
    pub owner: Pubkey,  // Owner's public key (set to program ID)
}
