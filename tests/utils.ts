import DLMM, { binIdToBinArrayIndex, deriveBinArray } from "@meteora-ag/dlmm-sdk-public";
import { getAssociatedTokenAddressSync, createAssociatedTokenAccountIdempotentInstruction, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";


// Surfnet Helpers
export const surfnetAirdrop = async (connection: Connection, address: string, lamports: number) => {
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

export const surfnetTokenAirdrop = async (connection: Connection, owner: string, mint: string, amount: number) => {
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

const fetchHighestLiquidityPool = async (mintA: PublicKey, mintB: PublicKey): Promise<PublicKey> => {
  // Meteora API endpoint for all pairs
  const response = await fetch('https://dlmm-api.meteora.ag/pair/all');
  const pairs = await response.json();

  // Filter for pairs matching our two mints
  const matchingPairs = pairs.filter((p: any) => 
    (p.mint_x === mintA.toBase58() && p.mint_y === mintB.toBase58()) ||
    (p.mint_x === mintB.toBase58() && p.mint_y === mintA.toBase58())
  );

  if (matchingPairs.length === 0) {
    throw new Error("No Meteora pool found for this pair");
  }

  // Sort by liquidity (TVL) descending to get the best pool
  matchingPairs.sort((a: any, b: any) => parseFloat(b.liquidity) - parseFloat(a.liquidity));

  // Return the highest liquidity pool address
  return new PublicKey(matchingPairs[0].address);
};

/**
 * Fetches the active bin for a DLMM pool and derives its Bin Array PDA.
 * * @param dlmm - The initialized DLMM instance
 * @param programId - The Meteora DLMM Program ID
 * @returns Object containing the active bin data, array index, and the derived PDA pubkey
 */
export async function getActiveBinArrayHelper(dlmm: DLMM, programId: PublicKey) {
  const activeBin = await dlmm.getActiveBin();
  const activeBinIdBN = new BN(activeBin.binId);

  // 1. Convert the raw Bin ID to its parent Bin Array Index
  const activeBinArrayIndex = binIdToBinArrayIndex(activeBinIdBN);

  // 2. Derive the exact PDA of the Bin Array
  const [activeBinArrayPubkey] = deriveBinArray(
    dlmm.pubkey, 
    activeBinArrayIndex, 
    programId
  );

  console.log(`\n=== 📊 POOL LIQUIDITY ANALYSIS ===`);
  console.log(`Pool Address: ${dlmm.pubkey.toBase58()}`);
  console.log(`Current Active Bin ID: ${activeBin.binId} (Price: ${activeBin.price})`);
  console.log(`Active Bin Array Index: ${activeBinArrayIndex.toString()}`);
  console.log(`Active Bin Array Pubkey: ${activeBinArrayPubkey.toBase58()}`);
  console.log(`==================================\n`);

  return {
    activeBin,
    activeBinArrayIndex,
    activeBinArrayPubkey
  };
}