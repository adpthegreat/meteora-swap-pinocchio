import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import DLMM, { binIdToBinArrayIndex, deriveBinArray } from '@meteora-ag/dlmm-sdk-public';
import BN from 'bn.js';
import { assert } from 'chai';
import { getSwapInstruction } from '../clients/js/src/generated/instructions/swap.ts';
import { getSwap2Instruction } from '../clients/js/src/generated/instructions/swap2.ts';
import { getActiveBinArrayHelper, surfnetAirdrop, surfnetTokenAirdrop } from './utils.ts';
import { address } from '@solana/kit';

// Connect to local Surfpool instance (mirrors mainnet)
const CONNECTION = new Connection('http://127.0.0.1:8899', 'confirmed');
const CONNECTION1 = new Connection('https://api.mainnet.solana.com','confirmed' )
// Mainnet Mints
const WSOL_MINT = new PublicKey('So11111111111111111111111111111111111111112');
const USDC_MINT = new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v');
const USDT_MINT = new PublicKey('Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB');

// Mainnet Meteora DLMM Pools (Replace with exact pool addresses if differ
// search for liq pools on https://www.meteora.ag/?tab=new 
// Assuming we are using known high-liquidity pools for these pairs
const WSOL_USDC_POOL = new PublicKey('BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y');  //lb pair
const USDC_USDT_POOL = new PublicKey('ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq'); //lb pair

const METEORA_DLMM_PROGRAM_ID = new PublicKey('LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo'); // Mainnet DLMM Program

const MEMO_PROGRAM_ID = new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr');

 const [eventAuthorityPda] = PublicKey.findProgramAddressSync(
   [Buffer.from('__event_authority')],
   METEORA_DLMM_PROGRAM_ID
 );

 console.log(`this is the event authority ${eventAuthorityPda}`)
