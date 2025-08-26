// Parse verification key to Rust format
// Based on hush project's implementation
const ffjavascript = require("ffjavascript");
const { unstringifyBigInts, leInt2Buff } = ffjavascript.utils;
const fs = require("fs");
const path = require("path");

async function main() {
  const inputPath = path.join(__dirname, "../build/verification_key.json");
  const outputPath = path.join(__dirname, "../../programs/tornado_solana/src/");
  
  console.log("Reading verification key from:", inputPath);
  
  const vkData = JSON.parse(fs.readFileSync(inputPath, 'utf8'));
  console.log("Number of public inputs:", vkData.nPublic);
  console.log("Number of IC points:", vkData.IC.length);
  
  // Process the verification key data
  const processedData = {
    nPublic: vkData.nPublic,
    vk_alpha_1: [],
    vk_beta_2: [],
    vk_gamma_2: [],
    vk_delta_2: [],
    IC: []
  };
  
  // Process alpha_g1 (G1 point - 64 bytes)
  // Skip the third coordinate (always 1)
  for (let j = 0; j < 2; j++) {
    const bytes = leInt2Buff(unstringifyBigInts(vkData.vk_alpha_1[j]), 32).reverse();
    processedData.vk_alpha_1.push(...bytes);
  }
  
  // Process beta_g2 (G2 point - 128 bytes)
  // Skip the third pair (always [1, 0])
  for (let j = 0; j < 2; j++) {
    const bytes1 = leInt2Buff(unstringifyBigInts(vkData.vk_beta_2[j][0]), 32);
    const bytes2 = leInt2Buff(unstringifyBigInts(vkData.vk_beta_2[j][1]), 32);
    const combined = [...bytes1, ...bytes2].reverse();
    processedData.vk_beta_2.push(...combined.slice(0, 32));
    processedData.vk_beta_2.push(...combined.slice(32, 64));
  }
  
  // Process gamma_g2 (G2 point - 128 bytes)
  // Skip the third pair (always [1, 0])
  for (let j = 0; j < 2; j++) {
    const bytes1 = leInt2Buff(unstringifyBigInts(vkData.vk_gamma_2[j][0]), 32);
    const bytes2 = leInt2Buff(unstringifyBigInts(vkData.vk_gamma_2[j][1]), 32);
    const combined = [...bytes1, ...bytes2].reverse();
    processedData.vk_gamma_2.push(...combined.slice(0, 32));
    processedData.vk_gamma_2.push(...combined.slice(32, 64));
  }
  
  // Process delta_g2 (G2 point - 128 bytes)
  // Skip the third pair (always [1, 0])
  for (let j = 0; j < 2; j++) {
    const bytes1 = leInt2Buff(unstringifyBigInts(vkData.vk_delta_2[j][0]), 32);
    const bytes2 = leInt2Buff(unstringifyBigInts(vkData.vk_delta_2[j][1]), 32);
    const combined = [...bytes1, ...bytes2].reverse();
    processedData.vk_delta_2.push(...combined.slice(0, 32));
    processedData.vk_delta_2.push(...combined.slice(32, 64));
  }
  
  // Process IC points (G1 points - 64 bytes each)
  for (let ic of vkData.IC) {
    const icPoint = [];
    // Skip the third coordinate (always 1)
    for (let j = 0; j < 2; j++) {
      const bytes = leInt2Buff(unstringifyBigInts(ic[j]), 32).reverse();
      icPoint.push(...bytes);
    }
    processedData.IC.push(icPoint);
  }
  
  // Generate the Rust file
  let rustCode = `use groth16_solana::groth16::Groth16Verifyingkey;

pub const VERIFYINGKEY: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: ${vkData.nPublic - 1},

    vk_alpha_g1: [
        ${processedData.vk_alpha_1.join(', ')},
    ],

    vk_beta_g2: [
        ${processedData.vk_beta_2.join(', ')},
    ],

    vk_gamme_g2: [
        ${processedData.vk_gamma_2.join(', ')},
    ],

    vk_delta_g2: [
        ${processedData.vk_delta_2.join(', ')},
    ],

    vk_ic: &[`;
  
  // Add IC points
  for (let icPoint of processedData.IC) {
    rustCode += `
        [
            ${icPoint.join(', ')},
        ],`;
  }
  
  rustCode += `
    ]
};

/// Get the verifying key for the withdraw circuit
pub fn get_circuit_verifying_key() -> &'static Groth16Verifyingkey {
    &VERIFYINGKEY
}
`;
  
  // Write the Rust file
  const outputFile = path.join(outputPath, "verifying_key.rs");
  fs.writeFileSync(outputFile, rustCode);
  
  console.log("Successfully generated:", outputFile);
  console.log("Verifying key stats:");
  console.log("- Alpha G1: 64 bytes");
  console.log("- Beta G2: 128 bytes");
  console.log("- Gamma G2: 128 bytes");
  console.log("- Delta G2: 128 bytes");
  console.log("- IC points:", processedData.IC.length, "x 64 bytes each");
  console.log("- Total size:", 64 + 128 + 128 + 128 + (processedData.IC.length * 64), "bytes");
}

main().catch(console.error);