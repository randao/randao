import { ethers } from "hardhat";

async function main() {
  const Randao = await ethers.getContractFactory("Randao");
  try {
    const randao = await Randao.deploy();
    await randao.deployed();
    console.log("Randao deployed to ", randao.address);
  } catch(err) { 
    console.log("Randao deploy error: ", err);
  }
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
