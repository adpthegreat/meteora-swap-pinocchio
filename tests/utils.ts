import DLMM, { BinArrayAccount, binIdToBinArrayIndex, deriveBinArray } from "@meteora-ag/dlmm";
import { getAssociatedTokenAddressSync, createAssociatedTokenAccountIdempotentInstruction, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";


// Surfnet Helpers
const surfnetCall = async (connection: Connection, method: string, params: unknown[]) => {
  const response = await fetch(connection.rpcEndpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params })
  });
  const body = await response.json();
  if (body.error) {
    throw new Error(`${method} failed: ${JSON.stringify(body.error)}`);
  }
  return body.result;
};

export const surfnetAirdrop = (connection: Connection, address: string, lamports: number) =>
  surfnetCall(connection, 'surfnet_setAccount', [address, { lamports }]);

export const surfnetTokenAirdrop = (connection: Connection, owner: string, mint: string, amount: number) =>
  surfnetCall(connection, 'surfnet_setTokenAccount', [owner, mint, { amount, state: 'initialized' }]);

/**
 * Retries transient upstream failures: Surfpool lazily clones mainnet accounts,
 * and the mainnet RPC behind it intermittently answers "Internal error".
 */
export async function withRetry<T>(label: string, fn: () => Promise<T>, attempts = 4): Promise<T> {
  for (let attempt = 1; ; attempt++) {
    try {
      return await fn();
    } catch (err) {
      if (attempt >= attempts || !/Internal error|fetch failed|ECONNRESET|429/i.test(String(err))) {
        throw err;
      }
      console.warn(`${label}: transient RPC error (attempt ${attempt}/${attempts}), retrying`);
      await new Promise((resolve) => setTimeout(resolve, 1000 * attempt));
    }
  }
}

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
 * Reads the pool's current active bin and fetches the bin array that holds it.
 * Fetches just these two accounts, unlike `dlmm.getBinArrays()`, which scans every
 * bin array of the pool with getProgramAccounts.
 * @param dlmm - The initialized DLMM instance
 * @param programId - The Meteora DLMM Program ID
 * @returns The active bin id, its bin array index, and the bin array's pubkey and account
 */
export async function getActiveBinArrayHelper(dlmm: DLMM, programId: PublicKey) {
  const { activeId } = await dlmm.program.account.lbPair.fetch(dlmm.pubkey);

  // 1. Convert the raw Bin ID to its parent Bin Array Index
  const activeBinArrayIndex = binIdToBinArrayIndex(new BN(activeId));

  // 2. Derive the exact PDA of the Bin Array
  const [activeBinArrayPubkey] = deriveBinArray(
    dlmm.pubkey,
    activeBinArrayIndex,
    programId
  );

  // 3. Fetch it in the shape swapQuote expects
  const activeBinArray: BinArrayAccount = {
    publicKey: activeBinArrayPubkey,
    account: await dlmm.program.account.binArray.fetch(activeBinArrayPubkey),
  };

  console.log(`Pool Address: ${dlmm.pubkey.toBase58()}`);
  console.log(`Current Active Bin ID: ${activeId}`);
  console.log(`Active Bin Array Index: ${activeBinArrayIndex.toString()}`);
  console.log(`Active Bin Array Pubkey: ${activeBinArrayPubkey.toBase58()}`);
  console.log(`==================================\n`);

  return {
    activeId,
    activeBinArrayIndex,
    activeBinArrayPubkey,
    activeBinArray
  };
}
