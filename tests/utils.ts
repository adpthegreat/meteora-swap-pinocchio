import { getAssociatedTokenAddressSync, createAssociatedTokenAccountIdempotentInstruction, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { Connection } from "@solana/web3.js";


// Surfnet Helpers
const surfnetAirdrop = async (connection: Connection, address: string, lamports: number) => {
  const call = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "surfnet_setAccount",
    "params": [address, { "lamports": lamports }]
  };
  
  await fetch(connection.rpcEndpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(call)
  });
};

const surfnetTokenAirdrop = async (connection: Connection, owner: string, mint: string, amount: number) => {
  const call = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "surfnet_setTokenAccount",
    "params": [owner, mint, {"amount": amount, "state": "initialized"}]
  };
  
  await fetch(connection.rpcEndpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(call)
  });
};