describe('Meteora Two-Hop Swaps', () => {
  let user: Keypair;
  let dlmmSolUsdc: DLMM;
  let dlmmUsdcUsdt: DLMM;

  let userWsolAta: PublicKey;
  let userUsdcAta: PublicKey;
  let userUsdtAta: PublicKey;

  before(async () => {
    user = Keypair.generate();
    userWsolAta = getAssociatedTokenAddressSync(WSOL_MINT, user.publicKey);
    userUsdcAta = getAssociatedTokenAddressSync(USDC_MINT, user.publicKey);
    userUsdtAta = getAssociatedTokenAddressSync(USDT_MINT, user.publicKey);

    // 1. Airdrop SOL for gas
    await surfnetAirdrop(CONNECTION, user.publicKey.toBase58(), 10 * 10 ** 9);

    // 2. Airdrop Initial wSOL for the swap input using Surfpool utility
    // Giving the user 10 wSOL
    await surfnetTokenAirdrop(CONNECTION, user.publicKey.toBase58(), WSOL_MINT.toBase58(), 10 * 10 ** 9);
    
    // Initialize required ATAs with 0 balance so they exist
    await surfnetTokenAirdrop(CONNECTION, user.publicKey.toBase58(), USDC_MINT.toBase58(), 0);
    await surfnetTokenAirdrop(CONNECTION, user.publicKey.toBase58(), USDT_MINT.toBase58(), 0);

    // 3. Initialize DLMM SDK instances
    dlmmSolUsdc = await DLMM.create(CONNECTION1, WSOL_USDC_POOL);
    dlmmUsdcUsdt = await DLMM.create(CONNECTION1, USDC_USDT_POOL);
  });

  it('Test 1: Two Top-Level Swap Instructions (SOL -> USDC -> USDT)', async () => {
    const amountIn = new BN(1 * 10 ** 9); // Swap 1 wSOL

    let activeBinArray1 = await getActiveBinArrayHelper(dlmmSolUsdc, METEORA_DLMM_PROGRAM_ID);
    let activeBinArray2 = await getActiveBinArrayHelper(dlmmUsdcUsdt, METEORA_DLMM_PROGRAM_ID);

    // --- QUOTE & PREPARE SWAP 1 (SOL -> USDC) ---
    const binArraysSolUsdc = await dlmmSolUsdc.getBinArrays(); // true = swap X to Y (assuming SOL is X)
    
    const quoteSolUsdc = dlmmSolUsdc.swapQuote(amountIn, true, new BN(10), binArraysSolUsdc);
    const solUsdcBinArraysUsed = quoteSolUsdc.binArraysPubkey;

    for (const binArray of binArraysSolUsdc) {
        let totalX = new BN(0);
        let totalY = new BN(0);
        
        // Sum up the reserves of all 70 bins inside this specific Bin Array
        for (const bin of binArray.account.bins) {
          totalX = totalX.add(bin.amountX);
          totalY = totalY.add(bin.amountY);
        }

        // Only print arrays that actually contain liquidity to keep your console clean
        if (totalX.gtn(0) || totalY.gtn(0)) {
          console.log(`\nBin Array Pubkey: ${binArray.publicKey.toBase58()}`);
          
          // Print raw lamport amounts
          console.log(`Total Token X (SOL) : ${totalX.toString()}`);
          console.log(`Total Token Y (USDC): ${totalY.toString()}`);
          
          // Optional: Convert to human-readable UI amounts (assuming 9 decimals for SOL, 6 for USDC)
          // console.log(`UI Token X: ${totalX.toNumber() / 1e9}`);
          // console.log(`UI Token Y: ${totalY.toNumber() / 1e6}`);
        }
      }
    
    console.log(`==================================\n`);
    
    const binArraysUsdcUsdt = await dlmmUsdcUsdt.getBinArrays(); // true = swap X to Y (assuming USDC is X)
    // --- QUOTE & PREPARE SWAP 2 (USDC -> USDT) ---
    const quoteUsdcUsdt = dlmmUsdcUsdt.swapQuote(quoteSolUsdc.minOutAmount, true, new BN(100), binArraysUsdcUsdt);
    const usdcUsdtBinArraysUsed = quoteUsdcUsdt.binArraysPubkey;
    // Helper to map Codama instruction to Web3.js Instruction and append bin arrays
    const buildProgramSwapIx = (
      dlmm: DLMM, 
      inTokenAta: PublicKey, 
      outTokenAta: PublicKey, 
      amountIn: BN, 
      minOut: BN, 
      binArrays: any[]
    ) => {
      const state = dlmm.lbPair;
      // Get Codama Ix
      const codamaIx = getSwapInstruction({
        user: { address: address(user.publicKey.toBase58()), signTransactions: async (txs) => txs }, // 3 = Signer
        lbPair: address(dlmm.pubkey.toBase58()),
        binrrayBitmapExt: dlmm.binArrayBitmapExtension ? address(dlmm.binArrayBitmapExtension.publicKey.toBase58()) : address(METEORA_DLMM_PROGRAM_ID.toBase58()),
        reserveX: address(state.reserveX.toBase58()),
        reserveY: address(state.reserveY.toBase58()),
        userTokenIn: address(inTokenAta.toBase58()),
        userTokenOut: address(outTokenAta.toBase58()),
        tokenXMint: address(state.tokenXMint.toBase58()),
        tokenYMint: address(state.tokenYMint.toBase58()),
        oracle: address(state.oracle.toBase58()),
        hostFeeIn: address(METEORA_DLMM_PROGRAM_ID.toBase58()), // Host fee usually points to program ID if unused
        tokenXProgram: address(TOKEN_PROGRAM_ID.toBase58()),
        tokenYProgram: address(TOKEN_PROGRAM_ID.toBase58()),
        memoProgram: address(MEMO_PROGRAM_ID.toBase58()),
        eventAuthority: address(eventAuthorityPda.toBase58()), 
        dlmmProgram: address(METEORA_DLMM_PROGRAM_ID.toBase58()),
        amountIn: BigInt(amountIn.toString()),
        minOut: BigInt(minOut.toString()),
        numBinrrays: 1 //we are only passing the active bin array 
      });

      // console.log(`bin arrays len: ${binArrays.length}`)
      // NEW: LOGGING - See exactly what Codama generated before it hits Web3.js
      console.log(`\n=== Mapping Codama Accounts for Swap ===`);
      codamaIx.accounts.forEach((acc: any, i: number) => {
        console.log(`[Idx ${i}] Role ${acc.role} | Address: ${acc.address}`);
      });
      console.log(`========================================\n`);
      
      // Convert to web3.js v1 instruction
      const ix = new TransactionInstruction({
        programId: new PublicKey(codamaIx.programAddress),
        keys: codamaIx.accounts.map((acc: any) => ({
          pubkey: new PublicKey(acc.address),
          isSigner: acc.role === 2 || acc.role === 2,
          isWritable: acc.role === 1 || acc.role === 3,
        })),
        data: Buffer.from(codamaIx.data),
      });


      for (let i = 0; i < binArrays.length; i++) {
        const bin = binArrays[i];
        
        let binPubkey = bin;

        if (!binPubkey) {
          throw new Error(`Invalid Bin Array format at index ${i}: ${JSON.stringify(bin)}`);
        }

        // ix.keys.push({ pubkey: binPubkey, isSigner: false, isWritable: true }); //getting invalidBinArray so i just use the active bin
      }

      if (state.tokenXMint.toString() == WSOL_MINT.toString()) { //SOL/USDC
        ix.keys.push({ pubkey: activeBinArray1.activeBinArrayPubkey, isSigner: false, isWritable: true }); //push active bin instead of whatever the swapQuote returns 
      }

      if (state.tokenYMint.toString() == USDT_MINT.toString()) { //USDC/USDT
        ix.keys.push({ pubkey: activeBinArray2.activeBinArrayPubkey, isSigner: false, isWritable: true }); //push active bin instead of whatever the swapQuote returns 
      }

      return ix;
    };

    const ix1 = buildProgramSwapIx(dlmmSolUsdc, userWsolAta, userUsdcAta, amountIn, quoteSolUsdc.minOutAmount, usdcUsdtBinArraysUsed);
    const ix2 = buildProgramSwapIx(dlmmUsdcUsdt, userUsdcAta, userUsdtAta, quoteSolUsdc.minOutAmount, quoteUsdcUsdt.minOutAmount, solUsdcBinArraysUsed);

    const tx = new Transaction().add(ix1, ix2);
    tx.recentBlockhash = (await CONNECTION.getLatestBlockhash()).blockhash;
    tx.feePayer = user.publicKey;

    // Fetch Before Balances
    const preUsdt = await CONNECTION.getTokenAccountBalance(userUsdtAta);
    
    // NEW: Pre-Swap Logging
    console.log(`\n[Test 1] Executing Two-Hop Transaction...`);
    console.log(`Pre-Swap USDT Balance: ${preUsdt.value.uiAmount}`);

    // Execute
    const sig = await sendAndConfirmTransaction(CONNECTION, tx, [user], { skipPreflight: true });
    console.log(`Test 1 Tx Sig: ${sig}`);

    // Fetch After Balances
    const postUsdt = await CONNECTION.getTokenAccountBalance(userUsdtAta);
    const postUsdc = await CONNECTION.getTokenAccountBalance(userUsdcAta);

    // NEW: Post-Swap Logging
    console.log(`Post-Swap USDT Balance: ${postUsdt.value.uiAmount}`);
    console.log(`Leftover USDC Dust: ${postUsdc.value.uiAmount}`);
    console.log(`Total USDT Earned: ${Number(postUsdt.value.amount) - Number(preUsdt.value.amount)} lamports`);

    // Assertions
    assert.isTrue(Number(postUsdt.value.amount) > Number(preUsdt.value.amount), "USDT balance should increase");
  });


  it('Test 2: Instruction Introspection Pattern', async () => {
    let activeBinArray1 = await getActiveBinArrayHelper(dlmmSolUsdc, METEORA_DLMM_PROGRAM_ID);
    let activeBinArray2 = await getActiveBinArrayHelper(dlmmUsdcUsdt, METEORA_DLMM_PROGRAM_ID);
    // We are resetting/adding another 1 wSOL for this test via airdrop if needed, but we started with 10.
    const amountIn = new BN(1 * 10 ** 9); 


    const binArraysSolUsdc = await dlmmSolUsdc.getBinArrays(); // true = swap X to Y (assuming SOL is X)
    const quoteSolUsdc = dlmmSolUsdc.swapQuote(amountIn, true, new BN(10), binArraysSolUsdc);
    
    const binArraysUsdcUsdt = await dlmmUsdcUsdt.getBinArrays(); // true = swap X to Y (assuming USDC is X)
    const quoteUsdcUsdt = dlmmUsdcUsdt.swapQuote(quoteSolUsdc.minOutAmount, true, new BN(100), binArraysUsdcUsdt);
    const usdcUsdtBinArraysUsed = quoteUsdcUsdt.binArraysPubkey;

    // 1. Build Instruction 0 (Direct DLMM SDK call)
    // The SDK swap() method returns a transaction. We extract the first instruction.
    const dlmmSwapTx = await dlmmSolUsdc.swap({
      inToken: WSOL_MINT,
      outToken: USDC_MINT,
      inAmount: amountIn,
      minOutAmount: quoteSolUsdc.minOutAmount,
      lbPair: WSOL_USDC_POOL,
      user: user.publicKey,
      binArraysPubkey: [activeBinArray1.activeBinArrayPubkey] //binArraysSolUsdc.map(b => b.publicKey),
    });
    
    // Extract the actual Meteora Swap instruction (filter out compute budget/token ATA setups if any exist in the builder)
    const dlmmDirectIx = dlmmSwapTx.instructions.find(ix => ix.programId.equals(METEORA_DLMM_PROGRAM_ID))!;

    // 2. Build Instruction 1 (Our program `swap2`)
    const state = dlmmUsdcUsdt.lbPair;
    const codamaIx2 = getSwap2Instruction({
      user: { address: address(user.publicKey.toBase58()), signTransactions: async (txs) => txs },
      instructionsSysvar: address(SYSVAR_INSTRUCTIONS_PUBKEY.toBase58()),
      lbPair: address(dlmmUsdcUsdt.pubkey.toBase58()),
      binArrayBitmapExt: dlmmUsdcUsdt.binArrayBitmapExtension ? address(dlmmUsdcUsdt.binArrayBitmapExtension.publicKey.toBase58()) : address(METEORA_DLMM_PROGRAM_ID.toBase58()),
      reserveX: address(state.reserveX.toBase58()),
      reserveY: address(state.reserveY.toBase58()),
      userTokenIn: address(userUsdcAta.toBase58()),
      userTokenOut: address(userUsdtAta.toBase58()),
      tokenXMint: address(state.tokenXMint.toBase58()),
      tokenYMint: address(state.tokenYMint.toBase58()),
      oracle: address(state.oracle.toBase58()),
      hostFeeIn: address(METEORA_DLMM_PROGRAM_ID.toBase58()),
      tokenXProgram: address(TOKEN_PROGRAM_ID.toBase58()),
      tokenYProgram: address(TOKEN_PROGRAM_ID.toBase58()),
      memoProgram: address(MEMO_PROGRAM_ID.toBase58()),
      eventAuthority: address(eventAuthorityPda.toBase58()),
      dlmmProgram: address(METEORA_DLMM_PROGRAM_ID.toBase58()),
      amountIn:  BigInt(quoteSolUsdc.minOutAmount.toString()), 
      minOut: BigInt(quoteUsdcUsdt.minOutAmount.toString()), 
      numBinrrays: 1  //we are only passing the active bin array 
    });

    console.log(`this is the minOutAmount solUsdc${quoteSolUsdc.minOutAmount.toString()}`)
    console.log(`this is the minOutAmount usdcUsdt ${quoteUsdcUsdt.minOutAmount.toString()}`)
    console.log(`this is the bin array length ${usdcUsdtBinArraysUsed.length}`)

    const swap2Ix = new TransactionInstruction({
      programId: new PublicKey(codamaIx2.programAddress),
      keys: codamaIx2.accounts.map((acc: any) => ({
        pubkey: new PublicKey(acc.address),
        isSigner: acc.role === 2 || acc.role === 2,
        isWritable: acc.role === 1 || acc.role === 3,
      })),
      data: Buffer.from(codamaIx2.data),
    });

    let binArrays =  usdcUsdtBinArraysUsed;

     if (state.tokenYMint.toString() == USDT_MINT.toString()) { //USDC/USDT
        swap2Ix.keys.push({ pubkey: activeBinArray2.activeBinArrayPubkey, isSigner: false, isWritable: true }); //push active bin instead of whatever the swapQuote returns 
      }

      for (let i = 0; i < binArrays.length; i++) {
        const bin = binArrays[i];
        
        let binPubkey = bin;

        if (!binPubkey) {
          throw new Error(`Invalid Bin Array format at index ${i}: ${JSON.stringify(bin)}`);
        }

        // swap2Ix.keys.push({ pubkey: binPubkey, isSigner: false, isWritable: true }); //getting invalidBinArray tried again at 7.49pm 
      }

     

    // Pack into a single transaction (Instruction 0 = DLMM , Instruction 1 = swap2)
    const tx = new Transaction().add(dlmmDirectIx, swap2Ix);
    tx.recentBlockhash = (await CONNECTION.getLatestBlockhash()).blockhash;
    tx.feePayer = user.publicKey;

    const preUsdt = await CONNECTION.getTokenAccountBalance(userUsdtAta);
    const sig = await sendAndConfirmTransaction(CONNECTION, tx, [user], { skipPreflight: true });
    console.log(`Test 2 Tx Sig: ${sig}`);

    const postUsdt = await CONNECTION.getTokenAccountBalance(userUsdtAta);
    console.log(`USDT Earned : ${Number(postUsdt.value.amount) - Number(preUsdt.value.amount)}`);

    // Assertions
    assert.isTrue(Number(postUsdt.value.amount) > Number(preUsdt.value.amount), "USDT balance should increase via introspection logic");
  });
});

