use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer as TokenTransfer, TokenAccount, Token};
use anchor_lang::solana_program::{system_instruction, program::invoke};

declare_id!("2B5axdbUe68mDy5FWq1hLANcwHtQ1fbCJj8HkaMLCU2t");

#[program]
mod solana_proxy {
    use super::*;

    // 初始化 ProxyAccount
    pub fn initialize_proxy_account(ctx: Context<InitializeProxyAccount>) -> Result<()> {
        let proxy_account = &mut ctx.accounts.proxy_account;
        proxy_account.aa_accounts = Vec::new();  // 初始化为一个空的AA账户列表
        Ok(())
    }

    // 创建一个AA账户
    pub fn create_aa_account(ctx: Context<CreateAAAccount>, aa_account: Pubkey) -> Result<()> {
        let proxy_account = &mut ctx.accounts.proxy_account;
        proxy_account.aa_accounts.push(aa_account);
        Ok(())
    }

    // 通过Proxy合约控制AA账户，执行SOL或SPL Token转账
    pub fn control_aa_account(ctx: Context<ControlAAAccount>, amount: u64, action: u8) -> Result<()> {
        match action {
            // 执行 SOL 转账
            1 => {
                // 创建转账指令，从AA账户转移SOL到目标账户
                let transfer_instruction = system_instruction::transfer(
                    &ctx.accounts.aa_account.key,         // AA账户
                    &ctx.accounts.to_account.key,         // 目标账户
                    amount,                               // 转移的SOL数量，单位为lamports
                );

                // 调用系统程序执行 SOL 转账
                invoke(
                    &transfer_instruction,
                    &[
                        ctx.accounts.aa_account.to_account_info(),
                        ctx.accounts.to_account.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                )?;
            },
            // 执行 SPL Token 转账
            2 => {
                let cpi_accounts = TokenTransfer {
                    from: ctx.accounts.from_token_account.to_account_info(),
                    to: ctx.accounts.to_token_account.to_account_info(),
                    authority: ctx.accounts.aa_account.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

                // 执行 SPL Token 转账
                token::transfer(cpi_ctx, amount)?;
            },
            _ => {
                return Err(ProgramError::InvalidArgument.into());
            }
        }
        Ok(())
    }

    // 通过Proxy与应用交互
    pub fn interact_with_application(ctx: Context<InteractWithApplication>, aa_account: Pubkey) -> Result<()> {
        // 可执行与其他应用的交互逻辑，例如与DEX进行交易等
        Ok(())
    }
}

// 初始化 ProxyAccount 的上下文
#[derive(Accounts)]
pub struct InitializeProxyAccount<'info> {
    #[account(init, payer = user, space = 8 + 32 * 10)] // 分配足够的空间用于存储 10 个 AA 账户
    pub proxy_account: Account<'info, ProxyAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// 定义创建AA账户的上下文
#[derive(Accounts)]
pub struct CreateAAAccount<'info> {
    #[account(mut)]
    pub proxy_account: Account<'info, ProxyAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
}

// 定义通过Proxy控制AA账户的上下文
#[derive(Accounts)]
pub struct ControlAAAccount<'info> {
    #[account(mut)]
    pub aa_account: Signer<'info>,               // AA账户的签名者
    #[account(mut)]
    pub to_account: AccountInfo<'info>,          // 目标账户（接收SOL）
    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>, // SPL Token来源账户
    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>,   // SPL Token目标账户
    pub system_program: Program<'info, System>,           // Solana系统程序
    pub token_program: Program<'info, Token>,             // SPL Token程序
}

// 定义与应用交互的上下文
#[derive(Accounts)]
pub struct InteractWithApplication<'info> {
    #[account(mut)]
    pub aa_account: AccountInfo<'info>,
}

// Proxy合约账户结构
#[account]
pub struct ProxyAccount {
    pub aa_accounts: Vec<Pubkey>,  // 存储AA账户的公钥列表
}
