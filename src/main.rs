use std::str::FromStr;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, ResponseError, error::InternalError};

use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::{AccountMeta, Instruction, }, pubkey::Pubkey, signature::Keypair, signer::Signer};
use solana_system_interface::{instruction as system_instruction};
use spl_token::{id,instruction};
use base64::{Engine as _, engine::general_purpose};
use std::fmt;

#[derive(Serialize, Deserialize)]
struct SendSolInfo {
    from: String,
    to: String,
    lamports: u64
}


#[derive(Serialize, Deserialize)]
struct MintInfo {
    mintAuthority: String,
    mint: String,
    decimals: u8
}

#[derive(Serialize, Deserialize)]
struct MintTokenInfo {
    mint: String,
    destination: String,
    authority: String,
    amount: u64
}

#[derive(Serialize, Deserialize)]
struct SendTokenInfo {
    destination: String,
    mint: String,
    owner: String,
    amount: u64
}

#[derive(Serialize, Deserialize)]
struct KeypairInfo {
    pubkey: String,
    secret: String
}

#[derive(Serialize, Deserialize)]
struct KeypairResp {
    success: bool,
    data: KeypairInfo
}

#[derive(Serialize, Deserialize)]
pub struct InstructionData {
    pub program_id: String,
    pub accounts: Vec<AccountMeta>,
    pub instruction_data: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccountTokenData {
    pub pubkey: Pubkey,
    pub is_signer: bool,
}

#[derive(Serialize, Deserialize)]
pub struct InstructionTokenData {
    pub program_id: String,
    pub accounts: Vec<AccountTokenData>,
    pub instruction_data: String,
}

#[derive(Serialize, Deserialize)]
pub struct InstructionSolData {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub instruction_data: String,
}


#[derive(Serialize, Deserialize)]
struct MintResp {
    success: bool,
    data: InstructionData
}


#[derive(Serialize, Deserialize)]
struct SendTokenResp {
    success: bool,
    data: InstructionTokenData
}
#[derive(Serialize, Deserialize)]
struct SendSolResp {
    success: bool,
    data: InstructionSolData
}

#[derive(Serialize, Deserialize)]
struct ErrorResp {
    success: bool,
    error: String
}

#[derive(Debug)]
pub struct ApiError {
    message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(ErrorResp {
            success: false,
            error: self.message.clone(),
        })
    }
}

impl From<&str> for ApiError {
    fn from(msg: &str) -> Self {
        ApiError { message: msg.to_string() }
    }
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        ApiError { message }
    }
}

impl From<actix_web::error::JsonPayloadError> for ApiError {
    fn from(err: actix_web::error::JsonPayloadError) -> Self {
        ApiError {
            message: "Invalid JSON format or structure".to_string(),
        }
    }
}

impl From<actix_web::Error> for ApiError {
    fn from(err: actix_web::Error) -> Self {
        ApiError {
            message: "Request processing error".to_string(),
        }
    }
}

#[post("/keypair")]
async fn keypair() -> Result<HttpResponse, ApiError> {
    let kp = Keypair::new();
    let resp = KeypairResp {
        success: true,
        data: KeypairInfo {
            pubkey: kp.pubkey().to_string(),
            secret: bs58::encode(kp.to_bytes()).into_string()
        }
    };
    Ok(HttpResponse::Ok().json(resp))
}

#[post("/token/create")]
async fn create_token(info: web::Json<MintInfo>) -> Result<HttpResponse, ApiError> {
    let mint_authority = Pubkey::from_str(&info.mintAuthority)
        .map_err(|_| ApiError::from("Invalid mint authority"))?;
    
    let mint = Pubkey::from_str(&info.mint)
        .map_err(|_| ApiError::from("Invalid mint"))?;
    
    let instruction_data = instruction::initialize_mint(
        &id(),
        &mint,
        &mint_authority,
        Some(&mint_authority),
        info.decimals,
    ).map_err(|e| ApiError::from(format!("Failed to create mint instruction: {}", e)))?;

    let resp = MintResp {
        success: true,
        data: InstructionData {
            program_id: spl_token::ID.to_string(),
            accounts: instruction_data.accounts,
            instruction_data: general_purpose::STANDARD.encode(&instruction_data.data)
        }
    };
    
    Ok(HttpResponse::Ok().json(resp))
}

