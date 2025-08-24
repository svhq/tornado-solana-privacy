import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TornadoSolana } from "../target/types/tornado_solana";
import { assert } from "chai";
import * as crypto from "crypto";

describe("tornado_solana", () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TornadoSolana as Program<TornadoSolana>;
  
  // Test accounts
  let tornadoState: anchor.web3.PublicKey;
  let depositor: anchor.web3.Keypair;
  let recipient: anchor.web3.Keypair;
  
  // Test data
  const denomination = new anchor.BN(1_000_000_000); // 1 SOL
  let commitment: Buffer;
  let nullifier: Buffer;
  let nullifierHash: Buffer;
  let secret: Buffer;

  before(async () => {
    // Generate test accounts
    depositor = anchor.web3.Keypair.generate();
    recipient = anchor.web3.Keypair.generate();
    
    // Fund depositor account
    const airdropSignature = await provider.connection.requestAirdrop(
      depositor.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);
    
    // Derive PDA for tornado state
    [tornadoState] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("tornado")],
      program.programId
    );
    
    // Generate commitment data (in production, this would be done client-side)
    nullifier = crypto.randomBytes(31);
    secret = crypto.randomBytes(31);
    
    // Compute commitment = Hash(nullifier || secret)
    const commitmentData = Buffer.concat([nullifier, secret]);
    commitment = crypto.createHash('sha256').update(commitmentData).digest();
    
    // Compute nullifier hash
    nullifierHash = crypto.createHash('sha256').update(nullifier).digest();
  });

  it("Initializes the tornado pool", async () => {
    try {
      const tx = await program.methods
        .initialize(denomination)
        .accounts({
          tornadoState,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      
      console.log("Initialize transaction:", tx);
      
      // Fetch and verify the initialized state
      const state = await program.account.tornadoState.fetch(tornadoState);
      assert.equal(state.denomination.toString(), denomination.toString());
      assert.equal(state.authority.toString(), provider.wallet.publicKey.toString());
      assert.equal(state.nextIndex, 0);
    } catch (error) {
      console.error("Initialize error:", error);
      throw error;
    }
  });

  it("Makes a deposit", async () => {
    try {
      // Convert commitment to array format expected by the program
      const commitmentArray = Array.from(commitment);
      
      const tx = await program.methods
        .deposit(commitmentArray as any)
        .accounts({
          tornadoState,
          depositor: depositor.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([depositor])
        .rpc();
      
      console.log("Deposit transaction:", tx);
      
      // Verify the deposit was recorded
      const state = await program.account.tornadoState.fetch(tornadoState);
      assert.equal(state.nextIndex, 1);
      
      // Check that commitment was stored
      const storedCommitment = state.commitments[0];
      assert.deepEqual(storedCommitment, commitmentArray);
      
      // Verify SOL was transferred to the tornado state account
      const tornadoBalance = await provider.connection.getBalance(tornadoState);
      assert.isAbove(tornadoBalance, denomination.toNumber());
    } catch (error) {
      console.error("Deposit error:", error);
      throw error;
    }
  });

  it("Makes a withdrawal", async () => {
    try {
      // Get current state for merkle root
      const state = await program.account.tornadoState.fetch(tornadoState);
      const currentRoot = state.roots[state.currentRootIndex];
      
      // Create mock proof (in production, this would be a real ZK proof)
      const mockProof = Buffer.alloc(256);
      
      // Convert to array formats
      const proofArray = Array.from(mockProof);
      const rootArray = Array.from(currentRoot);
      const nullifierHashArray = Array.from(nullifierHash);
      
      const fee = new anchor.BN(10_000_000); // 0.01 SOL fee
      const refund = new anchor.BN(0);
      
      const tx = await program.methods
        .withdraw(
          proofArray,
          rootArray as any,
          nullifierHashArray as any,
          recipient.publicKey,
          null, // No relayer
          fee,
          refund
        )
        .accounts({
          tornadoState,
          recipient: recipient.publicKey,
        })
        .rpc();
      
      console.log("Withdrawal transaction:", tx);
      
      // Verify the withdrawal
      const stateAfter = await program.account.tornadoState.fetch(tornadoState);
      
      // Check nullifier was marked as spent
      const spentNullifier = stateAfter.nullifierHashes[0];
      assert.deepEqual(spentNullifier, nullifierHashArray);
      
      // Verify recipient received funds
      const recipientBalance = await provider.connection.getBalance(recipient.publicKey);
      const expectedAmount = denomination.toNumber() - fee.toNumber();
      assert.approximately(recipientBalance, expectedAmount, 100_000); // Allow small variance for rent
    } catch (error) {
      console.error("Withdrawal error:", error);
      throw error;
    }
  });

  it("Prevents double spending", async () => {
    try {
      // Try to withdraw with the same nullifier again
      const state = await program.account.tornadoState.fetch(tornadoState);
      const currentRoot = state.roots[state.currentRootIndex];
      
      const mockProof = Buffer.alloc(256);
      const proofArray = Array.from(mockProof);
      const rootArray = Array.from(currentRoot);
      const nullifierHashArray = Array.from(nullifierHash);
      
      const fee = new anchor.BN(10_000_000);
      const refund = new anchor.BN(0);
      
      try {
        await program.methods
          .withdraw(
            proofArray,
            rootArray as any,
            nullifierHashArray as any,
            recipient.publicKey,
            null,
            fee,
            refund
          )
          .accounts({
            tornadoState,
            recipient: recipient.publicKey,
          })
          .rpc();
        
        assert.fail("Should have failed with already spent error");
      } catch (error) {
        assert.include(error.toString(), "NoteAlreadySpent");
      }
    } catch (error) {
      console.error("Double spend test error:", error);
      throw error;
    }
  });

  it("Prevents duplicate deposits", async () => {
    try {
      // Try to deposit the same commitment again
      const commitmentArray = Array.from(commitment);
      
      try {
        await program.methods
          .deposit(commitmentArray as any)
          .accounts({
            tornadoState,
            depositor: depositor.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([depositor])
          .rpc();
        
        assert.fail("Should have failed with duplicate commitment error");
      } catch (error) {
        assert.include(error.toString(), "DuplicateCommitment");
      }
    } catch (error) {
      console.error("Duplicate deposit test error:", error);
      throw error;
    }
  });
});