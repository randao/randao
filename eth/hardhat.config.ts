import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";

const config: HardhatUserConfig = {
  solidity: "0.8.17",
  defaultNetwork: 'hardhat',
  networks: {
    hardhat:
    {
      accounts: [
        {
          privateKey: '0xb501fc5879f214ee8be2832e43955ac0f19e20d1f7e33436d6746ac889dc043d',
          balance: '100000000000000000000',
        },
        {
          privateKey: '0x523170AAE57904F24FFE1F61B7E4FF9E9A0CE7557987C2FC034EACB1C267B4AE',
          balance: '100000000000000000000',
        },
        {
          privateKey: '0x67195c963ff445314e667112ab22f4a7404bad7f9746564eb409b9bb8c6aed32',
          balance: '100000000000000000000',
        },
      ],
      blockGasLimit: 100000000000,
      gasPrice: 1,
      chainId: 2152,
      initialBaseFeePerGas: 1,
      mining: {
        auto: true,
        interval: 5000,
      },
    },
    localhost: {
      url: "http://127.0.0.1:8545",
      accounts: ["0xb501fc5879f214ee8be2832e43955ac0f19e20d1f7e33436d6746ac889dc043d"],
      chainId: 2152,
    },
    testnet: {
      url: "https://prod-testnet.prod.findora.org:8545",
      accounts: ["0xb501fc5879f214ee8be2832e43955ac0f19e20d1f7e33436d6746ac889dc043d"],
      chainId: 2153,
    },
    mainnet: {
      url: "https://prod-mainnet.prod.findora.org:8545",
      accounts: ["0xb501fc5879f214ee8be2832e43955ac0f19e20d1f7e33436d6746ac889dc043d"],
      chainId: 2152,
    }
  }
};

export default config;

