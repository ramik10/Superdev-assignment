use std::str::FromStr;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder,Result};

use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::Keypair, signer::Signer};
use spl_token::{id,instruction};
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize)]
struct MintInfo {
    mintAuthority: String,
    mint: String,
    decimals: u8
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
struct MintResp {
    success: bool,
    data: InstructionData
}

#[derive(Serialize, Deserialize)]
struct ErrorResp {
    success: bool,
    error: String
}

#[post("/keypair")]
async fn keypair() -> Result<HttpResponse> {
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
async fn create_token(info: web::Json<MintInfo>) -> Result<HttpResponse> {
    let mint_authority = match Pubkey::from_str(&info.mintAuthority) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            let error_resp = ErrorResp {
                success: false,
                error: "Invalid mint authority".to_string()
            };
            return Ok(HttpResponse::BadRequest().json(error_resp));
        }
    };
    
    let mint = match Pubkey::from_str(&info.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            let error_resp = ErrorResp {
                success: false,
                error: "Invalid mint".to_string()
            };
            return Ok(HttpResponse::BadRequest().json(error_resp));
        }
    };
    
    let decimals = info.decimals;
    let instruction_data = match instruction::initialize_mint(
        &id(),
        &mint,
        &mint_authority,
        Some(&mint_authority),
        decimals,
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            let error_resp = ErrorResp {
                success: false,
                error: format!("Failed to create mint instruction: {}", e)
            };
            return Ok(HttpResponse::InternalServerError().json(error_resp));
        }
    };

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


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(keypair)
            .service(create_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}   