#[post("/token/mint")]
async fn mint_token(info: web::Json<MintTokenInfo>) -> Result<HttpResponse, ApiError> {
    let mint_authority = Pubkey::from_str(&info.authority)
        .map_err(|_| ApiError::from("Invalid mint authority"))?;
    
    let mint = Pubkey::from_str(&info.mint)
        .map_err(|_| ApiError::from("Invalid mint"))?;
    
    let destination = Pubkey::from_str(&info.destination)
        .map_err(|_| ApiError::from("Invalid destination"))?;

    let instruction_data = instruction::mint_to(
        &id(),
        &mint,
        &destination,
        &mint_authority,
        &[&mint_authority],
        info.amount,
    ).map_err(|e| ApiError::from(format!("Failed to create mint instruction: {}", e)))?;

    let resp = MintResp {
        success: true,
        data: InstructionData {
            program_id: spl_token::ID.to_string(),
            accounts: instruction_data.accounts,
            instruction_data: general_purpose::STANDARD.encode(&instruction_data.data)
        }
    };
    
    Ok(HttpResponse::Ok().json(resp))
}

#[post("/send/sol")]
async fn send_sol(info: web::Json<SendSolInfo>) -> Result<HttpResponse, ApiError> {
    let from = Pubkey::from_str(&info.from)
        .map_err(|_| ApiError::from("Invalid from"))?;
    
    let to = Pubkey::from_str(&info.to)
        .map_err(|_| ApiError::from("Invalid to"))?;

    let amount = info.lamports;
    
    let instruction_data = system_instruction::transfer(&from, &to, amount);
    
    let resp = SendSolResp {
        success: true,
        data: InstructionSolData {
            program_id: String::from("11111111111111111111111111111111"),
            accounts: instruction_data.accounts.iter().take(2).map(|a| a.pubkey.to_string()).collect(),
            instruction_data: general_purpose::STANDARD.encode(&instruction_data.data)
        }
    };
    
    Ok(HttpResponse::Ok().json(resp))
}


#[post("/send/token")]
async fn send_token(info: web::Json<SendTokenInfo>) -> Result<HttpResponse, ApiError> {
    let from = Pubkey::from_str(&info.owner)
        .map_err(|_| ApiError::from("Invalid from"))?;
    
    let to = Pubkey::from_str(&info.destination)
        .map_err(|_| ApiError::from("Invalid to"))?;

    let mint = Pubkey::from_str(&info.mint)
        .map_err(|_| ApiError::from("Invalid mint"))?;

    let amount = info.amount;
    
    let instruction_data = instruction::transfer_checked(
        &id(),
        &from, 
        &mint, 
        &to,
        &from,
        &[&from], 
        amount,
        6,
    ).map_err(|e| ApiError::from(format!("Failed to create token transfer instruction: {}", e)))?;

    let accounts = instruction_data.accounts.iter().map(|a| AccountTokenData {
        pubkey: a.pubkey,
        is_signer: a.is_signer,
    }).collect();

    let resp = SendTokenResp {
        success: true,
        data: InstructionTokenData {
            program_id: spl_token::ID.to_string(),
            accounts: accounts,
            instruction_data: general_purpose::STANDARD.encode(&instruction_data.data)
        }
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _req| {
                        let api_error = ApiError::from("Invalid JSON format or structure");
                        actix_web::error::InternalError::from_response(err, api_error.error_response()).into()
                    })
            )
            .service(keypair)
            .service(create_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}   